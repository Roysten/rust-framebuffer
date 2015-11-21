extern crate rand;
extern crate framebuffer;

use rand::Rng;
use framebuffer::{KdMode, Framebuffer};

const STAR_SPEED: f32 = 1.003;
const STAR_GROWTH: f32 = 1.002;
const STAR_COUNT: usize = 10;

struct Starfield {
    stars: [Star; STAR_COUNT],
}

impl Starfield {

    fn new(framebuffer: &Framebuffer) -> Starfield {
        let w = framebuffer.var_screen_info.xres as usize;
        let h = framebuffer.var_screen_info.yres as usize;

        let stars = [Star::new_rand(w, h); STAR_COUNT];
        Starfield {
            stars: stars,
        }
    }
    
    fn tick(&mut self, framebuffer: &Framebuffer, frame: &mut [u8]) {
        let w = framebuffer.var_screen_info.xres as usize;
        let h = framebuffer.var_screen_info.yres as usize;

        for star in self.stars.iter_mut() {
            let star_data = star.tick(w, h);
            if star_data.0 < w && star_data.1 < h {
                Starfield::draw_star(star_data, framebuffer, frame);
            }
        }
    }


    fn draw_star(star_data: (usize, usize, f32), framebuffer: &Framebuffer, frame: &mut[u8]) {
        let w = framebuffer.var_screen_info.xres as usize;
        let h = framebuffer.var_screen_info.yres as usize;
    
        let line_length = framebuffer.fix_screen_info.line_length as usize;
        let bytespp = framebuffer.var_screen_info.bits_per_pixel as usize / 8;

        macro_rules! coords_to_index {
            ($x:expr, $y: expr) => { $y * line_length + $x * bytespp }
        }

        let dim = star_data.2 as usize + 1;
        for i in 0 .. dim {
            for j in 0 .. dim {
                if star_data.0 + i < w && star_data.1 + j < h {
                    frame[coords_to_index!(star_data.0 + i, star_data.1 + j)] = 255;
                    frame[coords_to_index!(star_data.0 + i, star_data.1 + j) + 1] = 255;
                    frame[coords_to_index!(star_data.0 + i, star_data.1 + j) + 2] = 255;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Star {
    a: f32,
    b: f32,
    x: f32,
    z: f32,
}

impl Star {

    fn new_rand(w: usize, h: usize) -> Star {
        let mut star = Star { 
            a: 0.0, 
            x: 0.0,
            b: 0.0,
            z: 0.0,
        };
        star.init(w, h);
        star
    }

    fn init(&mut self, w: usize, h: usize) {
        let wh = w as f32 / 2.0;
        let hh = h as f32 / 2.0;

        let mut rng = rand::thread_rng();
        self.x = rng.gen_range::<f32>(-wh, wh);
        self.b = rng.gen_range::<f32>(-hh, hh);
        self.a = self.b / self.x;
        self.z = rng.gen_range::<f32>(1.0, 1.001);
    }

    fn get_pos(&self, w: usize, h: usize) -> (usize, usize) {
        let x_coord = (self.x + w as f32 / 2.0) as usize;
        let y_coord = (self.a * self.x + self.b + h as f32 / 2.0) as usize; //y = ax + b
        (x_coord, y_coord)
    }

    fn tick(&mut self, w: usize, h: usize) -> (usize, usize, f32) {
        let mut pos = self.get_pos(w, h);
        if pos.0 >= w || pos.1 >= h {
            self.init(w, h);
            pos = self.get_pos(w, h);
        }

        self.x *= STAR_SPEED;
        self.z *= STAR_GROWTH;
        (pos.0, pos.1, self.z)
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let w = framebuffer.var_screen_info.xres;
    let h = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let mut frame = vec![0u8; (line_length * h) as usize];

    let mut starfield = Starfield::new(&framebuffer);

    //Disable text mode in current tty
    //let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();
    
    loop {
        for x in frame.iter_mut() { *x = 0; }
        starfield.tick(&framebuffer, &mut frame);
        let _ = framebuffer.write_frame(&frame);
    }
    
    //Reenable text mode in current tty
    //let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}
