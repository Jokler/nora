// FIXME temporary file

use std::ffi::CString;

use anyhow::{bail, Result};
use libc::{c_char, c_short, c_ulong, c_ushort, c_void};

pub struct Display {
    ptr: *mut CDisplay,
    has_xfixes: bool,
}

impl Display {
    pub fn open(name: Option<&str>) -> Self {
        let name_ptr = if let Some(name) = name.map(|s| CString::new(s).unwrap()) {
            name.as_ptr()
        } else {
            std::ptr::null()
        };

        let ptr = unsafe { XOpenDisplay(name_ptr) };

        assert!(!ptr.is_null());

        let c_true = 1;
        let has_xfixes = unsafe { XFixesQueryVersion(ptr, &mut 2, &mut 0) } == c_true;

        Display { ptr, has_xfixes }
    }

    pub fn get_cursor_image(&self) -> Result<CursorImage> {
        if !self.has_xfixes {
            bail!("xfixes version is too old");
        }

        let ptr = unsafe { XFixesGetCursorImage(self.ptr) };
        assert!(!ptr.is_null());

        Ok(CursorImage { ptr })
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}

pub struct CursorImage {
    ptr: *mut CCursorImage,
}

impl CursorImage {
    pub fn x(&self) -> i16 {
        unsafe { &*self.ptr }.x
    }

    pub fn y(&self) -> i16 {
        unsafe { &*self.ptr }.y
    }

    pub fn xhot(&self) -> u16 {
        unsafe { &*self.ptr }.xhot
    }

    pub fn yhot(&self) -> u16 {
        unsafe { &*self.ptr }.yhot
    }

    pub fn width(&self) -> u16 {
        unsafe { &*self.ptr }.width
    }

    pub fn height(&self) -> u16 {
        unsafe { &*self.ptr }.height
    }

    pub fn pixels(&self) -> &[CPixel] {
        let size = self.width() * self.height();
        let ptr = unsafe { &*self.ptr }.pixels;
        assert!(!ptr.is_null());

        unsafe { std::slice::from_raw_parts(ptr, size as usize) }
    }
}

impl Drop for CursorImage {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}

enum CDisplay {}

#[repr(C)]
pub struct CPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    padding: [u8; 4],
}

#[repr(C)]
struct CCursorImage {
    x: c_short,
    y: c_short,
    width: c_ushort,
    height: c_ushort,
    xhot: c_ushort,
    yhot: c_ushort,
    serial: c_ulong,
    pixels: *const CPixel,
    atom: c_ulong,
    name: *const c_char,
}

extern "C" {
    fn XOpenDisplay(display_name: *const i8) -> *mut CDisplay;
    fn XFixesQueryVersion(display: *mut CDisplay, major_v: *mut i32, minor_v: *const i32) -> i32;
    fn XFixesGetCursorImage(display: *mut CDisplay) -> *mut CCursorImage;
}
