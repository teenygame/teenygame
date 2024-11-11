//! **teenygame** is a real simple multiplatform game framework for Rust.

const _: () = assert!(
    cfg!(not(all(feature = "smol", feature = "tokio"))),
    "cannot enable both smol and tokio"
);

const _: () = assert!(
    cfg!(any(
        target_arch = "wasm32",
        feature = "smol",
        feature = "tokio"
    )),
    "must enable one of smol or tokio for non-wasm environments"
);

#[cfg(feature = "audio")]
pub mod audio;
pub mod file;
pub mod futures;
pub mod graphics;
pub mod image;
pub mod input;
pub mod math;
pub mod time;

mod marker;

pub use teenygame_macro::game;

#[cfg(feature = "audio")]
use audio::Audio;
use canvasette::Canvas;
use graphics::Graphics;
use input::InputState;
use std::time::Duration;
use time::Instant;
use winit::event::WindowEvent;
use winit::event::{KeyEvent, TouchPhase};
use winit::keyboard::PhysicalKey;

struct GraphicsState {
    canvasette_renderer: canvasette::Renderer,
}

struct Application<G> {
    #[cfg(feature = "audio")]
    audio: Audio,

    input_state: InputState,
    game: G,

    gfx_state: Option<GraphicsState>,

    #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
    tokio_rt: tokio::runtime::Runtime,

    update_ticker: UpdateTicker,
}

struct UpdateTicker {
    tick_interval: Duration,
    current_draw_time: Instant,
    draw_time_accumulator: Duration,
}

impl UpdateTicker {
    fn new(ticks_per_second: u32) -> Self {
        Self {
            tick_interval: Duration::from_secs(1) / ticks_per_second as u32,
            current_draw_time: Instant::now(),
            draw_time_accumulator: Duration::ZERO,
        }
    }

    fn start_draw(&mut self) {
        let new_redraw_time = Instant::now();
        let frame_time = new_redraw_time - self.current_draw_time;
        self.current_draw_time = new_redraw_time;
        self.draw_time_accumulator += frame_time;
    }

    fn tick(&mut self) -> bool {
        if self.draw_time_accumulator < self.tick_interval {
            return false;
        }
        self.draw_time_accumulator -= self.tick_interval;
        true
    }
}

impl<G> wginit::ApplicationHandler for Application<G>
where
    G: Game,
{
    type UserEvent = std::convert::Infallible;

    fn new(_user_event_sender: wginit::UserEventSender<Self::UserEvent>) -> Self {
        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        let tokio_rt = tokio::runtime::Runtime::new().unwrap();

        #[cfg(feature = "audio")]
        let audio = Audio::new().unwrap();

        let input_state = InputState::new();

        Self {
            game: G::new(),
            gfx_state: None,

            #[cfg(feature = "audio")]
            audio,
            input_state,

            #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
            tokio_rt,

            update_ticker: UpdateTicker::new(G::TICKS_PER_SECOND),
        }
    }

    fn resumed(&mut self, ctxt: &wginit::Context) {
        let window = ctxt.window.unwrap();
        let wgpu = ctxt.wgpu.unwrap();

        let canvasette_renderer = canvasette::Renderer::new(
            &wgpu.device,
            wgpu.surface.get_capabilities(&wgpu.adapter).formats[0],
        );

        self.gfx_state = Some(GraphicsState {
            canvasette_renderer,
        });

        let gfx_state = self.gfx_state.as_mut().unwrap();

        self.game.resumed(&mut Context {
            input: &self.input_state,
            #[cfg(feature = "audio")]
            audio: &mut self.audio,
            gfx: &mut Graphics {
                canvasette_renderer: &mut gfx_state.canvasette_renderer,
                wgpu,
                window,
            },
        });
    }

    fn suspended(&mut self, _ctxt: &wginit::Context) {
        self.gfx_state = None;
        self.game.suspended();
    }

    fn window_event(&mut self, _ctxt: &wginit::Context, event: winit::event::WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => match state {
                winit::event::ElementState::Pressed => {
                    self.input_state.keyboard.handle_key_down(key_code);
                }
                winit::event::ElementState::Released => {
                    self.input_state.keyboard.handle_key_up(key_code);
                }
            },
            WindowEvent::MouseInput { state, button, .. } => match state {
                winit::event::ElementState::Pressed => {
                    self.input_state.mouse.handle_button_down(button);
                }
                winit::event::ElementState::Released => {
                    self.input_state.mouse.handle_button_up(button);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.input_state.mouse.set_position(Some(position));
            }
            WindowEvent::CursorLeft { .. } => {
                self.input_state.mouse.set_position(None);
            }
            WindowEvent::Touch(touch) => {
                match touch.phase {
                    TouchPhase::Started => {
                        self.input_state
                            .touch
                            .handle_touch_start(touch.id, touch.location);
                    }
                    TouchPhase::Moved => {
                        self.input_state
                            .touch
                            .handle_touch_move(touch.id, touch.location);
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.input_state.touch.handle_touch_end(touch.id);
                    }
                };
            }
            _ => {}
        };
    }

    fn redraw(&mut self, window: &winit::window::Window, wgpu: &wginit::Wgpu) {
        // Allow use of the Tokio runtime from game callbacks.
        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        let _guard = self.tokio_rt.enter();

        let gfx_state = self.gfx_state.as_mut().unwrap();

        self.update_ticker.start_draw();
        while self.update_ticker.tick() {
            self.game.update(&mut Context {
                input: &self.input_state,
                #[cfg(feature = "audio")]
                audio: &mut self.audio,
                gfx: &mut Graphics {
                    canvasette_renderer: &mut gfx_state.canvasette_renderer,
                    wgpu,
                    window,
                },
            });
            self.input_state.update();
        }

        let mut canvas = Canvas::new();
        self.game.draw(
            &mut Context {
                input: &self.input_state,
                #[cfg(feature = "audio")]
                audio: &mut self.audio,
                gfx: &mut Graphics {
                    canvasette_renderer: &mut gfx_state.canvasette_renderer,
                    wgpu,
                    window,
                },
            },
            &mut canvas,
        );

        let frame = wgpu
            .surface
            .get_current_texture()
            .expect("failed to acquire next swap chain texture");

        graphics::render_to_texture(
            wgpu,
            &mut gfx_state.canvasette_renderer,
            &canvas,
            &frame.texture,
        );

        window.pre_present_notify();
        frame.present();
        window.request_redraw();
    }
}

/// Bag of stuff available to be accessed during [`Game::update`].
pub struct Context<'a> {
    /// Input state.
    pub input: &'a InputState,

    #[cfg(feature = "audio")]
    /// Audio context.
    pub audio: &'a mut Audio,

    /// Graphics context.
    pub gfx: &'a mut Graphics<'a>,
}

/// Trait to implement for your game.
pub trait Game {
    /// How may times [`Game::update`] should be called per second.
    ///
    /// Defaults to 60.
    const TICKS_PER_SECOND: u32 = 60;

    /// Constructs the game.
    ///
    /// If Tokio support is enabled, the Tokio runtime will be available here.
    fn new() -> Self;

    /// The game was resumed (e.g. this is now the foreground app).
    fn resumed(&mut self, ctxt: &mut Context) {
        _ = ctxt;
    }

    /// The game was suspended (e.g. this is no longer the foreground app).
    fn suspended(&mut self) {}

    /// Updates the game state [`Game::TICKS_PER_SECOND`] per second.
    ///
    /// This may be called multiple times between calls to [`Game::draw`], depending on the time elapsed. This implements the [fix your timestep](https://gafferongames.com/post/fix_your_timestep/) pattern internally.
    ///
    /// You may not perform any drawing in this function.
    fn update(&mut self, ctxt: &mut Context);

    /// Draws the game state.
    fn draw<'a>(&'a mut self, ctxt: &mut Context, canvas: &mut Canvas<'a>);
}

/// Runs the game.
///
/// This should be the only function called in your `main`. It will:
/// - Set up logging (and panic handling for WASM).
/// - Create the event loop.
/// - If enabled and running on a native platform, start the Tokio runtime.
/// - Starts the event loop and hands over control.
///
/// You can either manually call this in your `main` function, or you can annotate your `Game` struct with the [`game`] macro.
pub fn run<G>()
where
    G: Game,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        wasm_logger::init(wasm_logger::Config::default());
    }

    wginit::run::<Application<G>>().unwrap();
}
