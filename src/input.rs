//! Input handling.

use std::collections::{HashMap, HashSet};
use winit::dpi::PhysicalPosition;
pub use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Contact(u64);

/// Touch state.
pub struct Touch {
    last_contacts: HashMap<Contact, PhysicalPosition<f64>>,
    contacts: HashMap<Contact, PhysicalPosition<f64>>,
    next_contact_id: u64,
    ids_to_contacts: HashMap<u64, Contact>,
}

impl Touch {
    fn new() -> Self {
        Self {
            contacts: HashMap::new(),
            last_contacts: HashMap::new(),
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
    pub fn held_contacts(&self) -> impl Iterator<Item = Contact> + '_ {
        self.contacts.keys().cloned()
    }

    /// Iterates over all contacts that were just pressed.
    pub fn pressed_contacts(&self) -> impl Iterator<Item = Contact> + '_ {
        let last_contacts = self.last_contacts.keys().cloned().collect::<HashSet<_>>();
        self.contacts
            .keys()
            .cloned()
            .filter(move |contact| !last_contacts.contains(contact))
    }

    /// Iterates over all contacts that were just released.
    pub fn released_contacts(&self) -> impl Iterator<Item = Contact> + '_ {
        let contacts = self.contacts.keys().cloned().collect::<HashSet<_>>();
        self.last_contacts
            .keys()
            .cloned()
            .filter(move |contact| !contacts.contains(contact))
    }

    /// Gets the position of a contact, or None if the contact was already lifted from the screen.
    pub fn contact_position(&self, contact: Contact) -> Option<(f64, f64)> {
        self.contacts.get(&contact).map(|v| (*v).into())
    }

    pub(crate) fn handle_touch_start(&mut self, id: u64, location: PhysicalPosition<f64>) {
        let contact = self.next_contact();
        self.ids_to_contacts.insert(id, contact);
        self.contacts.insert(contact, location);
    }

    pub(crate) fn handle_touch_move(&mut self, id: u64, location: PhysicalPosition<f64>) {
        self.contacts.insert(self.ids_to_contacts[&id], location);
    }

    pub(crate) fn handle_touch_end(&mut self, id: u64) {
        self.contacts.remove(&self.ids_to_contacts[&id]);
        self.ids_to_contacts.remove(&id);
    }

    fn update(&mut self) {
        self.last_contacts.clone_from(&self.contacts);
    }
}

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

    fn update(&mut self) {
        self.last_keys_held.clone_from(&self.keys_held);
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

    fn update(&mut self) {
        self.last_mouse_buttons_held
            .clone_from(&self.mouse_buttons_held);
    }
}
