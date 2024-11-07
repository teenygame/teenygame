use std::{f32::consts::TAU, num::NonZero, ops::Deref};

use soa_rs::{soa, Soa, Soars};
use teenygame::{
    graphics::{font, Canvas, Color, Drawable, LazyTexture, TextureSlice},
    image,
    input::KeyCode,
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
    deaths: usize,
    n: usize,
    bullets: Soa<Bullet>,
    bullet_texture: LazyTexture,
    player_pos: Vec2,
    elapsed: usize,
    last_draw_time: time::Instant,
    face: font::Attrs,
}

struct TextureSlices<'a> {
    bullet: TextureSlice<'a>,
    player: TextureSlice<'a>,
}

impl<'a> TextureSlices<'a> {
    fn new(parent: TextureSlice<'a>) -> Option<Self> {
        Some(Self {
            bullet: parent.slice(ivec2(0, 48), uvec2(16, 16))?,
            player: parent.slice(ivec2(0, 440), uvec2(8, 8))?,
        })
    }
}

const SIZE: UVec2 = uvec2(1024, 1024);

const SCALE: u32 = 2;

const BULLET_RADIUS: f32 = 4.0;
const PLAYER_HITBOX: f32 = 4.0;

impl teenygame::Game for Game {
    fn new(ctxt: &mut Context) -> Self {
        let window = ctxt.gfx.window();
        window.set_title("Bullet Hell");
        window.set_size(SIZE * SCALE, false);

        Self {
            deaths: 0,
            n: 0,
            bullets: soa![],
            bullet_texture: LazyTexture::new(
                image::load_from_memory(include_bytes!("Shot_01.png")).unwrap(),
            ),
            player_pos: Vec2::new(SIZE.x as f32 / 2.0, SIZE.y as f32 * 3.0 / 4.0),
            elapsed: 0,
            last_draw_time: time::Instant::now(),
            face: ctxt
                .gfx
                .add_font(include_bytes!("PixelOperator.ttf"))
                .remove(0),
        }
    }

    fn update(&mut self, ctxt: &mut Context) {
        let mut direction = Vec2::ZERO;

        for (keycode, d) in [
            (KeyCode::ArrowUp, Vec2::new(0.0, -1.0)),
            (KeyCode::ArrowDown, Vec2::new(0.0, 1.0)),
            (KeyCode::ArrowLeft, Vec2::new(-1.0, 0.0)),
            (KeyCode::ArrowRight, Vec2::new(1.0, 0.0)),
        ] {
            if ctxt.input.keyboard.is_key_held(keycode) {
                direction += d;
            }
        }

        const NORMAL_SPEED: f32 = 5.0;
        const FOCUSED_SPEED: f32 = 2.0;

        let speed = if ctxt.input.keyboard.is_key_held(KeyCode::ShiftLeft) {
            FOCUSED_SPEED
        } else {
            NORMAL_SPEED
        };

        if direction != Vec2::ZERO {
            let next_pos = self.player_pos + direction.normalize() * speed;
            if next_pos.x >= 0.0
                && next_pos.y >= 0.0
                && next_pos.x < SIZE.x as f32
                && next_pos.y < SIZE.y as f32
            {
                self.player_pos = next_pos;
            }
        }

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
            const REPEATS: usize = 6;
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

        let dead = self.bullets.pos().iter().any(|pos| {
            (pos.x - self.player_pos.x).powi(2) + (pos.y - self.player_pos.y).powi(2)
                <= (BULLET_RADIUS + PLAYER_HITBOX).powi(2)
        });

        if dead {
            self.deaths += 1;
            for (_, bullet) in self.bullets.iter_mut().enumerate() {
                let dist = bullet.pos.distance(self.player_pos);
                *bullet.lifetime = Some(
                    NonZero::new((dist / 30.0) as u32).unwrap_or_else(|| NonZero::new(1).unwrap()),
                );
            }
            self.player_pos = Vec2::new(SIZE.x as f32 / 2.0, SIZE.y as f32 * 3.0 / 4.0);
        }

        self.elapsed += 1;
    }

    fn draw<'a>(&'a mut self, ctxt: &mut Context, canvas: &mut Canvas<'a>) {
        let start_time = time::Instant::now();
        let slices = TextureSlices::new(TextureSlice::from(
            self.bullet_texture.get_or_load_texture(&ctxt.gfx),
        ))
        .unwrap();

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

            canvas.draw(
                slices.player,
                Affine2::from_scale(vec2(SCALE as f32, SCALE as f32))
                    * Affine2::from_translation(-slices.player.size().as_vec2() / 2.0)
                    * Affine2::from_translation(vec2(self.player_pos.x, self.player_pos.y)),
            );
        }

        canvas.draw(
            ctxt.gfx
                .prepare_text(
                    format!(
                        "deaths: {}\nnum bullets: {}\nfps: {:.02}",
                        self.deaths,
                        self.bullets.len(),
                        1.0 / (start_time - self.last_draw_time).as_secs_f32()
                    ),
                    font::Metrics::relative(64.0, 1.0),
                    self.face.clone(),
                )
                .tinted(Color::new(0xff, 0xff, 0xff, 0xff)),
            translate(16.0, 56.0),
        );

        self.last_draw_time = start_time;
    }
}
