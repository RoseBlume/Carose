use carose::{Window, TextAlign, Menu};
use carose::colors::{WHITE, RED, FOREST_GREEN, DARK_GREEN, BLACK};
use carose::sprites::{Direction, SpriteType, SpriteRender, Sprite};
use carose::audio::{Audio, Bgs, SoundSource, BuiltInSound};
use carose::controls::{Keyboard, Key};
use rand::Rng;
use std::thread;
use std::time::{Duration, Instant};

struct EnemyData { index: usize, x_base: i32, amplitude: f32, frequency: f32, speed_y: usize, t: f32 }


fn create_player() -> Sprite {
    let bitmap: Vec<Vec<u32>> = (0..8).map(|row| {
        (0..8).map(|col| if (row + col) % 2 == 0 { FOREST_GREEN } else { DARK_GREEN }).collect()
    }).collect();

    let mut sprite = Sprite {
        sprite_type: SpriteType::Player,
        health: 100,
        position: (375, 500),
        size: (8, 8),
        render: SpriteRender::Bitmap { pixels: bitmap },
        velocity: (0.0, 0.0),
        gravity: Direction::None,
        is_solid: false,
    };
    sprite.upscale(8);
    sprite
}

fn remove_menu_text(window: &mut Window, menu: &Menu, prefix: &str) {
    for i in 0..menu.options.len() {
        window.remove_text(&format!("{}_{}", prefix, i));
    }
}

fn main_menu(window: &mut Window, keyboard: &mut Keyboard) {
    let mut menu = Menu::new(vec!["Play", "Exit"], RED, BLACK);
    window.set_background_color(WHITE);
    loop {
        keyboard.update();
        menu.draw(window, "main_option");

        if keyboard.clicked(Key::Char('w')) { menu.move_up(); }
        if keyboard.clicked(Key::Char('s')) { menu.move_down(); }

        if keyboard.clicked(Key::Enter) {
            match menu.current() {
                "Play" => { window.sprites.push(create_player()); break; }
                "Exit" => std::process::exit(0),
                _ => {}
            }
        }

        window.draw();
        thread::sleep(Duration::from_millis(30));
    }
    remove_menu_text(window, &menu, "main_option");
    window.set_background_color(BLACK);
}

fn main() {
    let bgs = Bgs::new(SoundSource::File("assets/audio/bgs/Fog over the Old Road.wav"));
    bgs.playing(true);
    let mut window = Window::new("Arc Shooter", 800, 600);
    window.set_background_color(BLACK);
    let mut keyboard = Keyboard::new();
    let audio = Audio {};

    let mut player_index = window.sprites.len();
    // Start main menu
    main_menu(&mut window, &mut keyboard);

    // Create player
    


    let mut projectiles = Vec::new();
    let mut enemies: Vec<EnemyData> = Vec::new();
    let mut score = 0;
    let mut rng = rand::rng();
    let mut last_spawn = Instant::now();
    let mut spawn_rate: f32 = rng.random_range(1.3..1.5);
    let score_id = "score";
    let health_id = "health";
    window.show_text(score_id, &format!("Score: {}", score), (10, 10), 4, WHITE, TextAlign::AutoFit);
    window.show_text(health_id, &format!("Health: {}", 0), (10, 50), 4, WHITE, TextAlign::AutoFit);

    let mut paused = false;
    let mut pause_menu = Menu::new(vec!["Resume", "Exit"], RED, WHITE);

    while window.is_open() {
        keyboard.update();

        // --- Toggle pause ---
        if keyboard.clicked(Key::Escape) { paused = !paused; }

        // --- PAUSE MENU ---
        if paused {
            pause_menu.draw(&mut window, "pause_option");
            if keyboard.clicked(Key::Char('w')) { pause_menu.move_up(); }
            if keyboard.clicked(Key::Char('s')) { pause_menu.move_down(); }

            if keyboard.clicked(Key::Enter) {
                match pause_menu.current() {
                    "Resume" => { paused = false; remove_menu_text(&mut window, &pause_menu, "pause_option"); }
                    "Exit" => {
                        remove_menu_text(&mut window, &pause_menu, "pause_option");
                        window.sprites.clear();
                        main_menu(&mut window, &mut keyboard);
                        window.sprites.push(create_player());
                        player_index = window.sprites.len() - 1;
                        paused = false;
                    }
                    _ => {}
                }
            }
            window.draw();
            continue; // skip game update while paused
        }

        // --- Update score display ---
        window.update_text(score_id, &format!("Score: {}", score));
        window.update_text(health_id, &format!("Health: {}", window.sprites[player_index].health));
        // --- Player movement & shooting ---
        let mut pos = window.sprites[player_index].position;
        if window.is_focused() {
            if keyboard.pressed(Key::Char('a')) { pos.0 = pos.0.saturating_sub(10); }
            if keyboard.pressed(Key::Char('d')) { pos.0 = (pos.0 + 10).min(750); }
            if keyboard.clicked(Key::Space) {
                let idx = window.sprites.len();
                window.sprites.push(Sprite {
                    sprite_type: SpriteType::Projectile,
                    health: 1,
                    position: (pos.0 + 20, pos.1),
                    size: (10, 10),
                    render: SpriteRender::Color(WHITE),
                    velocity: (0.0, 0.0),
                    gravity: Direction::None,
                    is_solid: false,
                });
                projectiles.push(idx);
                audio.play(SoundSource::BuiltIn(BuiltInSound::Shoot));
            }
        }
        window.move_sprite(player_index, pos);

        // --- Spawn enemies ---
        if last_spawn.elapsed().as_secs_f32() > spawn_rate {
            spawn_rate = rng.random_range(1.0..1.2);
            let x = rng.random_range(50..750);
            let idx = window.sprites.len();
            window.sprites.push(Sprite {
                sprite_type: SpriteType::Enemy,
                health: 30,
                position: (x, 0),
                size: (50, 50),
                render: SpriteRender::Color(RED),
                velocity: (0.0, 0.0),
                gravity: Direction::None,
                is_solid: false,
            });
            enemies.push(EnemyData { index: idx, x_base: x as i32, amplitude: rng.random_range(30.0..80.0), frequency: rng.random_range(0.02..0.05), speed_y: rng.random_range(4..6), t: 0.0 });
            last_spawn = Instant::now();
        }

        // --- Movement & collisions ---
        let mut dead_sprites = Vec::new();

        // Projectiles
        for &p in &projectiles {
            if p >= window.sprites.len() { dead_sprites.push(p); continue; }
            let mut pp = window.sprites[p].position;
            pp.1 = pp.1.saturating_sub(10);
            if pp.1 == 0 { dead_sprites.push(p); } else { window.move_sprite(p, pp); }
        }

        // Enemies
        for e in &mut enemies {
            if e.index >= window.sprites.len() { continue; }
            let mut ep = window.sprites[e.index].position;
            ep.1 += e.speed_y;
            if ep.1 >= 550 { dead_sprites.push(e.index); continue; }

            e.t += 1.0;
            ep.0 = (e.x_base + (e.amplitude * (e.t * e.frequency).sin()) as i32 + ((pos.0 as i32 - e.x_base) / 25).clamp(-4, 4))
                .clamp(0, 750) as usize;
            window.move_sprite(e.index, ep);

            // Projectile collision
            for &p in &projectiles {
                if p >= window.sprites.len() { continue; }
                let pp = window.sprites[p].position;
                if pp.0 < ep.0 + 50 && pp.0 + 10 > ep.0 && pp.1 < ep.1 + 50 && pp.1 + 10 > ep.1 {
                    window.sprites[e.index].health -= 10;
                    dead_sprites.push(p);
                }
            }

            // Player collision with enemy
            if pos.0 < ep.0 + 50 && pos.0 + 8 > ep.0 && pos.1 < ep.1 + 50 && pos.1 + 8 > ep.1 {
                window.sprites[player_index].health = window.sprites[player_index].health.saturating_sub(20);
                dead_sprites.push(e.index);
            }

            if window.sprites[e.index].health <= 0 { score += 100; dead_sprites.push(e.index); }
        }

        // --- Cleanup ---
        dead_sprites.sort_unstable(); dead_sprites.dedup();
        for &i in dead_sprites.iter().rev() { if i < window.sprites.len() { window.remove_sprite(i); } }
        projectiles.retain(|p| !dead_sprites.contains(p));
        for p in &mut projectiles { *p -= dead_sprites.iter().filter(|&&d| d < *p).count(); }
        enemies.retain(|e| !dead_sprites.contains(&e.index));
        for e in &mut enemies { e.index -= dead_sprites.iter().filter(|&&d| d < e.index).count(); }

        window.update_physics();
        window.draw();

        // --- Check player death ---
        if window.sprites[player_index].health == 0 {
            // Remove the dead player sprite so it doesn’t keep drawing
            window.remove_sprite(player_index);

            // Game Over menu
            let mut gameover_menu = Menu::new(vec!["Play Again", "Exit"], RED, WHITE);
            loop {
                keyboard.update();
                gameover_menu.draw(&mut window, "gameover_option");

                if keyboard.clicked(Key::Char('w')) { gameover_menu.move_up(); }
                if keyboard.clicked(Key::Char('s')) { gameover_menu.move_down(); }

                if keyboard.clicked(Key::Enter) {
                    match gameover_menu.current() {
                        "Play Again" => {
                            // Reset everything
                            window.sprites.clear();
                            projectiles.clear();
                            enemies.clear();
                            score = 0;

                            // Recreate player
                            let player = create_player();
                            player_index = window.sprites.len();
                            window.sprites.push(player);

                            remove_menu_text(&mut window, &gameover_menu, "gameover_option");
                            break; // exit menu and restart loop
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
