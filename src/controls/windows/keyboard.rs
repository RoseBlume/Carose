use std::collections::HashMap;
use std::{thread, time::Duration};
use super::GetAsyncKeyState;
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Key {
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

    // Arrow Keys
    Up,
    Down,
    Left,
    Right,

    F(u8), // F1–F12
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



fn vk_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) & 0x8000u16 as i16) != 0 }
}

pub struct Keyboard {
    keys: HashMap<Key, KeyData>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Call once per frame
    pub fn update(&mut self) {
        // Letters A–Z
        for vk in 0x41..=0x5A {
            let c = (vk as u8 as char).to_ascii_lowercase();
            self.update_key(Key::Char(c), vk_down(vk));
        }

        // Numbers 0–9
        for vk in 0x30..=0x39 {
            self.update_key(Key::Num((vk - 0x30) as u8), vk_down(vk));
        }

        // Function keys F1–F12
        for i in 0..12 {
            self.update_key(Key::F(i + 1), vk_down((0x70 + i).into()));
        }

        // Control keys
        self.update_key(Key::Space, vk_down(0x20));
        self.update_key(Key::Backspace, vk_down(0x08));
        self.update_key(Key::Tab, vk_down(0x09));
        self.update_key(Key::Enter, vk_down(0x0D));
        self.update_key(Key::Escape, vk_down(0x1B));

        self.update_key(Key::LeftShift, vk_down(0xA0));
        self.update_key(Key::RightShift, vk_down(0xA1));
        self.update_key(Key::LeftCtrl, vk_down(0xA2));
        self.update_key(Key::RightCtrl, vk_down(0xA3));

        self.update_key(Key::Up, vk_down(0x26));
        self.update_key(Key::Down, vk_down(0x28));
        self.update_key(Key::Left, vk_down(0x25));
        self.update_key(Key::Right, vk_down(0x27));
        thread::sleep(Duration::from_millis(10));
    }

    fn update_key(&mut self, key: Key, is_down: bool) {
        self.keys
            .entry(key)
            .or_insert_with(KeyData::new)
            .update(is_down);
    }

    pub fn pressed(&self, key: Key) -> bool {
        self.keys.get(&key).map_or(false, |k| k.pressed())
    }

    pub fn released(&self, key: Key) -> bool {
        self.keys.get(&key).map_or(true, |k| k.released())
    }

    pub fn clicked(&mut self, key: Key) -> bool {
        self.keys
            .get_mut(&key)
            .map_or(false, |k| k.clicked())
    }
}
