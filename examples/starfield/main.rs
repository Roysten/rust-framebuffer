extern crate rand;
extern crate framebuffer;

use rand::Rng;
use framebuffer::{KdMode, Framebuffer};

const STAR_SPEED: f32 = 0.001;
const STAR_COUNT: usize = 1000;

struct Starfield {
    stars: [Star; STAR_COUNT],
    w: usize,
    h: usize,
    line_length: usize,
    bytespp: usize,
}

impl Starfield {

    fn new(w: usize, h: usize, line_length: usize, bytespp: usize) -> Starfield {
        let stars = [Star::new_rand(w, h); STAR_COUNT];
        Starfield {
            stars: stars,
            w: w,
            h: h,
            line_length: line_length,
            bytespp: bytespp,
        }
    }
    
    fn tick(&mut self, frame: &mut [u8]) {
        for star in self.stars.iter_mut() {
            let pos = star.tick();
            if pos.0 < self.w && pos.1 < self.h {
                //self.draw_dot();
                frame[pos.1 * self.line_length + pos.0 * self.bytespp] = 255;
                frame[pos.1 * self.line_length + pos.0 * self.bytespp + 1] = 255;
                frame[pos.1 * self.line_length + pos.0 * self.bytespp + 2] = 255;
            } else {
                //Re-init star when out of bounds
                star.init();
            }
        }
    }

    fn draw_dot(&self) {
    }
}

#[derive(Clone, Copy)]
struct Star {
    w: f32,
    h: f32,

    a: f32,
    b: f32,
    x: f32,
}

impl Star {

    fn new_rand(w: usize, h: usize) -> Star {
        let mut star = Star { 
            w: w as f32,
            h: h as f32,
            a: 0.0, 
            x: 0.0,
            b: 0.0,
        };
        star.init();
        star
    }

    fn init(&mut self)  {
        let wh = self.w / 2.0;
        let hh = self.h / 2.0;

        let mut rng = rand::thread_rng();
        self.x = rng.gen_range::<f32>(-wh, wh);
        self.b = rng.gen_range::<f32>(-hh, hh);
        self.a = self.b / self.x;
    }

    fn tick(&mut self) -> (usize, usize) {
        let pos = (
            (self.x + self.w / 2.0) as usize,
            (self.a * self.x + self.b + self.h / 2.0) as usize
        );
        self.x += if self.x < 0.0 { self.x * STAR_SPEED } else { self.x * STAR_SPEED };
        pos
    }

}

fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let w = framebuffer.var_screen_info.xres;
    let h = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;
    let mut frame = vec![0u8; (line_length * h) as usize];

    let mut starfield = Starfield::new(w as usize, h as usize, line_length as usize, bytespp as usize);

    //Disable text mode in current tty
    //let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    
    /*let index = ((h / 2) * line_length + (w / 2) * bytespp) as usize;
    frame[index] = 255;
    frame[index + 1] = 255;
    frame[index + 2] = 255;

    loop { framebuffer.write_frame(&frame); } */
    loop {
        for x in frame.iter_mut() { *x = 0; }
        starfield.tick(&mut frame);
        let _ = framebuffer.write_frame(&frame);
    }
    
    //Reenable text mode in current tty
    //let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}
