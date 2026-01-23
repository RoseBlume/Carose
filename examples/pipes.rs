use carose::{
    Window, Menu, TextAlign,
    colors::{WHITE, RED, BLACK},
    sprites::{SpriteType, SpriteRender, Sprite, Vector},
    controls::Key,
    audio::{Audio, Bgs, SoundSource, BuiltInSound},
};
use rand::Rng;
use std::{thread, time::Duration, time::Instant, process};

fn main() {
    // --- Audio ---
    let bgs = Bgs::new(SoundSource::File("assets/audio/bgs/Crimson Turn-Based Clash.wav"));
    bgs.playing(true);
    let audio = Audio {};

    // --- Window ---
    let mut window = Window::new("Flappy Bird Clone", 400, 600);
    window.set_background_color(BLACK);

    // --- Main Menu ---
    let mut menu = Menu::new(vec!["Play", "Exit"], RED, WHITE);
    let mut player_index: usize;
    let mut rng = rand::rng();

    loop {
        window.update_controls();
        menu.draw(&mut window, "main_menu");

        if window.controls.clicked(Key::Char('w')) || window.controls.clicked(Key::Up) { menu.move_up(); }
        if window.controls.clicked(Key::Char('s')) || window.controls.clicked(Key::Down) { menu.move_down(); }

        if window.controls.clicked(Key::Enter) {
            match menu.current() {
                "Play" => {
                    remove_menu_text(&mut window, &menu, "main_menu");

                    // --- Create Player ---
                    player_index = window.sprites.len();
                    window.sprites.push(Sprite {
                        sprite_type: SpriteType::Player,
                        health: 1,
                        position: (100, 300),
                        size: (30, 30),
                        render: SpriteRender::Color(WHITE),
                        vectors: vec![Vector::Velocity(0, 0)], // Gravity will pull down
                        is_solid: false,
                    });
                    break;
                }
                "Exit" => return,
                _ => {}
            }
        }

        window.draw();
        thread::sleep(Duration::from_millis(30));
    }

    // --- Game Variables ---
    let mut score = 0;
    let mut last_pipe_spawn = Instant::now();
    let pipe_interval = 2.0; // seconds
    let pipe_gap = 150;
    let mut paused = false;
    let mut player_dead = false;
    let score_id = "score";
    window.show_text(score_id, &format!("Score: {}", score), (10, 10), 4, WHITE, TextAlign::AutoFit);

    // --- Game Loop ---
    while window.is_open() {
        window.update_controls();
        window.update_text(score_id, &format!("Score: {}", score));
        let (width, height) = window.get_size();
        // --- Pause ---
        if window.controls.clicked(Key::Escape) { paused = !paused; }
        if window.controls.pressed(Key::LeftCtrl) && window.controls.clicked(Key::Char('c')) {
            process::exit(0);
        }
        if paused { window.draw(); thread::sleep(Duration::from_millis(30)); continue; }
        score += 1;
        if !player_dead {
            // --- Flap ---
            if window.controls.clicked(Key::Space) {
                window.sprites[player_index].set_velocity(0, -12); // flap upward
                audio.play(SoundSource::BuiltIn(BuiltInSound::Shoot));
            }

            // --- Gravity ---
            
            window.sprites[player_index].set_acceleration(0, 1); // constant downward pull
        }

        // --- Spawn Pipes ---
        if last_pipe_spawn.elapsed().as_secs_f32() > pipe_interval {
            spawn_pipe(&mut window, &mut rng, width, height, pipe_gap);
            last_pipe_spawn = Instant::now();
        }

        // --- Apply Vectors & Physics ---
        window.apply_vectors();

        // --- Collision ---
        if !player_dead {
            window.change_health_offscreen(SpriteType::Player, - 100);
            window.change_health_on_collision(SpriteType::Player, SpriteType::Custom("Pipe"), -1);
            if window.sprites[player_index].health <= 0 {
                player_dead = true;
                audio.play(SoundSource::File("assets/audio/sfx/hit.wav"));
            }
        }

        // --- Remove offscreen pipes ---
        window.remove_if_out_of_screen(SpriteType::Custom("Pipe"));

        // --- Score Increment ---
        for i in 0..window.sprites.len() {
            if window.sprites[i].health <= 0 {
                window.update_text(score_id, &format!("Score: {}", score));
            }
        }


        window.draw();

        // --- Player Death ---
        if player_dead {
            thread::sleep(Duration::from_secs(1));
            // Reset game
            window.sprites.clear();
            score = 0;
            player_dead = false;
            last_pipe_spawn = Instant::now();
            player_index = window.sprites.len();
            window.sprites.push(Sprite {
                sprite_type: SpriteType::Player,
                health: 1,
                position: (100, 300),
                size: (30, 30),
                render: SpriteRender::Color(WHITE),
                vectors: vec![Vector::Velocity(0, 0)],
                is_solid: false,
            });
        }

    }
}

// --- Helper Functions ---

fn spawn_pipe(window: &mut Window, rng: &mut impl Rng, width: usize, height: usize, gap: usize) {
    let gap_y = rng.random_range(50..(height - gap - 50));

    // Top pipe
    window.sprites.push(Sprite {
        sprite_type: SpriteType::Custom("Pipe"),
        health: 1,
        position: (width, 0),
        size: (50, gap_y),
        render: SpriteRender::Color(0x00FF00),
        vectors: vec![Vector::Velocity(-5, 0)],
        is_solid: false,
    });

    // Bottom pipe
    window.sprites.push(Sprite {
        sprite_type: SpriteType::Custom("Pipe"),
        health: 1,
        position: (width, gap_y + gap),
        size: (50, height - gap_y - gap),
        render: SpriteRender::Color(0x00FF00),
        vectors: vec![Vector::Velocity(-5, 0)],
        is_solid: false,
    });
}

fn remove_menu_text(window: &mut Window, menu: &Menu, prefix: &str) {
    for i in 0..menu.options.len() {
        window.remove_text(&format!("{}_{}", prefix, i));
    }
}
