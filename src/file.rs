#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
mod native;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not found")]
    NotFound,

    #[error("other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}
