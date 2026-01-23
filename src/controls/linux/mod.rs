use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::mem::{size_of, MaybeUninit};
use std::os::unix::io::FromRawFd;


use super::{Key, KeyData};

#[repr(C)]
#[derive(Copy, Clone)]
struct InputEvent {
    tv_sec: i64,
    tv_usec: i64,
    type_: u16,
    code: u16,
    value: i32,
}

// Event types
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const REL_WHEEL: u16 = 0x08;

// Mouse buttons (evdev codes)
const BTN_LEFT: u16 = 272;
const BTN_RIGHT: u16 = 273;
const BTN_MIDDLE: u16 = 274;

// Minimal FFI to avoid libc crate
#[link(name = "c")]
unsafe extern "C" {
    fn open(pathname: *const u8, flags: i32) -> i32;
}

// Open flags
const O_RDONLY: i32 = 0;

pub struct Input {
    keys: HashMap<Key, KeyData>,
    cursor: (i32, i32),
    scroll_delta: i32,
    focused: bool,
    devices: Vec<File>,
}

impl Input {
    pub fn new() -> Self {
        let mut devices = Vec::new();

        // Open all /dev/input/event* devices
        for i in 0..32 {
            let path = format!("/dev/input/event{}", i);
            if let Some(fd) = open_device(&path) {
                // Safe: File takes ownership of fd
                unsafe {
                    devices.push(File::from_raw_fd(fd));
                }
            }
        }

        Self {
            keys: HashMap::new(),
            cursor: (0, 0),
            scroll_delta: 0,
            focused: true,
            devices,
        }
    }

    pub fn poll(&mut self, focused: bool) {
        self.focused = focused;

        if !focused {
            self.keys.clear();
            return;
        }

        self.update();
    }

    pub fn update(&mut self) {
        self.scroll_delta = 0;

        // Read all events first
        let mut events = Vec::new();
        for dev in &mut self.devices {
            while let Some(ev) = read_event(dev) {
                events.push(ev);
            }
        }

        // Then handle them after `self.devices` borrow ends
        for ev in events {
            self.handle_event(ev);
        }
    }


    fn handle_event(&mut self, ev: InputEvent) {
        match ev.type_ {
            EV_KEY => {
                if let Some(key) = map_evdev_key(ev.code) {
                    let is_down = ev.value != 0;
                    self.update_key(key, is_down);
                }
            }
            EV_REL if ev.code == REL_WHEEL => {
                self.scroll_delta += ev.value;
            }
            _ => {}
        }
    }

    pub fn update_key(&mut self, key: Key, is_down: bool) {
        self.keys.entry(key).or_insert_with(KeyData::new).update(is_down);
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
        // No compositor-independent way; return fallback
        self.cursor
    }

    pub fn scroll_delta(&self) -> i32 {
        self.scroll_delta
    }

    pub fn end_frame(&mut self) {
        self.scroll_delta = 0;
    }
}

// Helper: map evdev codes to your Key enum
fn map_evdev_key(code: u16) -> Option<Key> {
    Some(match code {
        BTN_LEFT => Key::MouseLeft,
        BTN_RIGHT => Key::MouseRight,
        BTN_MIDDLE => Key::MouseMiddle,
        _ => return None,
    })
}

// Read one InputEvent from a device
fn read_event(dev: &mut File) -> Option<InputEvent> {
    let mut ev = MaybeUninit::<InputEvent>::uninit();
    let buf = unsafe {
        std::slice::from_raw_parts_mut(ev.as_mut_ptr() as *mut u8, size_of::<InputEvent>())
    };
    match dev.read(buf) {
        Ok(n) if n == size_of::<InputEvent>() => Some(unsafe { ev.assume_init() }),
        _ => None,
    }
}

// Open a device path using extern "C" open
fn open_device(path: &str) -> Option<i32> {
    let c_path = std::ffi::CString::new(path).ok()?;
    let fd = unsafe { open(c_path.as_ptr() as *const u8, O_RDONLY) };
    if fd >= 0 { Some(fd) } else { None }
}

