//! **teenygame** is a real simple multiplatform game framework for Rust. It currently supports the following platforms:
//! - Windows
//! - Linux
//! - macOS
//! - Web
//!
//! Mobile support might work but no promises!

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
pub mod input;
pub mod marker;
pub mod time;

#[cfg(feature = "audio")]
use audio::AudioContext;
use graphics::{Canvas, GraphicsState};
use input::InputState;
use std::time::Duration;
use time::Instant;
use winit::dpi::PhysicalSize;
use winit::event::KeyEvent;
use winit::keyboard::PhysicalKey;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopProxy},
};

enum UserEvent {
    GraphicsState(GraphicsState),
}

struct Application<G> {
    #[cfg(feature = "audio")]
    audio: AudioContext,

    gfx: Option<GraphicsState>,

    event_loop_proxy: EventLoopProxy<UserEvent>,
    input_state: InputState,
    game: Option<G>,

    #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
    tokio_rt: tokio::runtime::Runtime,

    update_ticker: UpdateTicker,
}

impl<G> Application<G>
where
    G: Game,
{
    fn new(event_loop: &EventLoop<UserEvent>) -> Self {
        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        let tokio_rt = tokio::runtime::Runtime::new().unwrap();

        Self {
            gfx: None,

            #[cfg(feature = "audio")]
            audio: AudioContext::new().unwrap(),

            event_loop_proxy: event_loop.create_proxy(),
            input_state: InputState::new(),
            game: None,

            #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
            tokio_rt,

            update_ticker: UpdateTicker::new(G::TICKS_PER_SECOND),
        }
    }
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

impl<G> ApplicationHandler<UserEvent> for Application<G>
where
    G: Game,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let gfx = GraphicsState::new(self.gfx.take(), event_loop);

        let event_loop_proxy = self.event_loop_proxy.clone();
        assert!(event_loop_proxy
            .send_event(UserEvent::GraphicsState(gfx))
            .is_ok());
    }

    fn suspended(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.suspend();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(gfx) = &mut self.gfx else {
            return;
        };

        match event {
            WindowEvent::Resized(size) => {
                gfx.resize(size);
            }
            WindowEvent::RedrawRequested => {
                // Allow use of the Tokio runtime from game callbacks.
                #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
                let _guard = self.tokio_rt.enter();

                let Some(game) = self.game.as_mut() else {
                    return;
                };

                self.update_ticker.start_draw();
                while self.update_ticker.tick() {
                    game.update(&mut UpdateContext {
                        input: &self.input_state,
                        #[cfg(feature = "audio")]
                        audio: &mut self.audio,
                        canvas: &gfx.canvas,
                        window: Window(&gfx.window),
                    });
                    self.input_state.update();
                }

                game.draw(&mut gfx.canvas);

                gfx.flush_and_swap_buffers();
                gfx.window.request_redraw();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
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
            _ => {}
        };
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::GraphicsState(gfx) => {
                #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
                let _guard = self.tokio_rt.enter();
                self.game = Some(G::new(Window(&gfx.window)));
                self.gfx = Some(gfx);
            }
        }
    }
}

/// Bag of stuff available to be accessed during [`Game::update`].
pub struct UpdateContext<'a> {
    /// Input state.
    pub input: &'a InputState,

    #[cfg(feature = "audio")]
    /// Audio context.
    pub audio: &'a mut AudioContext,

    /// Window.
    pub window: Window<'a>,

    canvas: &'a Canvas,
}

impl<'a> UpdateContext<'a> {
    /// Current size of the canvas.
    pub fn canvas_size(&self) -> (u32, u32) {
        self.canvas.size()
    }
}

/// Window.
pub struct Window<'a>(&'a winit::window::Window);

impl<'a> Window<'a> {
    /// Sets the title of the window.
    pub fn set_title(&self, title: &str) {
        self.0.set_title(title);
    }

    /// Requests the size of the window to be a given size.
    pub fn set_size(&self, width: u32, height: u32, resizable: bool) {
        self.0.set_resizable(resizable);
        let _ = self.0.request_inner_size(PhysicalSize::new(width, height));
    }
}

/// Trait to implement for your game.
pub trait Game {
    /// How may times [`Game::update`] should be called per second.
    ///
    /// Defaults to 60.
    const TICKS_PER_SECOND: u32 = 60;

    /// Constructs your game.
    ///
    /// If Tokio support is enabled, the Tokio runtime will be available here.
    fn new(window: Window) -> Self;

    /// Updates the game state [`Game::TICKS_PER_SECOND`] per second.
    ///
    /// This may be called multiple times between calls to [`Game::draw`], depending on the time elapsed. This implements the [fix your timestep](https://gafferongames.com/post/fix_your_timestep/) pattern internally.
    ///
    /// You may not perform any drawing in this function.
    fn update(&mut self, s: &mut UpdateContext);

    /// Draws the game state to the canvas.
    fn draw(&mut self, canvas: &mut Canvas);
}

/// Runs the game.
///
/// This should be the only function called in your `main`. It will:
/// - Set up logging (and panic handling for WASM).
/// - Create the event loop.
/// - If enabled and running on a native platform, start the Tokio runtime.
/// - Starts the event loop and hands over control.
///
/// For example, on native platforms:
/// ```
/// fn main() { run::<Game>(); }
/// ```
///
/// And on WASM:
/// ```
/// #[wasm_bindgen::prelude::wasm_bindgen]
/// pub fn init() { run::<Game>(); }
/// ```
///
/// Easy!
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

    let event_loop = winit::event_loop::EventLoop::with_user_event()
        .build()
        .unwrap();
    let mut app = Application::<G>::new(&event_loop);
    event_loop.run_app(&mut app).unwrap();
}
