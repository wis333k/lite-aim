// win_mouse: Win32 raw cursor FFI (PORT NGUYEN) — FPS mouse-look chuan
#![allow(dead_code)]
use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

extern "system" {
    fn GetCursorPos(lp_point: *mut Point) -> i32;
    fn SetCursorPos(x: i32, y: i32) -> i32;
    fn GetForegroundWindow() -> *mut c_void;
    fn GetClientRect(hwnd: *mut c_void, rect: *mut Rect) -> i32;
    fn ClientToScreen(hwnd: *mut c_void, point: *mut Point) -> i32;
    fn ShowCursor(show: i32) -> i32;
}

pub fn cursor_pos() -> (i32, i32) {
    let mut p = Point { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut p) };
    (p.x, p.y)
}

pub fn set_cursor(x: i32, y: i32) {
    unsafe { SetCursorPos(x, y) };
}

pub fn foreground() -> usize {
    unsafe { GetForegroundWindow() as usize }
}

// tam vung client (khong tinh title bar) — anchor mouse-look
pub fn client_center(hwnd: usize) -> Option<(i32, i32)> {
    unsafe {
        let h = hwnd as *mut c_void;
        if h.is_null() {
            return None;
        }
        let mut r = Rect { left: 0, top: 0, right: 0, bottom: 0 };
        if GetClientRect(h, &mut r) == 0 {
            return None;
        }
        let mut p = Point { x: 0, y: 0 };
        if ClientToScreen(h, &mut p) == 0 {
            return None;
        }
        let w = r.right - r.left;
        let hh = r.bottom - r.top;
        if w <= 0 || hh <= 0 {
            return None;
        }
        Some((p.x + w / 2, p.y + hh / 2))
    }
}

pub fn show_cursor(show: bool) {
    unsafe { ShowCursor(if show { 1 } else { 0 }) };
}
