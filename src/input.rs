//! Input handling.

use std::collections::HashSet;

use winit::dpi::PhysicalPosition;
pub use winit::{event::MouseButton, keyboard::KeyCode};

/// Keyboard state.
pub struct Keyboard {
    last_keys_held: HashSet<KeyCode>,
    keys_held: HashSet<KeyCode>,
}

impl Keyboard {
    fn new() -> Self {
        Self {
            last_keys_held: HashSet::new(),
            keys_held: HashSet::new(),
        }
    }

    /// Checks if the key was just pressed.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        !self.last_keys_held.contains(&key) && self.keys_held.contains(&key)
    }

    /// Checks if the key was just released.
    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.last_keys_held.contains(&key) && !self.keys_held.contains(&key)
    }

    /// Checks if the key is currently being held down.
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.keys_held.contains(&key)
    }

    pub(crate) fn handle_key_up(&mut self, key: KeyCode) {
        self.keys_held.remove(&key);
    }

    pub(crate) fn handle_key_down(&mut self, key: KeyCode) {
        self.keys_held.insert(key);
    }

    pub(crate) fn update(&mut self) {
        self.last_keys_held.clone_from(&self.keys_held);
    }
}

/// Keeps track of the current state of input devices.
pub struct InputState {
    /// Keyboard state.
    pub keyboard: Keyboard,

    /// Mouse state.
    pub mouse: Mouse,
}

impl InputState {
    pub(crate) fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
        }
    }

    pub(crate) fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
    }
}

/// Mouse state.
pub struct Mouse {
    last_mouse_buttons_held: HashSet<MouseButton>,
    mouse_buttons_held: HashSet<MouseButton>,
    pos: Option<PhysicalPosition<f64>>,
}

impl Mouse {
    fn new() -> Self {
        Self {
            last_mouse_buttons_held: HashSet::new(),
            mouse_buttons_held: HashSet::new(),
            pos: None,
        }
    }

    /// Checks if the button was just pressed.
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        !self.last_mouse_buttons_held.contains(&button) && self.mouse_buttons_held.contains(&button)
    }

    /// Checks if the button was just released.
    pub fn is_button_released(&self, button: MouseButton) -> bool {
        self.last_mouse_buttons_held.contains(&button) && !self.mouse_buttons_held.contains(&button)
    }

    /// Checks if the button is being held down.
    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons_held.contains(&button)
    }

    /// Gets the current position of the mouse.
    ///
    /// May return [`None`] if the mouse is not within the confines of the window.
    pub fn position(&self) -> Option<(f64, f64)> {
        self.pos.map(|pos| pos.into())
    }

    pub(crate) fn handle_button_up(&mut self, button: MouseButton) {
        self.mouse_buttons_held.remove(&button);
    }

    pub(crate) fn handle_button_down(&mut self, button: MouseButton) {
        self.mouse_buttons_held.insert(button);
    }

    pub(crate) fn set_position(&mut self, pos: Option<PhysicalPosition<f64>>) {
        self.pos = pos;
    }

    pub(crate) fn update(&mut self) {
        self.last_mouse_buttons_held
            .clone_from(&self.mouse_buttons_held);
    }
}
