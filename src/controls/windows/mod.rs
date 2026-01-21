pub mod keyboard;
pub mod mouse;


#[link(name = "user32")]
unsafe extern "system" {
    pub fn GetAsyncKeyState(vkey: i32) -> i16;
    pub fn GetCursorPos(point: *mut POINT) -> i32;
    pub fn PeekMessageW(
        msg: *mut MSG,
        hwnd: isize,
        min: u32,
        max: u32,
        remove: u32,
    ) -> i32;
}
#[repr(C)]
pub struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
pub struct MSG {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: POINT,
}
