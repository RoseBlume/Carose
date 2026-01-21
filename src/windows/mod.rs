pub mod text;
mod background;


use text::{
    get_font_map,
    TextItem
};
use std::str::FromStr;
use minifb::{Window as MfWindow, WindowOptions};
use crate::sprites::{SpriteRender, Sprite, Direction};
use std::collections::HashMap;

pub enum Background {
    Color(u32),
    Image(Vec<u32>), // size must be width * height
}



pub struct Window {
    pub width: usize,
    pub height: usize,
    pub sprites: Vec<Sprite>,
    pub background: Option<Background>,
    pub texts: HashMap<String, TextItem>,
    window: minifb::Window,

    pub paused: bool,

}

impl Window {
    pub fn new(title: &str, width: usize, height: usize) -> Self {
        let mut window = MfWindow::new(
            title,
            width,
            height,
            WindowOptions {
                resize: false,
                ..WindowOptions::default()
            },
        )
        .expect("Failed to create window");

        window.set_target_fps(60);

        Self {
            width,
            height,
            sprites: Vec::new(),
            background: None,
            texts: HashMap::new(),
            window,
            paused: false,
        }
    }


    

    

    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }
    
    pub fn set_target_fps(&mut self, fps: usize) {
        self.window.set_target_fps(fps)
    }

    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }
    #[cfg(target_os = "windows")]
    pub fn set_icon(&mut self, icon: &str) {
        self.window.set_icon(minifb::Icon::from_str(icon).unwrap());
    }

    pub fn set_position(&mut self, x: isize, y: isize) {
        self.window.set_position(x, y);
    }

    pub fn get_position(&self) -> (isize, isize) {
        self.window.get_position()
    }

    pub fn set_topmost_always(&mut self, topmost: bool) {
        self.window.topmost(topmost);
    }

    pub fn set_cursor_visibility(&mut self, visible: bool) {
        self.window.set_cursor_visibility(visible);
    }

    pub fn get_size(&self) -> (usize, usize) {
        self.window.get_size()
    }
    pub fn is_focused(&mut self) -> bool {
        self.window.is_active()
    }
    
    pub fn update_physics(&mut self) {
        for sprite in &mut self.sprites {
            // Skip walls/floors if we had them
            if sprite.is_solid { continue; }

            // For now, we just set velocity & gravity fields, no actual movement
            sprite.velocity = (0.0, 0.0); // velocity not applied yet
            sprite.gravity = Direction::None; // gravity disabled for now

            // Position would normally be updated by velocity here:
            // let new_x = sprite.position.0 as f32 + sprite.velocity.0;
            // let new_y = sprite.position.1 as f32 + sprite.velocity.1;
            // sprite.position.0 = new_x as usize;
            // sprite.position.1 = new_y as usize;
        }
    }

    pub fn draw(&mut self) {
        // --- Create buffer with background ---
        let mut buffer = match &self.background {
            Some(Background::Color(color)) => vec![*color; self.width * self.height],
            Some(Background::Image(image)) => image.clone(),
            None => vec![0x000000; self.width * self.height],
        };

        // --- Draw sprites ---
        for sprite in &mut self.sprites {
            let (sx, sy) = sprite.position;

            match &mut sprite.render {
                SpriteRender::Color(color) => {
                    let (w, h) = sprite.size;
                    for y in 0..h {
                        for x in 0..w {
                            let px = sx + x;
                            let py = sy + y;
                            if px < self.width && py < self.height {
                                buffer[py * self.width + px] = *color;
                            }
                        }
                    }
                }

                SpriteRender::Bitmap { pixels } => {
                    let h = pixels.len();
                    if h == 0 { continue; }
                    let w = pixels[0].len();

                    for y in 0..h {
                        for x in 0..w {
                            let pixel = pixels[y][x];
                            if pixel == 0 { continue; }

                            let px = sx + x;
                            let py = sy + y;
                            if px < self.width && py < self.height {
                                buffer[py * self.width + px] = pixel;
                            }
                        }
                    }
                }

                SpriteRender::AnimatedBitmap {
                    frames,
                    frame_index,
                    frame_delay,
                    frame_timer,
                } => {
                    if frames.is_empty() { continue; }

                    let frame = &frames[*frame_index];
                    let h = frame.len();
                    if h == 0 { continue; }
                    let w = frame[0].len();

                    for y in 0..h {
                        for x in 0..w {
                            let pixel = frame[y][x];
                            if pixel == 0 { continue; }

                            let px = sx + x;
                            let py = sy + y;
                            if px < self.width && py < self.height {
                                buffer[py * self.width + px] = pixel;
                            }
                        }
                    }

                    // advance animation
                    *frame_timer += 1;
                    if *frame_timer >= *frame_delay {
                        *frame_timer = 0;
                        *frame_index = (*frame_index + 1) % frames.len();
                    }
                }
            }
        }

        // --- Draw texts using 5x5 bitmap font ---
        let font_map = get_font_map();

        for text_item in self.texts.values() {
            let (tx, ty) = text_item.position;
            let color = text_item.color;
            let size = text_item.size;

            for (i, c) in text_item.content.chars().enumerate() {
                let char_pixels = font_map
                    .get(&c.to_ascii_uppercase())
                    .unwrap_or(&font_map[&' ']);

                for y in 0..5 {
                    for x in 0..5 {
                        if char_pixels[y][x] != 0 {
                            for sy in 0..size {
                                for sx in 0..size {
                                    let px = tx + i * (5 * size + 7) + x * size + sx;
                                    let py = ty + y * size + sy;
                                    if px < self.width && py < self.height {
                                        buffer[py * self.width + px] = color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.window
            .update_with_buffer(&buffer, self.width, self.height)
            .unwrap();
    }
}

