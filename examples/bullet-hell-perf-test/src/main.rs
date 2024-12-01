use std::{f32::consts::TAU, num::NonZero};

use soa_rs::{soa, Soa, Soars};
use teenygame::{
    graphics::{font, Canvas, Color, Drawable, Lazy, Texture, TextureSlice},
    image,
    math::*,
    time, Context,
};

#[derive(Soars)]
struct Bullet {
    n: usize,
    pos: Vec2,
    vel: Vec2,
    theta: f32,
    lifetime: Option<NonZero<u32>>,
}

#[teenygame::game]
struct Game {
    n: usize,
    bullets: Soa<Bullet>,
    bullet_texture: Lazy<Texture>,
    elapsed: usize,
    last_draw_time: time::Instant,
    font: Lazy<Vec<font::Attrs>>,
}

struct TextureSlices<'a> {
    bullet: TextureSlice<'a>,
}

impl<'a> TextureSlices<'a> {
    fn new(parent: &'a Texture) -> Option<Self> {
        Some(Self {
            bullet: parent.layer(0)?.slice(ivec2(0, 48), uvec2(16, 16))?,
        })
    }
}

const SIZE: UVec2 = uvec2(1024, 1024);

const SCALE: u32 = 2;

impl teenygame::Game for Game {
    fn new() -> Self {
        Self {
            n: 0,
            bullets: soa![],
            bullet_texture: Lazy::new(
                image::load_from_memory(include_bytes!("Shot_01.png")).unwrap(),
            ),
            elapsed: 0,
            last_draw_time: time::Instant::now(),
            font: Lazy::new(include_bytes!("PixelOperator.ttf").to_vec()),
        }
    }

    fn resumed(&mut self, ctxt: &mut Context) {
        let window = ctxt.gfx.window();
        window.set_title("Bullet Hell");
        window.set_size(SIZE * SCALE, false);
    }

    fn update(&mut self, _ctxt: &mut Context) {
        let mut cleanup = vec![];
        for (i, mut bullet) in self.bullets.iter_mut().enumerate() {
            *bullet.pos += *bullet.vel;
            if bullet.pos.x < -(SIZE.x as f32 * 0.5)
                || bullet.pos.y < -(SIZE.y as f32 * 0.5)
                || bullet.pos.x >= SIZE.x as f32 * 1.5
                || bullet.pos.y >= SIZE.y as f32 * 1.5
            {
                cleanup.push(i);
            }
            if let Some(lifetime) = &mut bullet.lifetime {
                *lifetime = if let Some(l) =
                    NonZero::new(u32::from(*lifetime).checked_sub(1).unwrap_or(0))
                {
                    l
                } else {
                    cleanup.push(i);
                    continue;
                };
            }
        }

        for i in cleanup.into_iter().rev() {
            self.bullets.swap_remove(i);
        }

        if self.elapsed % 2 == 0 {
            let t = self.elapsed as f32 / 100.0;
            let theta_base = t.sin() * 6.0;
            const REPEATS: usize = 180;
            for i in 0..REPEATS {
                let theta2 = i as f32 / REPEATS as f32 * TAU;
                let theta = theta_base + theta2;

                let c = theta.cos();
                let s = theta.sin();

                const SPEED: f32 = 3.0;
                self.bullets.push(Bullet {
                    n: self.n,
                    pos: SIZE.as_vec2() / 2.0,
                    vel: Vec2::new(c * SPEED, s * SPEED),
                    lifetime: None,
                    theta,
                });
                self.n += 1;
            }
        }

        self.elapsed += 1;
    }

    fn draw<'a>(&'a mut self, ctxt: &mut Context, canvas: &mut Canvas<'a>) {
        let start_time = time::Instant::now();
        let slices = TextureSlices::new(self.bullet_texture.get_or_load(ctxt.gfx)).unwrap();

        let mut to_draw = self
            .bullets
            .n()
            .iter()
            .cloned()
            .zip(
                self.bullets
                    .pos()
                    .iter()
                    .cloned()
                    .zip(self.bullets.theta().iter().cloned()),
            )
            .collect::<Vec<_>>();
        to_draw.sort_by_key(|(n, (_, _))| *n);

        for (n, (pos, theta)) in to_draw {
            let color = coolor::Hsl {
                h: n as f32 / 5.0,
                s: 1.0,
                l: 0.5,
            }
            .to_rgb();

            canvas.draw(
                slices
                    .bullet
                    .tinted(Color::new(color.r, color.g, color.b, 0xff)),
                Affine2::from_scale(vec2(SCALE as f32, SCALE as f32))
                    * Affine2::from_translation(vec2(pos.x, pos.y))
                    * Affine2::from_angle(theta + TAU / 4.0)
                    * Affine2::from_translation(-slices.bullet.size().as_vec2() / 2.0),
            );
        }

        let face = self.font.get_or_load(ctxt.gfx)[0].clone();

        canvas.draw(
            ctxt.gfx
                .prepare_text(
                    format!(
                        "num bullets: {}\nfps: {:.02}",
                        self.bullets.len(),
                        1.0 / (start_time - self.last_draw_time).as_secs_f32()
                    ),
                    font::Metrics::relative(64.0, 1.0),
                    face,
                )
                .tinted(Color::new(0xff, 0xff, 0xff, 0xff)),
            translate(16.0, 56.0),
        );

        self.last_draw_time = start_time;
    }
}
