use std::ffi::OsString;

use x11::xlib::{
    CurrentTime, PropModeReplace, RevertToParent, ZPixmap, XA_CARDINAL, XA_STRING, XA_WM_CLASS,
    XA_WM_NAME,
};

use anyhow::{anyhow, Context, Result};
use structopt::clap::AppSettings::TrailingVarArg;
use structopt::StructOpt;

mod ffi;
use ffi::Display;

// Sets all bits to 1 - mask everything
const ALL_PLANES: u32 = !0;

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings = &[TrailingVarArg],)
]
struct Args {
    #[structopt(short, long)]
    /// Add the cursor to the frozen image
    show_cursor: bool,
    #[structopt(required = true)]
    /// Executable with arguments to run
    executable: Vec<OsString>,
}

fn main() {
    if let Err(e) = run() {
        eprint!("ERROR: {}", e);
        e.chain()
            .skip(1)
            .for_each(|cause| eprint!(": {}", cause));
        eprintln!();
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = Args::from_args();

    let display = Display::open(None);
    let screen = display.screen(0);
    let root = screen.root;
    let (width, height) = (screen.width, screen.height);

    let pixmap_handle =
        display.create_pixmap(root, width as u32, height as u32, screen.root_depth as u32);

    let mut image = display.get_image(root, 0, 0, width, height, ALL_PLANES, ZPixmap);

    let len = (image.width * image.height * image.bits_per_pixel / 8) as usize;
    let image_data = unsafe { std::slice::from_raw_parts_mut(image.data as *mut _, len) };

    // Blend cursor onto the image
    if args.show_cursor {
        let cursor = display
            .get_cursor_image()
            .context("Failed to get cursor image")?;
        let pixels = cursor.pixels();
        let cursorx = cursor.x() as usize - cursor.xhot() as usize;
        let cursory = cursor.y() as usize - cursor.yhot() as usize;

        for x in 0.max(cursorx)..(width as usize).min(cursorx + cursor.width() as usize) {
            for y in 0.max(cursory)..(height as usize).min(cursory + cursor.height() as usize) {
                let cx: usize = x - cursorx;
                let cy: usize = y - cursory;

                let istart = (y * width as usize + x) as usize * 4;
                let cstart = cy * cursor.width() as usize + cx;

                let alpha = pixels[cstart].a as f32 / 255.0;

                let old_b = image_data[istart] as f32 * (1.0 - alpha);
                let old_g = image_data[istart + 1] as f32 * (1.0 - alpha);
                let old_r = image_data[istart + 2] as f32 * (1.0 - alpha);

                image_data[istart] = (old_b + pixels[cstart].b as f32 * alpha) as u8;
                image_data[istart + 1] = (old_g + pixels[cstart].g as f32 * alpha) as u8;
                image_data[istart + 2] = (old_r + pixels[cstart].r as f32 * alpha) as u8;
            }
        }
    }

    let gc_handle = display.create_gc(pixmap_handle);

    display.put_image(
        pixmap_handle,
        gc_handle,
        &mut image,
        0,
        0,
        0,
        0,
        width as u32,
        height as u32,
    );

    let window_handle = display.create_window(
        root as i32,
        0,
        0,
        width,
        height,
        0,
        screen.root_depth,
        x11::xlib::InputOutput,
        pixmap_handle,
    );

    display.free_gc(gc_handle);
    display.free_pixmap(pixmap_handle);

    // Setup window properties
    display.change_property(
        window_handle,
        XA_WM_NAME,
        XA_STRING,
        PropModeReplace,
        "fullscreen-viewer".as_bytes(),
    );

    display.change_property(
        window_handle,
        XA_WM_CLASS,
        XA_STRING,
        PropModeReplace,
        "fullscreen-viewer\0fullscreen-viewer\0".as_bytes(),
    );

    let atom = display
        .intern_atom("_NET_WM_BYPASS_COMPOSITOR", false)
        .context("Failed to get compositor bypass atom")?;

    display.change_property(window_handle, atom, XA_CARDINAL, PropModeReplace, &[1]);

    // Make window visible
    display.map_window(window_handle);

    // Put window on top
    display.set_stack_mode(window_handle, x11::xlib::Above);

    // Ensure that commands have completed
    //conn.flush();
    display.sync(false);

    display.set_input_focus(window_handle, RevertToParent, CurrentTime);

    let executable = args.executable.remove(0);
    std::process::Command::new(executable.clone())
        .args(args.executable)
        .status()
        .with_context(|| anyhow!("Failed to execute {}", executable.to_string_lossy()))?;

    Ok(())
}
