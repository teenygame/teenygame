use rand::prelude::IteratorRandom;
use std::collections::VecDeque;
use teenygame::{
    audio::{PlaybackHandle, Region, Sound, Source},
    graphics::{
        font::{Attrs, Metrics},
        Canvas, Color, Drawable as _, Texture, TextureSlice,
    },
    input::KeyCode,
    math::*,
    Context,
};

const BOARD_SIZE: UVec2 = uvec2(40, 22);

const CELL_SIZE: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Fruit,
    Snake,
}

const NORTH: IVec2 = ivec2(0, -1);
const SOUTH: IVec2 = ivec2(0, 1);
const EAST: IVec2 = ivec2(1, 0);
const WEST: IVec2 = ivec2(-1, 0);

pub struct Game {
    texture: Texture,
    pickup_sfx: Source,
    game_over_sfx: Source,
    bgm_handle: Option<PlaybackHandle>,
    game_over: bool,
    board: [[Option<Cell>; BOARD_SIZE.x as usize]; BOARD_SIZE.y as usize],
    snake: VecDeque<UVec2>,
    direction: IVec2,
    next_direction: IVec2,
    score: u32,
    elapsed: u32,
}

impl Game {
    fn spawn_fruit(&mut self) {
        let mut rng = rand::thread_rng();
        let (x, y) = self
            .board
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .filter(|(_, cell)| cell.is_none())
                    .map(move |(x, _)| (x, y))
            })
            .choose(&mut rng)
            .unwrap();
        self.board[y][x] = Some(Cell::Fruit);
    }
}

impl teenygame::Game for Game {
    fn new(ctxt: &mut Context) -> Self {
        let window = ctxt.gfx.window();
        window.set_title("Snake");
        window.set_size(BOARD_SIZE * CELL_SIZE, false);

        let mut board = [[None; BOARD_SIZE.x as usize]; BOARD_SIZE.y as usize];
        let snake = VecDeque::from([BOARD_SIZE / 2]);

        for pos in snake.iter() {
            board[pos.y as usize][pos.x as usize] = Some(Cell::Snake);
        }

        ctxt.gfx.add_font(include_bytes!("PixelOperator.ttf"));

        let bgm_source = Source::load(include_bytes!("8BitCave.wav")).unwrap();

        let mut game = Self {
            texture: ctxt.gfx.load_texture(teenygame::graphics::ImgRef::new(
                &[Color::new(0xff, 0xff, 0xff, 0xff)],
                1,
                1,
            )),
            pickup_sfx: Source::load(include_bytes!("pickup.wav")).unwrap(),
            game_over_sfx: Source::load(include_bytes!("game_over.wav")).unwrap(),
            bgm_handle: Some(ctxt.audio.play(&Sound {
                loop_region: Some(Region {
                    start: 0,
                    length: bgm_source.num_frames(),
                }),
                start_position: 5190,
                ..Sound::new(&bgm_source)
            })),
            game_over: false,
            board,
            snake,
            direction: SOUTH,
            next_direction: SOUTH,
            score: 0,
            elapsed: 0,
        };
        game.spawn_fruit();
        game
    }

    fn update(&mut self, ctxt: &mut Context) {
        if self.game_over {
            return;
        }

        if (ctxt.input.keyboard.is_key_held(KeyCode::ArrowLeft)
            || ctxt.input.keyboard.is_key_held(KeyCode::KeyA))
            && self.direction != EAST
        {
            self.next_direction = WEST;
        }
        if (ctxt.input.keyboard.is_key_held(KeyCode::ArrowRight)
            || ctxt.input.keyboard.is_key_held(KeyCode::KeyD))
            && self.direction != WEST
        {
            self.next_direction = EAST;
        }
        if (ctxt.input.keyboard.is_key_held(KeyCode::ArrowUp)
            || ctxt.input.keyboard.is_key_held(KeyCode::KeyW))
            && self.direction != SOUTH
        {
            self.next_direction = NORTH;
        }
        if (ctxt.input.keyboard.is_key_held(KeyCode::ArrowDown)
            || ctxt.input.keyboard.is_key_held(KeyCode::KeyS))
            && self.direction != NORTH
        {
            self.next_direction = SOUTH;
        }

        let ticks_per_move = ((15.0 / (self.score as f32 + 1.0).powf(0.25)) as u32).max(1);

        self.elapsed += 1;
        if self.elapsed < ticks_per_move {
            return;
        }

        self.direction = self.next_direction;

        let head = *self.snake.back().unwrap();

        let head2 = head.as_ivec2() + self.direction;
        let head2 = uvec2(
            head2.x.rem_euclid(BOARD_SIZE.x as i32) as u32,
            head2.y.rem_euclid(BOARD_SIZE.y as i32) as u32,
        );
        self.snake.push_back(head2.into());

        match self.board[head2.y as usize][head2.x as usize] {
            None => {
                let ohead = self.snake.pop_front().unwrap();
                self.board[ohead.y as usize][ohead.x as usize] = None;
            }
            Some(Cell::Fruit) => {
                self.spawn_fruit();
                self.score += 1;
                if let Some(handle) = &mut self.bgm_handle {
                    handle.set_speed((self.score as f64 + 1.0).powf(0.02));
                }
                ctxt.audio.play(&Sound::new(&self.pickup_sfx)).detach();
            }
            Some(Cell::Snake) => {
                self.game_over = true;
                ctxt.audio.play(&Sound::new(&self.game_over_sfx)).detach();
                self.bgm_handle.take();
            }
        }
        self.board[head2.y as usize][head2.x as usize] = Some(Cell::Snake);
        self.elapsed = 0;
    }

    fn draw<'a>(&'a mut self, ctxt: &mut Context, canvas: &mut Canvas<'a>) {
        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                canvas.draw_with_transform(
                    TextureSlice::from(&self.texture).tinted(match cell {
                        None => {
                            continue;
                        }
                        Some(Cell::Fruit) => Color::new(0xff, 0x00, 0x00, 0xff),
                        Some(Cell::Snake) => Color::new(0xff, 0xff, 0xff, 0xff),
                    }),
                    Affine2::from_translation(vec2(x as f32, y as f32) * CELL_SIZE as f32)
                        * Affine2::from_scale(vec2(CELL_SIZE as f32, CELL_SIZE as f32)),
                );
            }
        }

        canvas.draw(
            ctxt.gfx
                .prepare_text(
                    format!("Score: {}", self.score),
                    Metrics::relative(64.0, 1.0),
                    Attrs::default(),
                )
                .tinted(Color::new(0xff, 0xff, 0xff, 0xff)),
            vec2(16.0, 56.0),
        );

        if self.game_over {
            let prepared_game_over =
                ctxt.gfx
                    .prepare_text("GAME OVER", Metrics::relative(128.0, 1.0), Attrs::default());
            let game_over_size = prepared_game_over.size();
            canvas.draw(
                prepared_game_over.tinted(Color::new(0xff, 0x00, 0x00, 0xff)),
                vec2(
                    (BOARD_SIZE.x * CELL_SIZE / 2) as f32 - game_over_size.x / 2.0,
                    (BOARD_SIZE.y * CELL_SIZE / 2) as f32 + game_over_size.y / 2.0,
                ),
            );
        }
    }
}
