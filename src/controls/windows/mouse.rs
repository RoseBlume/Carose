
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::time::Duration;
use std::thread;
use super::{GetAsyncKeyState, GetCursorPos, PeekMessageW, POINT, MSG};
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ButtonState {
    Pressed,
    Released,
}

#[derive(Debug, Copy, Clone)]
struct ButtonData {
    state: ButtonState,
    was_pressed: bool,
}

impl ButtonData {
    fn new() -> Self {
        Self {
            state: ButtonState::Released,
            was_pressed: false,
        }
    }

    fn update(&mut self, is_down: bool) {
        thread::sleep(Duration::from_millis(10));
        match (self.state, is_down) {
            (ButtonState::Released, true) => {
                self.state = ButtonState::Pressed;
                self.was_pressed = true;
            }
            (ButtonState::Pressed, false) => {
                self.state = ButtonState::Released;
            }
            _ => {}
        }

    }

    fn pressed(&self) -> bool {
        self.state == ButtonState::Pressed
    }

    fn released(&self) -> bool {
        self.state == ButtonState::Released
    }

    fn clicked(&mut self) -> bool {
        if self.was_pressed && self.state == ButtonState::Released {
            self.was_pressed = false;
            true
        } else {
            false
        }
    }
}



const WM_MOUSEWHEEL: u32 = 0x020A;
const PM_REMOVE: u32 = 0x0001;



fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) & 0x8000u16 as i16) != 0 }
}

pub struct Mouse {
    buttons: HashMap<MouseButton, ButtonData>,
    cursor: (i32, i32),
    scroll_delta: i32,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            buttons: HashMap::new(),
            cursor: (0, 0),
            scroll_delta: 0,
        }
    }

    pub fn update(&mut self) {
        // Buttons
        self.update_button(MouseButton::Left, key_down(0x01));
        self.update_button(MouseButton::Right, key_down(0x02));
        self.update_button(MouseButton::Middle, key_down(0x04));

        // Cursor position
        unsafe {
            let mut pt = MaybeUninit::<POINT>::zeroed();
            if GetCursorPos(pt.as_mut_ptr()) != 0 {
                let pt = pt.assume_init();
                self.cursor = (pt.x, pt.y);
            }
        }

        // Scroll wheel (message-based)
        unsafe {
            let mut msg = MaybeUninit::<MSG>::zeroed();
            while PeekMessageW(msg.as_mut_ptr(), 0, WM_MOUSEWHEEL, WM_MOUSEWHEEL, PM_REMOVE) != 0 {
                let msg = msg.assume_init_read();
                let delta = ((msg.wparam >> 16) & 0xFFFF) as i16;
                self.scroll_delta += delta as i32;
            }
        }
    }

    fn update_button(&mut self, button: MouseButton, is_down: bool) {
        self.buttons
            .entry(button)
            .or_insert_with(ButtonData::new)
            .update(is_down);
    }

    pub fn pressed(&self, button: MouseButton) -> bool {
        self.buttons.get(&button).map_or(false, |b| b.pressed())
    }

    pub fn released(&self, button: MouseButton) -> bool {
        self.buttons.get(&button).map_or(true, |b| b.released())
    }

    pub fn clicked(&mut self, button: MouseButton) -> bool {
        self.buttons
            .get_mut(&button)
            .map_or(false, |b| b.clicked())
    }

    pub fn position(&self) -> (i32, i32) {
        self.cursor
    }

    pub fn scroll_up(&self) -> bool {
        self.scroll_delta > 0
    }

    pub fn scroll_down(&self) -> bool {
        self.scroll_delta < 0
    }
    pub fn scroll_delta(&self) -> i32 {
        self.scroll_delta
    }

    pub fn end_frame(&mut self) {
        self.scroll_delta = 0;
    }
}
