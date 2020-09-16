use std::ffi::CString;
use std::ops::Deref;

use anyhow::{bail, Result};
use libc::c_void;

use x11::xfixes::XFixesCursorImage;
use x11::xlib::False as XFalse;
use x11::xlib::True as XTrue;
use x11::xlib::{
    Atom, CWBackPixmap, CWOverrideRedirect, Drawable, Pixmap, Time, Window, XChangeProperty,
    XConfigureWindow, XCreateGC, XCreatePixmap, XCreateWindow, XFreeGC, XFreePixmap, XGetImage,
    XImage, XInternAtom, XMapWindow, XPutImage, XScreenOfDisplay, XSetInputFocus,
    XSetWindowAttributes, XSync, GC,
};

pub struct Display {
    ptr: *mut x11::xlib::_XDisplay,
    has_xfixes: bool,
}

impl Display {
    pub fn open(name: Option<&str>) -> Self {
        let name_ptr = if let Some(name) = name.map(|s| CString::new(s).unwrap()) {
            name.as_ptr()
        } else {
            std::ptr::null()
        };

        let ptr = unsafe { x11::xlib::XOpenDisplay(name_ptr) };

        assert!(!ptr.is_null());

        let has_xfixes = unsafe { x11::xfixes::XFixesQueryVersion(ptr, &mut 2, &mut 0) } == XTrue;

        Display { ptr, has_xfixes }
    }

    pub fn create_gc(&self, drawable: u64) -> GC {
        unsafe { XCreateGC(self.ptr, drawable, 0, (&mut []).as_mut_ptr()) }
    }

    pub fn create_pixmap(&self, drawable: u64, width: u32, height: u32, depth: u32) -> Pixmap {
        unsafe { XCreatePixmap(self.ptr, drawable, width, height, depth) }
    }

    pub fn create_window(
        &self,
        parent: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        border_width: i32,
        depth: i32,
        class: i32,
        pixmap: Pixmap,
    ) -> Window {
        let mut attributes = XSetWindowAttributes {
            background_pixmap: pixmap,
            background_pixel: 0,
            border_pixmap: 0,
            border_pixel: 0,
            bit_gravity: 0,
            win_gravity: 0,
            backing_store: 0,
            backing_planes: 0,
            backing_pixel: 0,
            save_under: 0,
            event_mask: 0,
            do_not_propagate_mask: 0,
            override_redirect: XTrue,
            colormap: 0,
            cursor: 0,
        };

        unsafe {
            XCreateWindow(
                self.ptr,
                parent as u64,
                x,
                y,
                width as u32,
                height as u32,
                border_width as u32,
                depth,
                class as u32,
                std::ptr::null_mut(),
                CWOverrideRedirect | CWBackPixmap,
                &mut attributes,
            )
        }
    }

    pub fn map_window(&self, w: Window) {
        unsafe { XMapWindow(self.ptr, w) };
    }

    pub fn set_stack_mode(&self, w: Window, mode: i32) {
        let mut changes = x11::xlib::XWindowChanges {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            border_width: 0,
            sibling: 0,
            stack_mode: mode,
        };

        unsafe { XConfigureWindow(self.ptr, w, x11::xlib::CWStackMode as u32, &mut changes) };
    }

    pub fn intern_atom(&self, atom_name: &str, only_if_exists: bool) -> Result<Atom> {
        let only_if_exists = if only_if_exists { XTrue } else { XFalse };
        let atom_name = CString::new(atom_name)?;

        Ok(unsafe { XInternAtom(self.ptr, atom_name.as_ptr(), only_if_exists) })
    }

    pub fn change_property<T>(
        &self,
        window: Window,
        property: Atom,
        typ: Atom,
        mode: i32,
        data: &[T],
    ) {
        let format = std::mem::size_of::<T>() * 8;
        let data_len = data.len();
        let data_ptr = data.as_ptr();
        unsafe {
            XChangeProperty(
                self.ptr,
                window,
                property,
                typ,
                format as i32,
                mode,
                data_ptr as *const _,
                data_len as i32,
            )
        };
    }

    pub fn set_input_focus(&self, focus: Window, revert_to: i32, time: Time) {
        unsafe { XSetInputFocus(self.ptr, focus, revert_to, time) };
    }

    pub fn sync(&self, discard: bool) {
        let discard = if discard { XTrue } else { XFalse };

        unsafe { XSync(self.ptr, discard) };
    }

    pub fn screen(&self, id: i32) -> Screen {
        let ptr = unsafe { XScreenOfDisplay(self.ptr, id) };
        assert!(!ptr.is_null());

        Screen { ptr }
    }

    pub fn get_image(
        &self,
        drawable: u64,
        x: i16,
        y: i16,
        width: i32,
        height: i32,
        mask: u32,
        format: i32,
    ) -> Image {
        let ptr = unsafe {
            XGetImage(
                self.ptr,
                drawable as u64,
                x as i32,
                y as i32,
                width as u32,
                height as u32,
                mask as u64,
                format,
            )
        };
        assert!(!ptr.is_null());

        Image { ptr }
    }

    pub fn put_image(
        &self,
        drawable: Drawable,
        gc: GC,
        image: &mut Image,
        src_x: i32,
        src_y: i32,
        dest_x: i32,
        dest_y: i32,
        width: u32,
        height: u32,
    ) {
        unsafe {
            XPutImage(
                self.ptr, drawable, gc, image.ptr, src_x, src_y, dest_x, dest_y, width, height,
            )
        };
    }

    pub fn get_cursor_image(&self) -> Result<CursorImage> {
        if !self.has_xfixes {
            bail!("xfixes version is too old");
        }

        let ptr = unsafe { x11::xfixes::XFixesGetCursorImage(self.ptr) };
        assert!(!ptr.is_null());

        Ok(CursorImage { ptr })
    }

    pub fn free_gc(&self, gc: GC) {
        unsafe { XFreeGC(self.ptr, gc) };
    }

    pub fn free_pixmap(&self, pixmap: Pixmap) {
        unsafe { XFreePixmap(self.ptr, pixmap) };
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
    ptr: *mut XFixesCursorImage,
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

        unsafe { std::slice::from_raw_parts(ptr as *mut _, size as usize) }
    }
}

impl Drop for CursorImage {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}

#[repr(C)]
pub struct CPixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    padding: [u8; 4],
}

pub struct Screen {
    ptr: *mut x11::xlib::Screen,
}

impl Deref for Screen {
    type Target = x11::xlib::Screen;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}

pub struct Image {
    ptr: *mut XImage,
}

impl Deref for Image {
    type Target = XImage;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.ptr as *mut c_void);
        }
    }
}
