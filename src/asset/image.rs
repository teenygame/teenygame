#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
mod native;

use std::sync::Arc;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

pub struct ImageAndMetadata<M> {
    pub image: Arc<Image>,
    pub metadata: M,
}
