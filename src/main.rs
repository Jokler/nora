use std::ffi::OsString;

use xcb::{
    change_property, configure_window, create_gc, create_pixmap, create_window, free_gc,
    free_pixmap, get_image, intern_atom, map_window, put_image, set_input_focus, CW_BACK_PIXMAP,
    IMAGE_FORMAT_Z_PIXMAP,
};

use anyhow::{anyhow, Context, Result};
use structopt::clap::AppSettings::TrailingVarArg;
use structopt::StructOpt;

// Sets all bits to 1 - mask everything
const ALL_PLANES: u32 = !0;

#[derive(StructOpt, Debug)]
#[structopt(
    global_settings = &[TrailingVarArg],)
]
struct Args {
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

    let (conn, preferred_screen) =
        xcb::base::Connection::connect(None).context("Failed to connect to X server")?;
    let screen = conn
        .get_setup()
        .roots()
        .nth(preferred_screen as usize)
        .ok_or(anyhow!("screen {} not found", preferred_screen))?;

    let window_handle = conn.generate_id();
    let (width, height) = (screen.width_in_pixels(), screen.height_in_pixels());

    // Image container
    let pixmap_handle = conn.generate_id();
    create_pixmap(
        &conn,
        screen.root_depth(),
        pixmap_handle,
        screen.root(),
        width,
        height,
    );

    let image = get_image(
        &conn,
        IMAGE_FORMAT_Z_PIXMAP as u8,
        screen.root(),
        0,
        0,
        width,
        height,
        ALL_PLANES,
    )
    .get_reply()
    .context("Failed to capture screen")?;
    let image_data = image.data();

    // Handle allows adjusting of drawing settings
    let gc_handle = conn.generate_id();
    create_gc(&conn, gc_handle, pixmap_handle, &[]);

    let max_request_length = xcb::big_requests::enable(&conn)
        .get_reply()
        .context("Failed to get maximum request length")?
        .maximum_request_length();

    // TODO Verify that this is always correct
    let stride = image_data.len() / height as usize;
    // Where does this come from?
    let req_size = 18;

    // If there is too much data it has to be split up
    if image_data.len() < max_request_length as usize {
        put_image(
            &conn,
            IMAGE_FORMAT_Z_PIXMAP as u8,
            pixmap_handle,
            gc_handle,
            width,
            height,
            0,
            0,
            0,
            image.depth(),
            image_data,
        );
    } else {
        let mut rows = (max_request_length as usize - req_size - 4) / stride;
        if rows <= 0 {
            panic!("{} rows to transmit", rows)
        };

        let mut start: usize = 0;
        let mut height = height as usize;
        let mut dst_y = 0;
        loop {
            if rows > height {
                rows = height;
            }

            let length = rows as usize * stride;

            put_image(
                &conn,
                IMAGE_FORMAT_Z_PIXMAP as u8,
                pixmap_handle,
                gc_handle,
                width,
                rows as u16,
                0,
                dst_y,
                0,
                image.depth(),
                &image_data[start..start + length],
            );

            height -= rows;
            dst_y += rows as i16;
            start += length;

            if height == 0 {
                break;
            }
        }
    }
    free_gc(&conn, gc_handle);

    // Check if the image transmission went well
    conn.has_error()
        .context("Found error after sending the image to X")?;

    // Setup window with the pixmap as a background
    let window_setup = [
        (xcb::ffi::XCB_CW_OVERRIDE_REDIRECT, 1),
        (CW_BACK_PIXMAP, pixmap_handle),
    ];

    create_window(
        &conn,
        xcb::ffi::XCB_COPY_FROM_PARENT as u8,
        window_handle,
        screen.root(),
        0,
        0,
        width,
        height,
        0,
        xcb::ffi::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16,
        xcb::ffi::XCB_WINDOW_CLASS_COPY_FROM_PARENT,
        &window_setup,
    );

    free_pixmap(&conn, pixmap_handle);

    // Setup window properties
    change_property(
        &conn,
        xcb::ffi::XCB_PROP_MODE_REPLACE as u8,
        window_handle,
        xcb::ffi::XCB_ATOM_WM_NAME,
        xcb::ffi::XCB_ATOM_STRING,
        8,
        "fullscreen-viewer".as_bytes(),
    );

    change_property(
        &conn,
        xcb::ffi::XCB_PROP_MODE_REPLACE as u8,
        window_handle,
        xcb::ffi::XCB_ATOM_WM_CLASS,
        xcb::ffi::XCB_ATOM_STRING,
        8,
        "fullscreen-viewer\0fullscreen-viewer\0".as_bytes(),
    );

    let atom = intern_atom(&conn, false, "_NET_WM_BYPASS_COMPOSITOR")
        .get_reply()
        .context("Failed to get compositor bypass atom")?
        .atom();

    change_property(
        &conn,
        xcb::ffi::XCB_PROP_MODE_REPLACE as u8,
        window_handle,
        atom,
        xcb::ffi::XCB_ATOM_CARDINAL,
        32,
        &[1],
    );

    // Make window visible
    map_window(&conn, window_handle);

    // Put window on top
    let values = [(
        xcb::ffi::XCB_CONFIG_WINDOW_STACK_MODE as u16,
        xcb::ffi::XCB_STACK_MODE_ABOVE,
    )];
    configure_window(&conn, window_handle, &values);

    // Ensure that commands have completed
    conn.flush();

    set_input_focus(
        &conn,
        xcb::ffi::XCB_INPUT_FOCUS_PARENT as u8,
        window_handle,
        xcb::ffi::XCB_CURRENT_TIME,
    );

    let executable = args.executable.remove(0);
    std::process::Command::new(executable.clone())
        .args(args.executable)
        .status()
        .with_context(|| anyhow!("Failed to execute {}", executable.to_string_lossy()))?;

    Ok(())
}
