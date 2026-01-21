use crate::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
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
    Custom(String),
}

pub struct Sprite {
    pub sprite_type: SpriteType,
    pub health: i32,
    pub position: (usize, usize),
    pub size: (usize, usize),
    pub render: SpriteRender,
    
    // Physics
    pub velocity: (f32, f32), // (vx, vy)
    pub gravity: Direction,
    pub is_solid: bool,       // true if it acts as a wall/floor
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
            },
            velocity: (0.0, 0.0),       // velocity initialized but not applied yet
            gravity: Direction::None,    // gravity disabled for now
            is_solid: false,             // not a wall/floor
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
            render: SpriteRender::Color(color),
            velocity: (0.0, 0.0),       // velocity initialized but not applied yet
            gravity: Direction::None,    // gravity disabled for now
            is_solid: false,             // not a wall/floor
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
            velocity: (0.0, 0.0),
            gravity: Direction::None,
            is_solid: false,
        });

        self.sprites.len() - 1
    }
    pub fn create_animated_bitmap_sprite(
        &mut self,
        position: (usize, usize),
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
            health: 1,
            position,
            size: (width, height), // logical size
            render: SpriteRender::AnimatedBitmap {
                frames: bitmaps,
                frame_index: 0,
                frame_delay,
                frame_timer: 0,
            },
            velocity: (0.0, 0.0),
            gravity: Direction::None,
            is_solid: false,
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
            velocity: (0.0, 0.0),
            gravity: Direction::None,
            is_solid: true,
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

    pub fn apply_damage_on_collision(
        &mut self,
        target_type: SpriteType,
        collider_type: SpriteType,
        damage: i32,
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
                    s1.health -= damage;
                }
            }
        }
    }
}