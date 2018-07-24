extern crate bmp;
extern crate framebuffer;

use framebuffer::{Framebuffer, KdMode};

fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let w = framebuffer.var_screen_info.xres;
    let h = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;

    let mut frame = vec![0u8; (line_length * h) as usize];
    let img = bmp::open("examples/rust-logo/rust-logo.bmp").unwrap();

    //Disable text mode in current tty
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

    for offset in 0..w - img.get_width() {
        for (x, y) in img.coordinates() {
            let px = img.get_pixel(x, y);
            let start_index = (y * line_length + (offset + x) * bytespp) as usize;
            frame[start_index] = px.b;
            frame[start_index + 1] = px.g;
            frame[start_index + 2] = px.r;
        }

        let _ = framebuffer.write_frame(&frame);
    }

    //Reenable text mode in current tty
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}
