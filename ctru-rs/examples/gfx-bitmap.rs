use ctru::prelude::*;
use ctru::services::gfx::{Flush, Screen, Swap};

/// Ferris image taken from <https://rustacean.net> and scaled down to 320x240px.
/// To regenerate the data, you will need to install `imagemagick` and run this
/// command from the `examples` directory:
///
/// ```sh
/// magick assets/ferris.png -channel-fx "red<=>blue" -rotate 90 assets/ferris.rgb
/// ```
///
/// This creates an image appropriate for the default frame buffer format of
/// [`Bgr8`](ctru::services::gspgpu::FramebufferFormat::Bgr8)
/// and rotates the image 90° to account for the portrait mode screen.
static IMAGE: &[u8] = include_bytes!("assets/ferris.rgb");

fn main() {
    ctru::use_panic_handler();

    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    println!("\x1b[21;4HPress Start to exit, or A to flip the image.");

    let mut bottom_screen = gfx.bottom_screen.borrow_mut();

    // We don't need double buffering in this example.
    // In this way we can draw our image only once on screen.
    bottom_screen.set_double_buffering(false);
    // Swapping buffers commits the change from the line above.
    bottom_screen.swap_buffers();

    // 3 bytes per pixel, we just want to reverse the pixels but not individual bytes
    let flipped_image: Vec<_> = IMAGE.chunks(3).rev().flatten().copied().collect();

    let mut image_bytes = IMAGE;

    // Main loop
    while apt.main_loop() {
        //Scan all the inputs. This should be done once for each frame
        hid.scan_input();

        if hid.keys_down().contains(KeyPad::START) {
            break;
        }

        // We assume the image is the correct size already, so we drop width + height.
        let frame_buffer = bottom_screen.raw_framebuffer();

        if hid.keys_down().contains(KeyPad::A) {
            image_bytes = if std::ptr::eq(image_bytes, IMAGE) {
                &flipped_image[..]
            } else {
                IMAGE
            };
        }

        // this copies more than necessary (once per frame) but it's fine...
        unsafe {
            frame_buffer
                .ptr
                .copy_from(image_bytes.as_ptr(), image_bytes.len());
        }

        // Flush framebuffers. Since we're not using double buffering,
        // this will render the pixels immediately
        bottom_screen.flush_buffers();

        //Wait for VBlank
        gfx.wait_for_vblank();
    }
}
