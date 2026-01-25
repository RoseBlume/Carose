#[cfg(target_os = "windows")]
#[cfg_attr(target_os = "windows", path = "windows/mod.rs")]
mod os;

#[cfg(target_os = "linux")]
#[cfg_attr(target_os = "linux", path = "linux/mod.rs")]
mod os;

pub use os::{Input};

/// Represents an abstract input key or button.
///
/// `Key` unifies keyboard and mouse inputs into a single type so input
/// handling can be device-agnostic. This allows keyboard keys and mouse
/// buttons to be queried through the same API.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    // Keyboard

    /// Alphanumeric character key (case-insensitive).
    Char(char),

    /// Numeric key (0–9).
    Num(u8),

    /// Backspace key.
    Backspace,

    /// Enter / Return key.
    Enter,

    /// Tab key.
    Tab,

    /// Escape key.
    Escape,

    /// Left Control key.
    LeftCtrl,

    /// Right Control key.
    RightCtrl,

    /// Left Shift key.
    LeftShift,

    /// Right Shift key.
    RightShift,

    /// Spacebar.
    Space,

    /// Up arrow key.
    Up,

    /// Down arrow key.
    Down,

    /// Left arrow key.
    Left,

    /// Right arrow key.
    Right,

    /// Function key (F1–F12).
    F(u8),

    // Mouse

    /// Left mouse button.
    MouseLeft,

    /// Right mouse button.
    MouseRight,

    /// Middle mouse button.
    MouseMiddle,
}

/// Internal key state representation.
///
/// Tracks whether a key is currently pressed or released.
/// This is not exposed publicly; it is used internally by the input system.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum KeyState {
    /// The key is currently held down.
    Pressed,

    /// The key is not pressed.
    Released,
}

/// Internal per-key state data.
///
/// Stores both the current state and whether the key has been
/// pressed since the last time it was released. This enables
/// edge-triggered input such as clicks.
#[derive(Debug, Copy, Clone)]
struct KeyData {
    /// Current physical state of the key.
    state: KeyState,

    /// Whether the key was pressed since the last release.
    was_pressed: bool,
}

impl KeyData {
    /// Creates a new key state initialized as released.
    fn new() -> Self {
        Self {
            state: KeyState::Released,
            was_pressed: false,
        }
    }

    /// Updates the key state based on whether the key is currently down.
    ///
    /// This method should be called once per frame during input polling.
    /// It handles transitions between pressed and released states and
    /// tracks press events for edge detection.
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

    /// Returns `true` if the key is currently held down.
    fn pressed(&self) -> bool {
        self.state == KeyState::Pressed
    }

    /// Returns `true` if the key is currently released.
    fn released(&self) -> bool {
        self.state == KeyState::Released
    }

    /// Returns `true` once when the key is clicked.
    ///
    /// A click is defined as a press followed by a release.
    /// This method returns `true` only once per click and
    /// resets automatically after being read.
    fn clicked(&mut self) -> bool {
        if self.was_pressed && self.state == KeyState::Released {
            self.was_pressed = false;
            true
        } else {
            false
        }
    }
}
