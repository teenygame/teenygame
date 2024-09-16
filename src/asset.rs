#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
mod native;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;

use std::sync::{Arc, Mutex};

pub struct Asset<T>(Arc<Mutex<AssetState<T>>>);

impl<T> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

type AssetState<T> = Option<Result<Arc<T>, Arc<Error>>>;

impl<T> Asset<T> {
    fn pending() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn get(&self) -> AssetState<T> {
        self.0.lock().unwrap().clone()
    }
}

trait AnyAsset {
    fn is_loaded(&self) -> bool;
}

impl<T> AnyAsset for Asset<T> {
    fn is_loaded(&self) -> bool {
        self.0.lock().unwrap().is_some()
    }
}

pub struct ResourceGroup {
    resources: Vec<Box<dyn AnyAsset>>,
}

impl ResourceGroup {
    pub fn new() -> Self {
        Self { resources: vec![] }
    }

    pub fn add<T>(&mut self, r: Asset<T>) -> Asset<T>
    where
        Asset<T>: 'static,
    {
        self.resources.push(Box::new(r.clone()));
        r
    }

    pub fn is_loaded(&self) -> bool {
        self.resources.iter().all(|r| r.is_loaded())
    }
}
