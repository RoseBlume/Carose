//! # carose
//!
//! `carose` is a lightweight 2D game framework focused on simplicity,
//! immediate-mode rendering, and minimal boilerplate.
//!
//! It provides a pixel-based window, sprite system, basic animation,
//! text rendering, audio playback, menus, and input handling — all designed
//! to work together in a straightforward game loop.
//!
//! ## Design Goals
//!
//! - Simple, explicit game loops
//! - Immediate-mode rendering (draw every frame)
//! - Minimal abstractions
//! - No ECS requirement
//! - Easy sprite and animation handling
//! - Built-in menus, audio, and controls
//!
//! ## Core Concepts
//!
//! ### Window
//!
//! [`Window`] is the central object of the engine. It owns:
//! - The render buffer
//! - All sprites
//! - All on-screen text
//! - Input state
//!
//! Most interaction happens through methods on `Window`.
//!
//! ### Sprites
//!
//! Sprites are stored internally by the window and can be:
//! - Solid color rectangles
//! - Bitmap sprites
//! - Animated bitmap sprites
//!
//! Sprites support:
//! - Position and size
//! - Health
//! - Collision checks
//! - Velocity and acceleration vectors
//!
//! ### Text
//!
//! Text is rendered using a built-in 5×5 bitmap font and can be aligned
//! automatically or manually using [`TextAlign`].
//!
//! ### Menus
//!
//! [`Menu`] provides a simple vertical menu system with keyboard navigation
//! and centered rendering, intended for pause menus, main menus, and game-over
//! screens.
//!
//! ### Audio
//!
//! The audio module provides:
//! - Fire-and-forget sound effects via [`crate::audio::Audio`]
//! - Looping background music and playlists via [`crate::audio::Audio`]
//!
//! ### Controls
//!
//! The controls system tracks keyboard and mouse input with support for:
//! - Pressed (held)
//! - Released
//! - Clicked (press → release)
//!
//! Input is frame-based and updated manually each loop.
//!
//! ## Typical Usage
//!
//! A typical game loop looks like:
//!
//! 1. Poll input
//! 2. Update game state
//! 3. Handle collisions and logic
//! 4. Draw the frame
//!
//! ```no_run
//! use carose::{Window, Menu, TextAlign};
//!
//! let mut window = Window::new("My Game", 800, 600);
//!
//! while window.is_open() {
//!     window.update_controls();
//!     // game logic
//!     window.draw();
//! }
//! ```
//!
//! ## Modules
//!
//! - [`windows`] — Window creation, rendering, text, sprites
//! - [`sprites`] — Sprite types, rendering, animation, physics vectors
//! - [`controls`] — Keyboard and mouse input handling
// ! - [`menu`] — Menu UI utilities
//! - [`audio`] — Sound effects and background music
//! - [`image`] — Bitmap and sprite sheet loading helpers
//! - [`colors`] — Common color constants
//!
//! This crate is intended for small to mid-sized 2D games,
//! prototypes, and learning projects.


pub mod windows;
pub mod menu;
pub mod image;
pub mod colors;
pub mod sprites;
pub mod audio;
pub mod controls;
pub use windows::Window;
pub use menu::Menu;
pub use windows::text::TextAlign;


