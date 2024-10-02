use rand::prelude::IteratorRandom;
use std::collections::VecDeque;
use teenygame::{
    audio::{PlaybackHandle, Region, Sound, Source},
    graphics::{AffineTransform, Align, Color, Font, Paint, Path, Stroke, TextStyle},
    input::KeyCode,
    UpdateContext,
};

const BOARD_WIDTH: usize = 40;
const BOARD_HEIGHT: usize = 22;

const CELL_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Fruit,
    Snake,
}

type Direction = (isize, isize);

const NORTH: Direction = (0, -1);
const SOUTH: Direction = (0, 1);
const EAST: Direction = (1, 0);
const WEST: Direction = (-1, 0);

pub struct Game {
    font: Font,
    pickup_sfx: Source,
    game_over_sfx: Source,
    bgm_handle: Option<PlaybackHandle>,
    game_over: bool,
    board: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT],
    snake: VecDeque<(usize, usize)>,
    direction: Direction,
    next_direction: Direction,
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
                    .filter(|(_, cell)| **cell == Cell::Empty)
                    .map(move |(x, _)| (x, y))
            })
            .choose(&mut rng)
            .unwrap();
        self.board[y][x] = Cell::Fruit;
    }
}

impl teenygame::Game for Game {
    fn new(s: &mut UpdateContext) -> Self {
        s.window.set_title("Snake");
        s.window.set_size(
            (BOARD_WIDTH * CELL_SIZE) as u32,
            (BOARD_HEIGHT * CELL_SIZE) as u32,
            true,
        );

        let mut board = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT];
        let snake = VecDeque::from([(BOARD_WIDTH / 2, BOARD_HEIGHT / 2)]);

        for (x, y) in snake.iter() {
            board[*y][*x] = Cell::Snake;
        }

        let bgm_source = Source::load(include_bytes!("8BitCave.wav")).unwrap();

        let mut game = Self {
            font: Font::load(include_bytes!("PixelOperator.ttf"), 0).unwrap(),
            pickup_sfx: Source::load(include_bytes!("pickup.wav")).unwrap(),
            game_over_sfx: Source::load(include_bytes!("game_over.wav")).unwrap(),
            bgm_handle: Some(s.audio.play(&Sound {
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

    fn update(&mut self, s: &mut UpdateContext) {
        if self.game_over {
            return;
        }

        if (s.input.keyboard.is_key_held(KeyCode::ArrowLeft)
            || s.input.keyboard.is_key_held(KeyCode::KeyA))
            && self.direction != EAST
        {
            self.next_direction = WEST;
        }
        if (s.input.keyboard.is_key_held(KeyCode::ArrowRight)
            || s.input.keyboard.is_key_held(KeyCode::KeyD))
            && self.direction != WEST
        {
            self.next_direction = EAST;
        }
        if (s.input.keyboard.is_key_held(KeyCode::ArrowUp)
            || s.input.keyboard.is_key_held(KeyCode::KeyW))
            && self.direction != SOUTH
        {
            self.next_direction = NORTH;
        }
        if (s.input.keyboard.is_key_held(KeyCode::ArrowDown)
            || s.input.keyboard.is_key_held(KeyCode::KeyS))
            && self.direction != NORTH
        {
            self.next_direction = SOUTH;
        }

        let ticks_per_move = (15 / (self.score as f32 + 1.0).powf(0.25) as u32).max(1);

        self.elapsed += 1;
        if self.elapsed < ticks_per_move {
            return;
        }

        self.direction = self.next_direction;

        let (hx, hy) = *self.snake.back().unwrap();
        let (dx, dy) = self.direction;

        let hx2 = (hx as isize + dx).rem_euclid(BOARD_WIDTH as isize) as usize;
        let hy2 = (hy as isize + dy).rem_euclid(BOARD_HEIGHT as isize) as usize;

        self.snake.push_back((hx2, hy2));

        match self.board[hy2][hx2] {
            Cell::Empty => {
                let (ohx, ohy) = self.snake.pop_front().unwrap();
                self.board[ohy][ohx] = Cell::Empty;
            }
            Cell::Fruit => {
                self.spawn_fruit();
                self.score += 1;
                s.audio.play(&Sound::new(&self.pickup_sfx)).detach();
            }
            Cell::Snake => {
                self.game_over = true;
                s.audio.play(&Sound::new(&self.game_over_sfx)).detach();
                self.bgm_handle.take();
            }
        }
        self.board[hy2][hx2] = Cell::Snake;
        self.elapsed = 0;
    }

    fn draw(&mut self, canvas: &mut teenygame::graphics::Canvas) {
        let (width, height) = canvas.size();
        canvas.clear_rect(0, 0, width, height, Color::new(0x00, 0x00, 0x00, 0xff));

        let mut canvas = canvas.transform(&AffineTransform::translation(
            (width as i32 / 2 - (BOARD_WIDTH * CELL_SIZE) as i32 / 2) as f32,
            (height as i32 / 2 - (BOARD_HEIGHT * CELL_SIZE) as i32 / 2) as f32,
        ));

        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                match cell {
                    Cell::Empty => {}
                    Cell::Fruit => {
                        canvas.fill_path(
                            &Path::new().rect(
                                (x * CELL_SIZE) as f32,
                                (y * CELL_SIZE) as f32,
                                CELL_SIZE as f32,
                                CELL_SIZE as f32,
                            ),
                            &Paint::color(Color::new(0xff, 0x00, 0x00, 0xff)),
                        );
                    }
                    Cell::Snake => {
                        canvas.fill_path(
                            &Path::new().rect(
                                (x * CELL_SIZE) as f32,
                                (y * CELL_SIZE) as f32,
                                CELL_SIZE as f32,
                                CELL_SIZE as f32,
                            ),
                            &Paint::color(Color::new(0xff, 0xff, 0xff, 0xff)),
                        );
                    }
                }
            }
        }

        canvas.fill_text(
            16.0,
            56.0,
            format!("Score: {}", self.score),
            &TextStyle {
                align: Align::Left,
                ..TextStyle::new(&self.font, 64.0)
            },
            &Paint::color(Color::new(0xff, 0xff, 0xff, 0xff)),
        );

        if self.game_over {
            let style = TextStyle {
                align: Align::Center,
                ..TextStyle::new(&self.font, 128.0)
            };
            let metrics = canvas.measure_text("GAME OVER", &style);

            canvas.fill_text(
                (BOARD_WIDTH * CELL_SIZE / 2) as f32,
                (BOARD_HEIGHT * CELL_SIZE / 2) as f32 + metrics.height / 2.0,
                "GAME OVER",
                &style,
                &Paint::color(Color::new(0xff, 0x00, 0x00, 0xff)),
            );
        }

        canvas.stroke_path(
            &Path::new().rect(
                0.0,
                0.0,
                (BOARD_WIDTH * CELL_SIZE) as f32,
                (BOARD_HEIGHT * CELL_SIZE) as f32,
            ),
            &Stroke {
                width: 8.0,
                ..Default::default()
            },
            &Paint {
                anti_alias: false,
                ..Paint::color(Color::new(0xff, 0xff, 0xff, 0xff))
            },
        );
    }
}
