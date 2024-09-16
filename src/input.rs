use std::collections::HashSet;

pub use winit::keyboard::KeyCode;
use winit::{dpi::PhysicalPosition, event::MouseButton};

pub struct Keyboard {
    last_keys_held: HashSet<KeyCode>,
    keys_held: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            last_keys_held: HashSet::new(),
            keys_held: HashSet::new(),
        }
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        !self.last_keys_held.contains(&key) && self.keys_held.contains(&key)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.last_keys_held.contains(&key) && !self.keys_held.contains(&key)
    }

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

pub struct InputState {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl InputState {
    pub fn new() -> Self {
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

pub struct Mouse {
    last_mouse_buttons_held: HashSet<MouseButton>,
    mouse_buttons_held: HashSet<MouseButton>,
    pos: Option<PhysicalPosition<f64>>,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            last_mouse_buttons_held: HashSet::new(),
            mouse_buttons_held: HashSet::new(),
            pos: None,
        }
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        !self.last_mouse_buttons_held.contains(&button) && self.mouse_buttons_held.contains(&button)
    }

    pub fn is_button_released(&self, button: MouseButton) -> bool {
        self.last_mouse_buttons_held.contains(&button) && !self.mouse_buttons_held.contains(&button)
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons_held.contains(&button)
    }

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
