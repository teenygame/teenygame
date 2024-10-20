mod game;

pub use game::Game;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn init() {
    use teenygame::run;
    run::<Game>();
}
