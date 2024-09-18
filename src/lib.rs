pub mod asset;
#[cfg(feature = "audio")]
pub mod audio;
pub mod gfx;
pub mod input;
pub mod time;

#[cfg(feature = "audio")]
use audio::AudioContext;
use gfx::{Canvas, GraphicsState};
use input::InputState;
use std::time::Duration;
use time::Instant;
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

pub struct Application<G> {
    #[cfg(feature = "audio")]
    audio: AudioContext,

    gfx: Option<GraphicsState>,

    event_loop_proxy: EventLoopProxy<UserEvent>,
    input_state: InputState,
    game: Option<G>,

    #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
    tokio_rt: tokio::runtime::Runtime,

    current_draw_time: Instant,
    draw_time_accumulator: Duration,
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

            current_draw_time: Instant::now(),
            draw_time_accumulator: Duration::ZERO,
        }
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

                let new_redraw_time = Instant::now();
                let frame_time = new_redraw_time - self.current_draw_time;
                self.current_draw_time = new_redraw_time;

                self.draw_time_accumulator += frame_time;

                while self.draw_time_accumulator >= G::TICK_TIME {
                    game.update(&mut Context {
                        input: &self.input_state,
                        #[cfg(feature = "audio")]
                        audio: &self.audio,
                        canvas: &mut gfx.canvas,
                    });
                    self.input_state.update();
                    self.draw_time_accumulator -= G::TICK_TIME;
                }

                game.draw(&mut gfx.canvas);

                gfx.flush_and_swap_buffers();
                gfx.canvas.gc();
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
            UserEvent::GraphicsState(mut gfx) => {
                #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
                let _guard = self.tokio_rt.enter();

                self.game = Some(G::new(&mut Context {
                    input: &self.input_state,
                    #[cfg(feature = "audio")]
                    audio: &self.audio,
                    canvas: &mut gfx.canvas,
                }));

                self.gfx = Some(gfx);
            }
        }
    }
}

pub struct Context<'a> {
    pub input: &'a InputState,
    pub canvas: &'a mut Canvas,
    #[cfg(feature = "audio")]
    pub audio: &'a AudioContext,
}

pub trait Game {
    const TICK_TIME: Duration = Duration::from_millis(1000 / 60);

    fn new(cx: &mut Context) -> Self;
    fn update(&mut self, cx: &mut Context);
    fn draw(&mut self, cx: &mut Canvas);
}

pub fn run<G>()
where
    G: Game,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    };

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
