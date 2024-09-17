#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod web;

pub struct Raw(pub Vec<u8>);
