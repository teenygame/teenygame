use rand::prelude::IteratorRandom;
use std::collections::VecDeque;
use teenygame::{
    audio::{PlaybackHandle, Region, Sound, Source},
    graphics::{
        font::{Attrs, Metrics},
        AffineTransform, Color, Texture,
    },
    input::KeyCode,
    Context,
};

const BOARD_WIDTH: usize = 40;
const BOARD_HEIGHT: usize = 22;

const CELL_SIZE: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Fruit,
    Snake,
}

type Direction = (isize, isize);

const NORTH: Direction = (0, -1);
const SOUTH: Direction = (0, 1);
const EAST: Direction = (1, 0);
const WEST: Direction = (-1, 0);

pub struct Game {
    texture: Texture,
    pickup_sfx: Source,
    game_over_sfx: Source,
    bgm_handle: Option<PlaybackHandle>,
    game_over: bool,
    board: [[Option<Cell>; BOARD_WIDTH]; BOARD_HEIGHT],
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
        window.set_size(
            (BOARD_WIDTH * CELL_SIZE) as u32,
            (BOARD_HEIGHT * CELL_SIZE) as u32,
            true,
        );

        let mut board = [[None; BOARD_WIDTH]; BOARD_HEIGHT];
        let snake = VecDeque::from([(BOARD_WIDTH / 2, BOARD_HEIGHT / 2)]);

        for (x, y) in snake.iter() {
            board[*y][*x] = Some(Cell::Snake);
        }

        ctxt.gfx.add_font(include_bytes!("PixelOperator.ttf"));

        let bgm_source = Source::load(include_bytes!("8BitCave.wav")).unwrap();

        let mut game = Self {
            texture: ctxt.gfx.load_texture(teenygame::graphics::ImgRef::new(
                &[
                    Color::new(0xff, 0xff, 0xff, 0xff),
                    Color::new(0xff, 0x00, 0x00, 0xff),
                ],
                2,
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

        let (hx, hy) = *self.snake.back().unwrap();
        let (dx, dy) = self.direction;

        let hx2 = (hx as isize + dx).rem_euclid(BOARD_WIDTH as isize) as usize;
        let hy2 = (hy as isize + dy).rem_euclid(BOARD_HEIGHT as isize) as usize;

        self.snake.push_back((hx2, hy2));

        match self.board[hy2][hx2] {
            None => {
                let (ohx, ohy) = self.snake.pop_front().unwrap();
                self.board[ohy][ohx] = None;
            }
            Some(Cell::Fruit) => {
                self.spawn_fruit();
                self.score += 1;
                if let Some(handle) = &mut self.bgm_handle {
                    handle.set_speed((self.score as f64 + 1.0).powf(0.02), Default::default());
                }
                ctxt.audio.play(&Sound::new(&self.pickup_sfx)).detach();
            }
            Some(Cell::Snake) => {
                self.game_over = true;
                ctxt.audio.play(&Sound::new(&self.game_over_sfx)).detach();
                self.bgm_handle.take();
            }
        }
        self.board[hy2][hx2] = Some(Cell::Snake);
        self.elapsed = 0;
    }

    fn draw<'a>(
        &'a mut self,
        ctxt: &mut teenygame::Context,
        scene: &mut teenygame::graphics::Scene<'a>,
    ) {
        let window = ctxt.gfx.window();
        let [width, height] = window.size();

        let scene = scene.add_child(AffineTransform::translation(
            (width as i32 / 2 - (BOARD_WIDTH * CELL_SIZE) as i32 / 2) as f32,
            (height as i32 / 2 - (BOARD_HEIGHT * CELL_SIZE) as i32 / 2) as f32,
        ));

        for (y, row) in self.board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                match cell {
                    None => {}
                    Some(Cell::Fruit) => {
                        scene.draw_sprite(
                            &self.texture,
                            1.0,
                            0.0,
                            1.0,
                            1.0,
                            (x * CELL_SIZE) as f32,
                            (y * CELL_SIZE) as f32,
                            CELL_SIZE as f32,
                            CELL_SIZE as f32,
                        );
                    }
                    Some(Cell::Snake) => {
                        scene.draw_sprite(
                            &self.texture,
                            0.0,
                            0.0,
                            1.0,
                            1.0,
                            (x * CELL_SIZE) as f32,
                            (y * CELL_SIZE) as f32,
                            CELL_SIZE as f32,
                            CELL_SIZE as f32,
                        );
                    }
                }
            }
        }

        scene.draw_text(
            ctxt.gfx.prepare_text(
                format!("Score: {}", self.score),
                Metrics::relative(64.0, 1.0),
                Attrs::default(),
            ),
            16.0,
            56.0,
            Color::new(0xff, 0xff, 0xff, 0xff),
        );

        if self.game_over {
            let prepared_game_over =
                ctxt.gfx
                    .prepare_text("GAME OVER", Metrics::relative(128.0, 1.0), Attrs::default());
            let [w, h] = prepared_game_over.bounding_box();
            scene.draw_text(
                prepared_game_over,
                (BOARD_WIDTH * CELL_SIZE / 2) as f32 - w / 2.0,
                (BOARD_HEIGHT * CELL_SIZE / 2) as f32 + h / 2.0,
                Color::new(0xff, 0x00, 0x00, 0xff),
            );
        }

        // Draw a border by using the 1x1 white pixel in our texture.
        scene.draw_sprite(
            &self.texture,
            0.0,
            0.0,
            1.0,
            1.0,
            -8.0,
            -8.0,
            (BOARD_WIDTH * CELL_SIZE + 12) as f32,
            8.0,
        );
        scene.draw_sprite(
            &self.texture,
            0.0,
            0.0,
            1.0,
            0.0,
            -8.0,
            (BOARD_HEIGHT * CELL_SIZE) as f32,
            (BOARD_WIDTH * CELL_SIZE + 12) as f32,
            8.0,
        );
        scene.draw_sprite(
            &self.texture,
            0.0,
            0.0,
            1.0,
            1.0,
            -8.0,
            -8.0,
            8.0,
            (BOARD_HEIGHT * CELL_SIZE + 12) as f32,
        );
        scene.draw_sprite(
            &self.texture,
            0.0,
            0.0,
            1.0,
            1.0,
            (BOARD_WIDTH * CELL_SIZE) as f32,
            -8.0,
            8.0,
            (BOARD_HEIGHT * CELL_SIZE + 12) as f32,
        );
    }
}
