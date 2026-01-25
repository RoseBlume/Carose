pub mod text;
mod background;
use crate::controls::Input;

use text::{
    get_font_map,
    TextItem
};
use minifb::{Window as MfWindow, WindowOptions};
use crate::sprites::{SpriteRender, Sprite};
use std::collections::HashMap;


pub enum Background {
    Color(u32),
    Image(Vec<Vec<u32>>), // size must be width * height
}



pub struct Window {
    pub width: usize,
    pub height: usize,
    pub sprites: Vec<Sprite>,
    pub background: Option<Background>,
    pub texts: HashMap<String, TextItem>,
    window: minifb::Window,

    pub controls: Input,

    pub paused: bool,
}

impl Window {
    /// Creates a new window and rendering context.
    ///
    /// Initializes an OS window with the given title and dimensions,
    /// enables resizing, sets a default target frame rate of 60 FPS,
    /// and prepares all internal rendering and input state.
    ///
    /// # Parameters
    /// - `title`: Title displayed in the window title bar.
    /// - `width`: Initial window width in pixels.
    /// - `height`: Initial window height in pixels.
    ///
    /// # Panics
    /// Panics if the window cannot be created.
    pub fn new(title: &str, width: usize, height: usize) -> Self {
        let mut window = MfWindow::new(
            title,
            width,
            height,
            WindowOptions {
                resize: true,
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

            controls: Input::new(),

            paused: false,
        }
    }

    /// Polls and updates input state for the current frame.
    ///
    /// Input is only processed while the window is focused.
    /// This should typically be called once per frame before
    /// reading input state.
    pub fn update_controls(&mut self) {
        let focused = self.window.is_active();
        self.controls.poll(focused);
    }

    /// Returns whether the window is currently open.
    ///
    /// This should be used as the main loop condition.
    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }
    /// Sets the target frames per second for the window.
    ///
    /// This controls how often the window redraws and how input
    /// events are processed.
    pub fn set_target_fps(&mut self, fps: usize) {
        self.window.set_target_fps(fps)
    }

    /// Updates the window title.
    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    /// Sets the window's position on the screen.
    ///
    /// Coordinates are in screen space.
    pub fn set_position(&mut self, x: isize, y: isize) {
        self.window.set_position(x, y);
    }

    /// Returns the current window position in screen coordinates.
    pub fn get_position(&self) -> (isize, isize) {
        self.window.get_position()
    }

    /// Forces the window to stay above all other windows when enabled.
    pub fn set_topmost_always(&mut self, topmost: bool) {
        self.window.topmost(topmost);
    }

    /// Shows or hides the mouse cursor while the window is focused.
    pub fn set_cursor_visibility(&mut self, visible: bool) {
        self.window.set_cursor_visibility(visible);
    }

    /// Returns the current window size.
    pub fn get_size(&self) -> (usize, usize) {
        self.window.get_size()
    }

    /// Returns the current window width in pixels.
    pub fn get_width(&self) -> usize {
        self.window.get_size().0
    }

    /// Returns the current window height in pixels.
    pub fn get_height(&self) -> usize {
        self.window.get_size().1
    }

    /// Returns whether the window currently has input focus.
    pub fn is_focused(&mut self) -> bool {
        self.window.is_active()
    }
    
    /// Renders a complete frame.
    ///
    /// This method:
    /// - Clears the screen using the configured background
    /// - Draws all sprites (including animated sprites)
    /// - Draws all text using a built-in 5x5 bitmap font
    /// - Advances sprite animations
    /// - Uploads the final frame buffer to the window
    ///
    /// This should be called once per frame.
    pub fn draw(&mut self) {
        // --- Create 2D buffer with background ---
        let mut buffer: Vec<Vec<u32>> = match &self.background {
            Some(Background::Color(color)) => {
                vec![vec![*color; self.width]; self.height]
            }
            Some(Background::Image(image)) => image.clone(),
            None => vec![vec![0x000000; self.width]; self.height],
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
                                buffer[py][px] = *color;
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
                                buffer[py][px] = pixel;
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
                                buffer[py][px] = pixel;
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
                                        buffer[py][px] = color;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // --- Flatten 2D buffer into 1D ---
        let flat_buffer: Vec<u32> = buffer.into_iter().flatten().collect();

        self.window
            .update_with_buffer(&flat_buffer, self.width, self.height)
            .unwrap();
    }

}

