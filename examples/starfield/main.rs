use framebuffer::Framebuffer;
use rand::Rng;

const STAR_SPEED: f32 = 1.003;
const STAR_GROWTH: f32 = 1.002;
const STAR_COUNT: usize = 40;

struct Starfield {
    stars: [Star; STAR_COUNT],
}

impl Starfield {
    fn new(framebuffer: &Framebuffer) -> Starfield {
        let w = framebuffer.var_screen_info.xres as usize;
        let h = framebuffer.var_screen_info.yres as usize;

        let mut stars = [Star {
            a: 0.0,
            b: 0.0,
            x: 0.0,
            size: 0.0,
            color: (255, 255, 255),
        }; STAR_COUNT];
        for star in stars.iter_mut() {
            *star = Star::new_rand(w, h);
        }
        Starfield { stars }
    }

    fn tick(&mut self, framebuffer: &Framebuffer, frame: &mut [u8]) {
        let w = framebuffer.var_screen_info.xres as usize;
        let h = framebuffer.var_screen_info.yres as usize;

        for star in self.stars.iter_mut() {
            Starfield::draw_star(star, framebuffer, frame, (0, 0, 0));
            star.tick(w, h);
            Starfield::draw_star(star, framebuffer, frame, star.color);
        }
    }

    fn draw_star(
        star_data: &Star,
        framebuffer: &Framebuffer,
        frame: &mut [u8],
        color: (u8, u8, u8),
    ) {
        let w = framebuffer.var_screen_info.xres as usize;
        let h = framebuffer.var_screen_info.yres as usize;

        let line_length = framebuffer.fix_screen_info.line_length as usize;
        let bytespp = framebuffer.var_screen_info.bits_per_pixel as usize / 8;

        macro_rules! coords_to_index {
            ($x:expr, $y:expr) => {
                $y * line_length + $x * bytespp
            };
        }

        let pos = star_data.get_pos(w, h);

        let dim = star_data.size as usize;
        for i in 0..dim {
            for j in 0..dim {
                if pos.0 + i < w && pos.1 + j < h {
                    frame[coords_to_index!(pos.0 + i, pos.1 + j)] = color.0;
                    frame[coords_to_index!(pos.0 + i, pos.1 + j) + 1] = color.1;
                    frame[coords_to_index!(pos.0 + i, pos.1 + j) + 2] = color.2;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Star {
    color: (u8, u8, u8),
    a: f32,
    b: f32,
    x: f32,
    size: f32,
}

impl Star {
    fn new_rand(w: usize, h: usize) -> Star {
        let mut star = Star {
            a: 0.0,
            x: 0.0,
            b: 0.0,
            size: 0.0,
            color: (0, 0, 0),
        };
        star.init(w, h);
        star
    }

    fn init(&mut self, w: usize, h: usize) {
        let wh = w as f32 / 4.0;
        let hh = h as f32 / 4.0;

        let mut rng = rand::thread_rng();
        self.x = rng.gen_range(-wh..wh);
        self.b = rng.gen_range(-hh..hh);
        if self.x != 0.0 {
            self.a = self.b / self.x;
        }
        self.size = rng.gen_range(1.0..1.001);
        self.color.0 = rng.gen_range(128..255);
        self.color.1 = rng.gen_range(128..255);
        self.color.2 = rng.gen_range(128..255);
    }

    fn get_pos(&self, w: usize, h: usize) -> (usize, usize) {
        let x_coord = (self.x + w as f32 / 2.0) as usize;
        let y_coord = (self.a * self.x + self.b + h as f32 / 2.0) as usize; //y = ax + b
        (x_coord, y_coord)
    }

    fn tick(&mut self, w: usize, h: usize) {
        let pos = self.get_pos(w, h);
        if pos.0 >= w || pos.1 >= h || pos.0 == 0 || pos.1 == 0 {
            self.init(w, h);
        }

        self.x *= STAR_SPEED;
        self.size *= STAR_GROWTH;
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new("/dev/fb0").unwrap();

    let _w = framebuffer.var_screen_info.xres_virtual;
    let h = framebuffer.var_screen_info.yres_virtual;
    let line_length = framebuffer.fix_screen_info.line_length;
    let mut frame = vec![0u8; (line_length * h) as usize];

    let mut starfield = Starfield::new(&framebuffer);

    //Disable text mode in current tty
    //let _ = Framebuffer::set_kd_mode(KdMode::Graphics).unwrap();

    loop {
        starfield.tick(&framebuffer, &mut frame);
        framebuffer.write_frame(&frame);
    }

    //Reenable text mode in current tty
    //let _ = Framebuffer::set_kd_mode(KdMode::Text).unwrap();
}
