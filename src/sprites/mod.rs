use crate::Window;
mod vectors;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Custom(i32, i32),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vector {
    Velocity(i32, i32),
    Acceleration(i32, i32),
}

#[derive(Clone)]
pub enum SpriteRender {
    Color(u32),

    Bitmap {
        pixels: Vec<Vec<u32>>, // row-major, 0 = transparent
    },

    /// Animated bitmap (multi-color, variable size)
    AnimatedBitmap {
        frames: Vec<Vec<Vec<u32>>>, // frames → rows → pixels
        frame_index: usize,
        frame_delay: u32,
        frame_timer: u32,
    },
}


#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SpriteType {
    Player,
    Enemy,
    Projectile,
    Wall,
    Overlay,
    Custom(&'static str),
}

pub struct Sprite {
    pub sprite_type: SpriteType,
    pub health: i32,
    pub position: (usize, usize),
    pub size: (usize, usize),
    pub render: SpriteRender,
    pub is_solid: bool,
    pub vectors: Vec<Vector>, // new
}



impl Sprite {
    pub fn upscale(&mut self, factor: usize) {
        if factor <= 1 {
            return;
        }

        match &mut self.render {
            SpriteRender::Bitmap { pixels } => {
                let original = pixels.clone();
                let h = original.len();
                let w = if h > 0 { original[0].len() } else { 0 };

                let mut upscaled = vec![vec![0u32; w * factor]; h * factor];

                for y in 0..h {
                    for x in 0..w {
                        let color = original[y][x];
                        if color == 0 { continue; }

                        for sy in 0..factor {
                            for sx in 0..factor {
                                upscaled[y * factor + sy][x * factor + sx] = color;
                            }
                        }
                    }
                }

                *pixels = upscaled;
                self.size = (w * factor, h * factor);
            }

            SpriteRender::AnimatedBitmap { frames, .. } => {
                for frame in frames.iter_mut() {
                    let original = frame.clone();
                    let h = original.len();
                    let w = if h > 0 { original[0].len() } else { 0 };

                    let mut upscaled = vec![vec![0u32; w * factor]; h * factor];

                    for y in 0..h {
                        for x in 0..w {
                            let color = original[y][x];
                            if color == 0 { continue; }

                            for sy in 0..factor {
                                for sx in 0..factor {
                                    upscaled[y * factor + sy][x * factor + sx] = color;
                                }
                            }
                        }
                    }

                    *frame = upscaled;
                }

                if let Some(frame) = frames.first() {
                    self.size = (frame[0].len(), frame.len());
                }
            }

            SpriteRender::Color(_) => {
                // Solid-color sprites just scale logically
                self.size = (self.size.0 * factor, self.size.1 * factor);
            }
        }
    }
}

impl Window {
    fn load_bitmap_from_file(path: &str) -> Vec<Vec<u32>> {
        let img = image::open(path).expect("Failed to open image").to_rgba8();
        let (width, height) = img.dimensions();
        let mut bitmap = vec![vec![0; width as usize]; height as usize];

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                // Convert RGBA to u32 (ARGB format)
                let argb = ((pixel[3] as u32) << 24) // alpha
                    | ((pixel[0] as u32) << 16)     // red
                    | ((pixel[1] as u32) << 8)      // green
                    | (pixel[2] as u32);            // blue
                bitmap[y as usize][x as usize] = argb;
            }
        }

        bitmap
    }

    // Load a single bitmap sprite from a file
    pub fn create_bitmap_sprite_from_file(
        &mut self,
        position: (usize, usize),
        path: &str,
        sprite_type: SpriteType,
    ) -> usize {
        let bitmap = Self::load_bitmap_from_file(path);
        self.create_bitmap_sprite(position, bitmap, sprite_type)
    }

    // Load multiple files as an animated sprite
    pub fn create_animated_bitmap_sprite_from_files(
        &mut self,
        position: (usize, usize),
        health: i32,
        paths: Vec<String>,
        sprite_type: SpriteType,
        frame_delay: u32,
    ) -> usize {
        let frames: Vec<Vec<Vec<u32>>> = paths
            .iter()
            .map(|path| Self::load_bitmap_from_file(path))
            .collect();
        self.create_animated_bitmap_sprite(position, health, frames, sprite_type, frame_delay)
    }
    pub fn create_animated_sprite(
        &mut self,
        position: (usize, usize),
        size: (usize, usize),
        sprite_type: SpriteType,
        health: i32,
        frames: Vec<Vec<Vec<u32>>>,
        frame_delay: u32,
    ) -> usize {
        self.sprites.push(Sprite {
            sprite_type,
            health,
            position,
            size,
            render: SpriteRender::AnimatedBitmap {
                frames,
                frame_index: 0,
                frame_delay,
                frame_timer: 0,
            },    // gravity disabled for now
            is_solid: false,             // not a wall/floor
            vectors: Vec::new(),
        });
        self.sprites.len() - 1
    }
    pub fn create_colored_sprite(
        &mut self,
        position: (usize, usize),
        size: (usize, usize),
        sprite_type: SpriteType,
        health: i32,
        color: u32,
    ) -> usize {
        self.sprites.push(Sprite {
            sprite_type,
            health,
            position,
            size,
            render: SpriteRender::Color(color),    // gravity disabled for now
            is_solid: false,             // not a wall/floor
            vectors: vec![Vector::Velocity(0, 0)]
        });
        self.sprites.len() - 1
    }
    pub fn create_bitmap_sprite(
        &mut self,
        position: (usize, usize),
        bitmap: Vec<Vec<u32>>, // 2D pixels (0 = transparent)
        sprite_type: SpriteType,
    ) -> usize {
        let height = bitmap.len();
        let width = if height > 0 { bitmap[0].len() } else { 0 };

        self.sprites.push(Sprite {
            sprite_type,
            health: 1,
            position,
            size: (width, height), // logical size
            render: SpriteRender::Bitmap {
                pixels: bitmap,
            },
            is_solid: false,
            vectors: Vec::new(),
        });

        self.sprites.len() - 1
    }
    pub fn create_animated_bitmap_sprite(
        &mut self,
        position: (usize, usize),
        health: i32,
        bitmaps: Vec<Vec<Vec<u32>>>, // frames → rows → pixels
        sprite_type: SpriteType,
        frame_delay: u32,
    ) -> usize {
        let (width, height) = if let Some(frame) = bitmaps.first() {
            let h = frame.len();
            let w = if h > 0 { frame[0].len() } else { 0 };
            (w, h)
        } else {
            (0, 0)
        };

        self.sprites.push(Sprite {
            sprite_type,
            health,
            position,
            size: (width, height), // logical size
            render: SpriteRender::AnimatedBitmap {
                frames: bitmaps,
                frame_index: 0,
                frame_delay,
                frame_timer: 0,
            },
            is_solid: false,
            vectors: Vec::new(),
        });

        self.sprites.len() - 1
    }
    pub fn create_wall(&mut self, position: (usize, usize), size: (usize, usize)) -> usize {
        self.sprites.push(Sprite {
            sprite_type: SpriteType::Wall,
            health: i32::MAX, // indestructible
            position,
            size,
            render: SpriteRender::Color(0x555555),
            is_solid: true,
            vectors: Vec::new(),
        });
        self.sprites.len() - 1
    }
    pub fn advance_animation(render: &mut SpriteRender) {
        if let SpriteRender::AnimatedBitmap {
            frames,
            frame_index,
            frame_delay,
            frame_timer,
        } = render {
            *frame_timer += 1;
            if *frame_timer >= *frame_delay {
                *frame_timer = 0;
                *frame_index = (*frame_index + 1) % frames.len();
            }
        }
    }
    pub fn move_sprite(&mut self, index: usize, new_pos: (usize, usize)) {
        if let Some(sprite) = self.sprites.get_mut(index) {
            sprite.position = new_pos;
        }
    }

    pub fn remove_sprite(&mut self, index: usize) {
        if index < self.sprites.len() {
            self.sprites.remove(index);
        }
    }

    pub fn on_death<F>(&mut self, sprite_type: SpriteType, mut on_death: F)
    where
        F: FnMut(&mut Window, usize),
    {
        for i in 0..self.sprites.len() {
            if self.sprites[i].sprite_type == sprite_type
                && self.sprites[i].health <= 0
            {
                on_death(self, i);
            }
        }
    }

    pub fn on_collision<F>(
        &mut self,
        a_type: SpriteType,
        b_type: SpriteType,
        mut on_collision: F,
    )
    where
        F: FnMut(&mut Window, usize, usize),
    {
        let len = self.sprites.len();

        for i in 0..len {
            if self.sprites[i].sprite_type != a_type {
                continue;
            }

            let (x1, y1) = self.sprites[i].position;
            let (w1, h1) = self.sprites[i].size;

            for j in 0..len {
                if i == j || self.sprites[j].sprite_type != b_type {
                    continue;
                }

                let (x2, y2) = self.sprites[j].position;
                let (w2, h2) = self.sprites[j].size;

                let intersects =
                    x1 < x2 + w2 &&
                    x1 + w1 > x2 &&
                    y1 < y2 + h2 &&
                    y1 + h1 > y2;

                if intersects {
                    on_collision(self, i, j);
                }
            }
        }
    }



    pub fn change_health_on_collision(
        &mut self,
        target_type: SpriteType,
        collider_type: SpriteType,
        health: i32,
    ) {
        let len = self.sprites.len();
        for i in 0..len {
            if self.sprites[i].sprite_type != target_type { continue; }

            for j in 0..len {
                if i == j || self.sprites[j].sprite_type != collider_type { continue; }

                let (s1, s2) = if i < j {
                    let (left, right) = self.sprites.split_at_mut(j);
                    (&mut left[i], &mut right[0])
                } else {
                    let (left, right) = self.sprites.split_at_mut(i);
                    (&mut right[0], &mut left[j])
                };

                if s1.position.0 < s2.position.0 + s2.size.0
                    && s1.position.0 + s1.size.0 > s2.position.0
                    && s1.position.1 < s2.position.1 + s2.size.1
                    && s1.position.1 + s1.size.1 > s2.position.1
                {
                    s1.health = s1.health.saturating_add(health);
                }
            }
        }
    }
    pub fn remove_on_death(&mut self, sprite_type: SpriteType) {
        let mut dead_indices = Vec::new();
        for (i, sprite) in self.sprites.iter().enumerate() {
            if sprite.sprite_type == sprite_type && sprite.health <= 0 {
                dead_indices.push(i);
            }
        }

        for &i in dead_indices.iter().rev() {
            self.remove_sprite(i);
        }
    }
    pub fn remove_on_collision(
        &mut self,
        collider_type: SpriteType,
        remove_type: SpriteType,
    ) {
        let mut dead_indices = Vec::new();
        let len = self.sprites.len();

        for i in 0..len {
            if self.sprites[i].sprite_type != remove_type {
                continue;
            }

            let (x1, y1) = self.sprites[i].position;
            let (w1, h1) = self.sprites[i].size;

            for j in 0..len {
                if i == j || self.sprites[j].sprite_type != collider_type {
                    continue;
                }

                let (x2, y2) = self.sprites[j].position;
                let (w2, h2) = self.sprites[j].size;

                let intersects =
                    x1 < x2 + w2 &&
                    x1 + w1 > x2 &&
                    y1 < y2 + h2 &&
                    y1 + h1 > y2;

                if intersects {
                    dead_indices.push(i);
                    break; // stop checking once collided
                }
            }
        }

        for &i in dead_indices.iter().rev() {
            self.remove_sprite(i);
        }
    }

    /// Remove sprites completely outside the screen
    pub fn remove_if_out_of_screen(&mut self, sprite_type: SpriteType) {
        let mut dead_indices = Vec::new();

        for (i, sprite) in self.sprites.iter().enumerate() {
            if sprite.sprite_type != sprite_type { continue; }

            let x = sprite.position.0 as i32;
            let y = sprite.position.1 as i32;
            let w = sprite.size.0 as i32;
            let h = sprite.size.1 as i32;

            // Fully outside the screen
            if x + w <= 0 || x >= self.width as i32 || y + h <= 0 || y >= self.height as i32 {
                dead_indices.push(i);
            }
        }

        // Remove from back to avoid index shift
        for &i in dead_indices.iter().rev() {
            self.remove_sprite(i);
        }
    }


    /// Prevent sprites of a given type from leaving the screen bounds
    pub fn prevent_leaving_screen(&mut self, sprite_type: SpriteType) {
        for sprite in self.sprites.iter_mut() {
            if sprite.sprite_type != sprite_type { continue; }

            let (w, h) = sprite.size;
            let (x, y) = sprite.position;

            let new_x = x.clamp(0, self.width.saturating_sub(w));
            let new_y = y.clamp(0, self.height.saturating_sub(h));

            sprite.position = (new_x, new_y);
        }
    }
    pub fn increment_on_sprite_death(
        &mut self,
        sprite_type: SpriteType,
        score: &mut i32,
        points: i32,
    ) {
        for sprite in self.sprites.iter_mut() {
            if sprite.sprite_type != sprite_type {
                continue;
            }

            if sprite.health <= 0 {
                *score += points;

                // Prevent double-scoring
                sprite.health = i32::MIN;
            }
        }
    }

    pub fn change_health_offscreen(&mut self, sprite_type: SpriteType, health_change: i32) {
        let (width, height) = self.get_size();
        for sprite in &mut self.sprites {
            if sprite.sprite_type != sprite_type {
                continue;
            }

            let (x, y) = sprite.position;
            let (w, h) = sprite.size;

            // Check if fully outside screen
            if x + w <= 0 || x >= width || y + h <= 0 || y >= height {
                sprite.health = sprite.health.saturating_add(health_change);
            }
        }
    }
}