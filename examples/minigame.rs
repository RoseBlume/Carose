use carose::{Window, TextAlign, Menu};
use carose::colors::{WHITE, RED, BLACK};
use carose::sprites::{SpriteType, SpriteRender, Sprite, Vector};
use carose::audio::{Audio, Bgs, SoundSource, BuiltInSound};
use carose::controls::Key;
use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};

fn remove_menu_text(window: &mut Window, menu: &Menu, prefix: &str) {
    for i in 0..menu.options.len() {
        window.remove_text(&format!("{}_{}", prefix, i));
    }
}

fn handle_menu_input(window: &mut Window, menu: &mut Menu) {
    if window.controls.clicked(Key::Char('w')) | window.controls.clicked(Key::Up) {
        menu.move_up();
    }
    if window.controls.clicked(Key::Char('s')) | window.controls.clicked(Key::Down) {
        menu.move_down();
    }
}

fn main_menu(window: &mut Window, bgs: &mut Bgs) -> usize {
    let mut menu = Menu::new(vec!["Play", "Exit"], RED, BLACK);
    window.set_background_color(WHITE);
    let player_index;

    loop {
        window.update_controls();
        let (width, height) = window.get_size();

        menu.draw(window, "main_option");
        handle_menu_input(window, &mut menu);

        if window.controls.clicked(Key::Enter) {
            match menu.current() {
                "Play" => {
                    let frames = (1..=3)
                        .map(|i| format!("assets/Sprites/Animated/Ship/shipsprite{}.bmp", i))
                        .collect::<Vec<_>>();

                    player_index = window.create_animated_bitmap_sprite_from_files(
                        ((width / 2).saturating_sub(64 / 2), height - 100),
                        100,
                        frames,
                        SpriteType::Player,
                        120,
                    );
                    break;
                }
                "Exit" => std::process::exit(0),
                _ => {}
            }
        }
        bgs.update_playlist();
        window.draw();
        thread::sleep(Duration::from_millis(30));
    }

    remove_menu_text(window, &menu, "main_option");
    window.set_background_color(BLACK);
    player_index
}

// --- Game Functions ---

fn spawn_falling(
    window: &mut Window,
    rng: &mut impl Rng,
    width: usize,
    sprite_type: SpriteType,
    size: (usize, usize),
    color: u32,
    health: i32,
    speed: i32,
) {
    let x = rng.random_range(50..(width - 50));
    window.sprites.push(Sprite {
        sprite_type,
        health,
        position: (x, 0),
        size,
        render: SpriteRender::Color(color),
        vectors: vec![Vector::Velocity(0, speed)],
        is_solid: false,
    });
}

// --- Main Loop ---

fn main() {
    let playlist: Vec<SoundSource> = vec![
        SoundSource::File("assets/audio/bgs/Crimson Gate Siege.wav"),
        SoundSource::File("assets/audio/bgs/Crimson Turn-Based Clash.wav"),
        SoundSource::File("assets/audio/bgs/Battle Textbox Theme.wav"),
        SoundSource::File("assets/audio/bgs/Dragonfire Siege.wav"),
        SoundSource::File("assets/audio/bgs/Fog over the Old Road.wav"),
        SoundSource::File("assets/audio/bgs/Obsidian Gate Awakens.wav"),
        SoundSource::File("assets/audio/bgs/Ruins of the Old Road.wav"),
    ];
    let mut bgs = Bgs::playlist(playlist);
    
    bgs.playing(true);

    let mut window = Window::new("Arc Shooter", 800, 600);
    window.set_background_color(BLACK);
    let audio = Audio::new();
    let mut rng = rand::rng();

    let mut player_index = main_menu(&mut window, &mut bgs);

    let mut projectiles = Vec::new();
    let mut score = 0;
    let mut spawn_rate: f32 = rng.random_range(1.3..1.5);
    let food_spawn_rate: f32 = 15.0;
    let mut last_spawn = Instant::now();
    let mut last_food_spawn = Instant::now();
    let score_id = "score";
    let health_id = "health";
    window.show_text(score_id, &format!("Score: {}", score), (10, 10), 4, WHITE, TextAlign::AutoFit);
    window.show_text(health_id, &format!("Health: {}", 0), (10, 50), 4, WHITE, TextAlign::AutoFit);

    let mut paused = false;
    let mut pause_menu = Menu::new(vec!["Resume", "Exit"], RED, WHITE);
    let mut player_dead = false;

    while window.is_open() {
        bgs.update_playlist();
        window.update_controls();
        let (width, _) = window.get_size();

        if !window.is_focused() { paused = true; }
        if window.controls.clicked(Key::Escape) { paused = true; }

        // --- Pause Menu ---
        if paused {
            pause_menu.draw(&mut window, "pause_option");
            handle_menu_input(&mut window, &mut pause_menu);

            if window.controls.clicked(Key::Enter) {
                match pause_menu.current() {
                    "Resume" => { paused = false; remove_menu_text(&mut window, &pause_menu, "pause_option"); }
                    "Exit" => {
                        remove_menu_text(&mut window, &pause_menu, "pause_option");
                        window.sprites.clear();
                        player_index = main_menu(&mut window, &mut bgs);
                        player_dead = false;
                        paused = false;
                    }
                    _ => {}
                }
            }
            window.draw();
            continue;
        }

        // --- Update HUD ---
        let health = if player_dead { 0 } else { window.sprites[player_index].health };
        window.update_text(score_id, &format!("Score: {}", score));
        window.update_text(health_id, &format!("Health: {}", health));

        // --- Player Actions ---
        if !player_dead {
            let mut pos = window.sprites[player_index].position;
            if window.controls.pressed(Key::Char('a')) { pos.0 = pos.0.saturating_sub(10); }
            if window.controls.pressed(Key::Char('d')) { pos.0 = (pos.0 + 10).min(width - window.sprites[player_index].size.0); }

            if window.controls.clicked(Key::Space) {
                let idx = window.sprites.len();
                window.sprites.push(Sprite {
                    sprite_type: SpriteType::Projectile,
                    health: 1,
                    position: (pos.0 + 20, pos.1),
                    size: (10, 10),
                    render: SpriteRender::Color(WHITE),
                    is_solid: false,
                    vectors: vec![Vector::Velocity(0, -10)],
                });
                projectiles.push(idx);
                audio.play(SoundSource::BuiltIn(BuiltInSound::Shoot));
            }
            window.move_sprite(player_index, pos);
        }

        // --- Enemy Spawn ---
        if last_spawn.elapsed().as_secs_f32() > spawn_rate {
            spawn_rate = rng.random_range(1.0..1.2);
            let speed = rng.random_range(7..13);
            spawn_falling(&mut window, &mut rng, width, SpriteType::Enemy, (50, 50), RED, 30, speed);
            last_spawn = Instant::now();
        }

        if last_food_spawn.elapsed().as_secs_f32() > food_spawn_rate {
            spawn_rate = rng.random_range(1.0..1.2);
            spawn_falling(&mut window, &mut rng, width, SpriteType::Custom("Food"), (30, 30), 0x00FF00, 1, 10);
            last_food_spawn = Instant::now();
        }
        window.change_health_on_collision(SpriteType::Player, SpriteType::Custom("Food"), 20);
        window.remove_on_collision(SpriteType::Player, SpriteType::Custom("Food"));
        window.change_health_on_collision(SpriteType::Enemy, SpriteType::Projectile, -10);
        window.change_health_on_collision(SpriteType::Projectile, SpriteType::Enemy, -1000);
        window.change_health_on_collision(SpriteType::Player, SpriteType::Enemy, -10);
        window.remove_if_out_of_screen(SpriteType::Projectile);
        window.remove_if_out_of_screen(SpriteType::Enemy);
        window.increment_on_sprite_death(SpriteType::Enemy, &mut score, 100);
        window.on_death(SpriteType::Enemy, |_, _| {
            audio.play(SoundSource::File("assets/audio/sfx/hit.wav"));
        });
        window.change_health_on_collision(SpriteType::Enemy, SpriteType::Player, -200);
        window.remove_on_death(SpriteType::Projectile);
        window.remove_on_death(SpriteType::Enemy);
        window.prevent_leaving_screen(SpriteType::Player);
        window.add_vector(SpriteType::Projectile, Vector::Velocity(0, -10));
        window.apply_vectors();

        window.draw();

        // --- Player Death Check ---
        if !player_dead && window.sprites[player_index].health == 0 {
            window.remove_sprite(player_index);
            window.update_text(health_id, &format!("Health: {}", health));

            // --- Game Over Menu ---
            let mut gameover_menu = Menu::new(vec!["Play Again", "Exit"], RED, WHITE);
            loop {
                window.update_controls();
                gameover_menu.draw(&mut window, "gameover_option");
                handle_menu_input(&mut window, &mut gameover_menu);

                if window.controls.clicked(Key::Enter) {
                    match gameover_menu.current() {
                        "Play Again" => {
                            window.sprites.clear();
                            projectiles.clear();
                            score = 0;
                            
                            // Recreate player
                            let frames = (1..=3)
                                .map(|i| format!("assets/Sprites/Animated/Ship/shipsprite{}.bmp", i))
                                .collect::<Vec<_>>();
                            player_index = window.create_animated_bitmap_sprite_from_files(
                                (375, 500), 100, frames, SpriteType::Player, 120
                            );

                            remove_menu_text(&mut window, &gameover_menu, "gameover_option");
                            player_dead = false;
                            break;
                        }
                        "Exit" => std::process::exit(0),
                        _ => {}
                    }
                }
                window.draw();
                thread::sleep(Duration::from_millis(30));
            }
        }
    }
}
