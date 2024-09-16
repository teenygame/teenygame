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

#[derive(thiserror::Error, Debug, Clone)]
#[error("{0}")]
pub struct Error(Arc<Box<dyn std::error::Error + Send + Sync>>);

type AssetState<T> = Option<Result<Arc<T>, Error>>;

impl<T> Asset<T> {
    fn pending() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn get(&self) -> AssetState<T> {
        self.0.lock().unwrap().clone()
    }
}

trait AnyAsset {
    fn status(&self) -> Option<Result<(), Error>>;
}

impl<T> AnyAsset for Asset<T> {
    fn status(&self) -> Option<Result<(), Error>> {
        self.0
            .lock()
            .unwrap()
            .as_ref()
            .map(|r| r.as_ref().map(|_| ()).map_err(|e| e.clone()))
    }
}

pub struct AssetLoadTracker {
    assets: Vec<Box<dyn AnyAsset>>,
}

impl AssetLoadTracker {
    pub fn new() -> Self {
        Self { assets: vec![] }
    }

    pub fn add<T>(&mut self, asset: &Asset<T>)
    where
        Asset<T>: 'static,
    {
        self.assets.push(Box::new(asset.clone()));
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn num_loaded(&self) -> Result<usize, Error> {
        let mut n = 0;
        for asset in self.assets.iter() {
            let Some(status) = asset.status() else {
                continue;
            };
            if let Err(err) = status {
                return Err(err);
            }
            n += 1
        }
        Ok(n)
    }
}
