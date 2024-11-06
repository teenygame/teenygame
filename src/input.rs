//! Input handling.

use crate::math;
use std::collections::{HashMap, HashSet};
use winit::dpi::PhysicalPosition;
pub use winit::{event::MouseButton, keyboard::KeyCode};

/// A contact point.
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Contact(u64);

/// Touch state.
pub struct Touch {
    last_held_contacts: HashMap<Contact, PhysicalPosition<f64>>,
    held_contacts: HashMap<Contact, PhysicalPosition<f64>>,
    next_contact_id: u64,
    ids_to_contacts: HashMap<u64, Contact>,
}

impl Touch {
    fn new() -> Self {
        Self {
            held_contacts: HashMap::new(),
            last_held_contacts: HashMap::new(),
            next_contact_id: 0,
            ids_to_contacts: HashMap::new(),
        }
    }

    fn next_contact(&mut self) -> Contact {
        let contact = Contact(self.next_contact_id);
        self.next_contact_id += 1;
        contact
    }

    /// Iterates over all held contacts.
    pub fn held_contacts(&self) -> impl Iterator<Item = (Contact, math::Vec2)> + '_ {
        self.held_contacts
            .iter()
            .map(|(contact, pos)| (*contact, math::Vec2::new(pos.x as f32, pos.y as f32)))
    }

    /// Iterates over all contacts that were just pressed.
    pub fn pressed_contacts(&self) -> impl Iterator<Item = (Contact, math::Vec2)> + '_ {
        self.held_contacts.iter().filter_map(|(contact, pos)| {
            if self.last_held_contacts.contains_key(contact) {
                Some((*contact, math::Vec2::new(pos.x as f32, pos.y as f32)))
            } else {
                None
            }
        })
    }

    /// Iterates over all contacts that were just released.
    pub fn released_contacts(&self) -> impl Iterator<Item = Contact> + '_ {
        self.last_held_contacts
            .keys()
            .cloned()
            .filter(|contact| !self.held_contacts.contains_key(contact))
    }

    /// Gets the position of a contact, or None if the contact was already lifted from the screen.
    pub fn contact_position(&self, contact: Contact) -> Option<math::Vec2> {
        self.held_contacts
            .get(&contact)
            .map(|pos| math::Vec2::new(pos.x as f32, pos.y as f32))
    }

    pub(crate) fn handle_touch_start(&mut self, id: u64, location: PhysicalPosition<f64>) {
        let contact = self.next_contact();
        self.ids_to_contacts.insert(id, contact);
        self.held_contacts.insert(contact, location);
    }

    pub(crate) fn handle_touch_move(&mut self, id: u64, location: PhysicalPosition<f64>) {
        self.held_contacts
            .insert(self.ids_to_contacts[&id], location);
    }

    pub(crate) fn handle_touch_end(&mut self, id: u64) {
        self.held_contacts.remove(&self.ids_to_contacts[&id]);
        self.ids_to_contacts.remove(&id);
    }

    fn update(&mut self) {
        self.last_held_contacts.clone_from(&self.held_contacts);
    }
}

/// Keyboard state.
pub struct Keyboard {
    held_keys: HashSet<KeyCode>,
    last_held_keys: HashSet<KeyCode>,
}

impl Keyboard {
    fn new() -> Self {
        Self {
            held_keys: HashSet::new(),
            last_held_keys: HashSet::new(),
        }
    }

    /// Iterates over all held keys.
    pub fn held_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.held_keys.iter().cloned()
    }

    /// Iterates over all keys that were just pressed.
    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.held_keys
            .iter()
            .cloned()
            .filter(move |contact| !self.last_held_keys.contains(contact))
    }

    /// Iterates over all keys that were just released.
    pub fn released_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.last_held_keys
            .iter()
            .cloned()
            .filter(move |contact| !self.held_keys.contains(contact))
    }

    /// Checks if the key was just pressed.
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        !self.last_held_keys.contains(&key) && self.held_keys.contains(&key)
    }

    /// Checks if the key was just released.
    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.last_held_keys.contains(&key) && !self.held_keys.contains(&key)
    }

    /// Checks if the key is currently being held down.
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.held_keys.contains(&key)
    }

    pub(crate) fn handle_key_up(&mut self, key: KeyCode) {
        self.held_keys.remove(&key);
    }

    pub(crate) fn handle_key_down(&mut self, key: KeyCode) {
        self.held_keys.insert(key);
    }

    fn update(&mut self) {
        self.last_held_keys.clone_from(&self.held_keys);
    }
}

/// Keeps track of the current state of input devices.
pub struct InputState {
    /// Keyboard state.
    pub keyboard: Keyboard,

    /// Mouse state.
    pub mouse: Mouse,

    /// Touch state.
    pub touch: Touch,
}

impl InputState {
    pub(crate) fn new() -> Self {
        Self {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            touch: Touch::new(),
        }
    }

    pub(crate) fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
        self.touch.update();
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
    pub fn position(&self) -> Option<math::Vec2> {
        self.pos
            .map(|pos| math::Vec2::new(pos.x as f32, pos.y as f32))
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

    fn update(&mut self) {
        self.last_mouse_buttons_held
            .clone_from(&self.mouse_buttons_held);
    }
}
