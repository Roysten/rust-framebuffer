// this demo reads the firmware image and its offset from the bgrt acpi table and shows it in the same place

extern crate bmp;
extern crate framebuffer;

use framebuffer::{Framebuffer, KdMode};
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::{thread, time};

fn read_u32_from_file(fname: &str) -> io::Result<u32> {
    let mut f = File::open(fname)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    buffer.trim().parse::<u32>()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "can't parse number"))
}

fn main() -> std::io::Result<()> {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let h = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;

    let mut frame = vec![0u8; (line_length * h) as usize];
    let img = bmp::open("/sys/firmware/acpi/bgrt/image").unwrap();
    let xoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/xoffset")?;
    let yoffset = read_u32_from_file("/sys/firmware/acpi/bgrt/yoffset")?;

    // Disable text mode in current tty
    let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

    for (x, y) in img.coordinates() {
        let px = img.get_pixel(x, y);
        let start_index = ((y + yoffset) * line_length + (xoffset + x) * bytespp) as usize;
        frame[start_index] = px.b;
        frame[start_index + 1] = px.g;
        frame[start_index + 2] = px.r;
    }

    let _ = framebuffer.write_frame(&frame);

    thread::sleep(time::Duration::new(5, 0));

    // Reenable text mode in current tty
    let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
    Ok(())
}
