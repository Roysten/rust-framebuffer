extern crate framebuffer;

use framebuffer::{Framebuffer, KdMode};

fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let w = framebuffer.var_screen_info.xres as usize;
    let h = framebuffer.var_screen_info.yres as usize;
    let line_length = framebuffer.fix_screen_info.line_length as usize;
    let bytespp = (framebuffer.var_screen_info.bits_per_pixel / 8) as usize;

    let mut frame = vec![0u8; line_length * h];

    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

    let half_line = w * bytespp / 2;
    frame[half_line] = 255;
    frame[half_line + 1] = 255;
    frame[half_line + 2] = 255;

    for y in 1..h {
        for x in 0..w {
            let curr_index = y * line_length + x * bytespp;
            let prev_index = curr_index - line_length;

            let val_a = if x == 0 {
                0
            } else {
                frame[prev_index - bytespp]
            };
            let val_b = if x == w - 1 {
                0
            } else {
                frame[prev_index + bytespp]
            };

            let val = if val_a > 0 { 255 } else { 0 } ^ if val_b > 0 { 255 } else { 0 };

            frame[curr_index] = val;
            frame[curr_index + 1] = val;
            frame[curr_index + 2] = val;
        }
    }

    let _ = framebuffer.write_frame(&frame);
    let _ = std::io::stdin().read_line(&mut String::new());
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}
