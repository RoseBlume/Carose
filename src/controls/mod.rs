#[cfg(target_os = "windows")]
#[cfg_attr(target_os = "windows", path = "windows/mod.rs")]
mod os;

#[cfg(target_os = "linux")]
#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
mod os;

pub use os::{Input};


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    // Keyboard
    Char(char),
    Num(u8),

    Backspace,
    Enter,
    Tab,
    Escape,

    LeftCtrl,
    RightCtrl,
    LeftShift,
    RightShift,
    Space,

    Up,
    Down,
    Left,
    Right,

    F(u8),

    // Mouse
    MouseLeft,
    MouseRight,
    MouseMiddle,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Copy, Clone)]
struct KeyData {
    state: KeyState,
    was_pressed: bool,
}

impl KeyData {
    fn new() -> Self {
        Self {
            state: KeyState::Released,
            was_pressed: false,
        }
    }

    fn update(&mut self, is_down: bool) {
        match (self.state, is_down) {
            (KeyState::Released, true) => {
                self.state = KeyState::Pressed;
                self.was_pressed = true;
            }
            (KeyState::Pressed, false) => {
                self.state = KeyState::Released;
            }
            _ => {}
        }
    }

    fn pressed(&self) -> bool {
        self.state == KeyState::Pressed
    }

    fn released(&self) -> bool {
        self.state == KeyState::Released
    }

    fn clicked(&mut self) -> bool {
        if self.was_pressed && self.state == KeyState::Released {
            self.was_pressed = false;
            true
        } else {
            false
        }
    }
}
