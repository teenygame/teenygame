use std::f32::consts::TAU;

use rgb::FromSlice;
use slotmap::{DefaultKey, DenseSlotMap};
use teenygame::{
    graphics::{font, AffineTransform, Color, ImgRef, Scene, Texture, TextureSlice},
    time, Context,
};

pub struct Bullet {
    n: usize,
    pos: glam::Vec2,
    vel: glam::Vec2,
    accel: glam::Vec2,
    theta: f32,
}

pub struct Game {
    n: usize,
    bullets: DenseSlotMap<DefaultKey, Bullet>,
    bullet_texture: Texture,
    elapsed: usize,
    last_draw_time: time::Instant,
}

const WIDTH: u32 = 2048;
const HEIGHT: u32 = 2048;

impl teenygame::Game for Game {
    fn new(ctxt: &mut Context) -> Self {
        let window = ctxt.gfx.window();
        window.set_title("Bullet Hell");
        window.set_size(WIDTH, HEIGHT, false);

        ctxt.gfx.add_font(include_bytes!("PixelOperator.ttf"));

        let img = image::load_from_memory(include_bytes!("bullets.png")).unwrap();

        Self {
            n: 0,
            bullets: DenseSlotMap::new(),
            bullet_texture: ctxt.gfx.load_texture(ImgRef::new(
                &img.as_rgba8().unwrap().as_rgba(),
                img.width() as usize,
                img.height() as usize,
            )),
            elapsed: 0,
            last_draw_time: time::Instant::now(),
        }
    }

    fn update(&mut self, _ctxt: &mut Context) {
        let mut cleanup = vec![];
        for (idx, bullet) in self.bullets.iter_mut() {
            bullet.vel += bullet.accel;
            bullet.pos += bullet.vel;
            if bullet.pos.x < -(WIDTH as f32 * 0.5)
                || bullet.pos.y < -(HEIGHT as f32 * 0.5)
                || bullet.pos.x >= WIDTH as f32 * 1.5
                || bullet.pos.y >= HEIGHT as f32 * 1.5
            {
                cleanup.push(idx);
            }
        }

        for idx in cleanup {
            self.bullets.remove(idx);
        }

        let t = self.elapsed as f32 / 50.0;
        let theta_base = t.sin() * t.cos() * 12.0;
        const REPEATS: usize = 12;
        for i in 0..REPEATS {
            let theta = theta_base + i as f32 / REPEATS as f32 * TAU;

            let c = theta.cos();
            let s = theta.sin();

            const SPEED: f32 = 3.0;
            self.bullets.insert(Bullet {
                n: self.n,
                pos: glam::Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
                accel: glam::Vec2::new(0.0, 0.0),
                vel: glam::Vec2::new(c * SPEED, s * SPEED),
                theta,
            });
            self.n += 1;
        }
        self.elapsed += 1;
    }

    fn draw<'a>(&'a mut self, ctxt: &mut Context, scene: &mut Scene<'a>) {
        let start_time = time::Instant::now();

        for (_, bullet) in self.bullets.iter() {
            let texture_slice = TextureSlice::from(&self.bullet_texture)
                .slice(0, 272, 16, 32)
                .unwrap();

            let color = coolor::Hsl {
                h: bullet.n as f32 / 2.0,
                s: 1.0,
                l: 0.5,
            }
            .to_rgb();

            let [tw, th] = texture_slice.size();
            scene.draw_sprite(
                texture_slice,
                Color::new(color.r, color.g, color.b, 0xff),
                AffineTransform::translation(-(tw as f32) / 2.0, -(th as f32) / 2.0)
                    * AffineTransform::rotation(bullet.theta + TAU / 4.0)
                    * AffineTransform::translation(bullet.pos.x, bullet.pos.y),
            );
        }

        scene.draw_text(
            ctxt.gfx.prepare_text(
                format!(
                    "num bullets: {}\nfps: {:.02}",
                    self.bullets.len(),
                    1.0 / (start_time - self.last_draw_time).as_secs_f32()
                ),
                font::Metrics::relative(64.0, 1.0),
                font::Attrs::default(),
            ),
            Color::new(0xff, 0xff, 0xff, 0xff),
            AffineTransform::translation(16.0, 56.0),
        );

        self.last_draw_time = start_time;
    }
}
