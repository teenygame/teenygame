use std::f32::consts::TAU;

use glam::Vec2;
use rgb::FromSlice;
use soa_rs::{soa, Soa, Soars};
use teenygame::{
    graphics::{font, AffineTransform, Canvas, Color, ImgRef, Texture, TextureSlice},
    input::KeyCode,
    time, Context,
};

#[derive(Soars)]
pub struct Bullet {
    n: usize,
    pos: glam::Vec2,
    vel: glam::Vec2,
    theta: f32,
}

pub struct Game {
    deaths: usize,
    n: usize,
    bullets: Soa<Bullet>,
    bullet_texture: Texture,
    player_pos: glam::Vec2,
    elapsed: usize,
    last_draw_time: time::Instant,
}

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;

const SCALE: u32 = 2;

const BULLET_RADIUS: f32 = 4.0;
const PLAYER_HITBOX: f32 = 4.0;

const DEATH_CLEAR_RADIUS: f32 = 100.0;

impl teenygame::Game for Game {
    fn new(ctxt: &mut Context) -> Self {
        let window = ctxt.gfx.window();
        window.set_title("Bullet Hell");
        window.set_size(WIDTH * SCALE, HEIGHT * SCALE, false);

        ctxt.gfx.add_font(include_bytes!("PixelOperator.ttf"));

        let img = image::load_from_memory(include_bytes!("bullets.png")).unwrap();

        Self {
            deaths: 0,
            n: 0,
            bullets: soa![],
            bullet_texture: ctxt.gfx.load_texture(ImgRef::new(
                &img.as_rgba8().unwrap().as_rgba(),
                img.width() as usize,
                img.height() as usize,
            )),
            player_pos: glam::Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 * 3.0 / 4.0),
            elapsed: 0,
            last_draw_time: time::Instant::now(),
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
                && next_pos.x < WIDTH as f32
                && next_pos.y < HEIGHT as f32
            {
                self.player_pos = next_pos;
            }
        }

        let mut cleanup = vec![];
        for (i, bullet) in self.bullets.iter_mut().enumerate() {
            *bullet.pos += *bullet.vel;
            if bullet.pos.x < -(WIDTH as f32 * 0.5)
                || bullet.pos.y < -(HEIGHT as f32 * 0.5)
                || bullet.pos.x >= WIDTH as f32 * 1.5
                || bullet.pos.y >= HEIGHT as f32 * 1.5
            {
                cleanup.push(i);
            }
        }

        cleanup.reverse();
        for i in cleanup {
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
                    pos: glam::Vec2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0),
                    vel: glam::Vec2::new(c * SPEED, s * SPEED),
                    theta,
                });
                self.n += 1;
            }
        }

        let mut dead = false;
        for pos in self.bullets.pos() {
            if (pos.x - self.player_pos.x).powi(2) + (pos.y - self.player_pos.y).powi(2)
                <= (BULLET_RADIUS + PLAYER_HITBOX).powi(2)
            {
                dead = true;
                break;
            }
        }

        if dead {
            self.deaths += 1;
            for i in self
                .bullets
                .pos()
                .iter()
                .enumerate()
                .filter_map(|(i, pos)| {
                    if (pos.x - self.player_pos.x).powi(2) + (pos.y - self.player_pos.y).powi(2)
                        <= DEATH_CLEAR_RADIUS.powi(2)
                    {
                        Some(i)
                    } else {
                        None
                    }
                })
                .rev()
                .collect::<Vec<_>>()
            {
                self.bullets.swap_remove(i);
            }
        }

        self.elapsed += 1;
    }

    fn draw<'a>(&'a mut self, ctxt: &mut Context, canvas: &mut Canvas<'a>) {
        let start_time = time::Instant::now();

        let bullet_texture_slice = TextureSlice::from(&self.bullet_texture)
            .slice(0, 272, 16, 32)
            .unwrap();

        let player_texture_slice = TextureSlice::from(&self.bullet_texture)
            .slice(0, 16, 16, 16)
            .unwrap();

        for (n, (pos, theta)) in self
            .bullets
            .n()
            .iter()
            .zip(self.bullets.pos().iter().zip(self.bullets.theta()))
        {
            let color = coolor::Hsl {
                h: *n as f32 / 2.0,
                s: 1.0,
                l: 0.5,
            }
            .to_rgb();

            let [tw, th] = bullet_texture_slice.size();
            canvas.draw_sprite(
                bullet_texture_slice,
                Color::new(color.r, color.g, color.b, 0xff),
                AffineTransform::translation(-(tw as f32) / 2.0, -(th as f32) / 2.0)
                    * AffineTransform::rotation(theta + TAU / 4.0)
                    * AffineTransform::translation(pos.x, pos.y)
                    * AffineTransform::scaling(SCALE as f32, SCALE as f32),
            );

            let [tw, th] = player_texture_slice.size();
            canvas.draw_sprite(
                player_texture_slice,
                Color::new(0xff, 0xff, 0xff, 0xff),
                AffineTransform::translation(-(tw as f32) / 2.0, -(th as f32) / 2.0)
                    * AffineTransform::translation(self.player_pos.x, self.player_pos.y)
                    * AffineTransform::scaling(SCALE as f32, SCALE as f32),
            );
        }

        canvas.draw_text(
            ctxt.gfx.prepare_text(
                format!(
                    "deaths: {}\nnum bullets: {}\nfps: {:.02}",
                    self.deaths,
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
