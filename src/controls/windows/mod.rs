use std::collections::HashMap;
use std::mem::MaybeUninit;
use super::{Key, KeyData};
const WM_MOUSEWHEEL: u32 = 0x020A;
const PM_REMOVE: u32 = 0x0001;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetAsyncKeyState(vkey: i32) -> i16;
    fn GetCursorPos(point: *mut POINT) -> i32;
    fn PeekMessageW(
        msg: *mut MSG,
        hwnd: isize,
        min: u32,
        max: u32,
        remove: u32,
    ) -> i32;
}
#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct MSG {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: POINT,
}


fn vk_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) & 0x8000u16 as i16) != 0 }
}

pub struct Input {
    keys: HashMap<Key, KeyData>,
    cursor: (i32, i32),
    scroll_delta: i32,
    focused: bool
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
            cursor: (0, 0),
            scroll_delta: 0,
            focused: false
        }
    }
    pub fn poll(&mut self, focused: bool) {
        self.focused = focused;

        if !focused {
            self.keys.clear();
            return;
        }

        self.update(); // keyboard + mouse + cursor + wheel
    }

    pub fn update(&mut self) {
        // -------- Keyboard --------

        for vk in 0x41..=0x5A {
            let c = (vk as u8 as char).to_ascii_lowercase();
            self.update_key(Key::Char(c), vk_down(vk));
        }

        for vk in 0x30..=0x39 {
            self.update_key(Key::Num((vk - 0x30) as u8), vk_down(vk));
        }

        for i in 0..12 {
            self.update_key(Key::F(i + 1), vk_down((0x70 + i).into()));
        }

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

        // -------- Mouse Buttons --------

        self.update_key(Key::MouseLeft, vk_down(0x01));
        self.update_key(Key::MouseRight, vk_down(0x02));
        self.update_key(Key::MouseMiddle, vk_down(0x04));

        // -------- Cursor Position --------

        unsafe {
            let mut pt = MaybeUninit::<POINT>::zeroed();
            if GetCursorPos(pt.as_mut_ptr()) != 0 {
                let pt = pt.assume_init();
                self.cursor = (pt.x, pt.y);
            }
        }

        // -------- Scroll Wheel --------

        unsafe {
            let mut msg = MaybeUninit::<MSG>::zeroed();
            while PeekMessageW(
                msg.as_mut_ptr(),
                0,
                WM_MOUSEWHEEL,
                WM_MOUSEWHEEL,
                PM_REMOVE,
            ) != 0
            {
                let msg = msg.assume_init_read();
                let delta = ((msg.wparam >> 16) & 0xFFFF) as i16;
                self.scroll_delta += delta as i32;
            }
        }
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
        self.keys.get_mut(&key).map_or(false, |k| k.clicked())
    }

    pub fn cursor_position(&self) -> (i32, i32) {
        self.cursor
    }

    pub fn scroll_delta(&self) -> i32 {
        self.scroll_delta
    }

    pub fn end_frame(&mut self) {
        self.scroll_delta = 0;
    }
}

