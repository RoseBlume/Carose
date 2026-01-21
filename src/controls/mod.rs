#[cfg_attr(target_os = "windows", path = "windows/mod.rs")]
mod os;

pub use os::{
    keyboard::{Key, Keyboard},
    mouse::{MouseButton, Mouse}
};