#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
mod native;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

use concread::{cowcell::CowCellReadTxn, CowCell};
use std::sync::Arc;

pub struct Resource<T>(Arc<CowCell<ResourceState<T>>>);

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

type ResourceState<T> = Option<Result<Arc<T>, Arc<Box<dyn std::error::Error + Send + Sync>>>>;

impl<T> Resource<T> {
    fn pending() -> Self {
        Self(Arc::new(CowCell::new(None)))
    }

    pub fn get(&self) -> CowCellReadTxn<ResourceState<T>> {
        self.0.read()
    }
}
