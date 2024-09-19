mod image;
mod raw;

pub use image::*;
pub use raw::*;

use std::{
    future::Future,
    sync::{Arc, Mutex},
};

pub struct Asset<T>(Arc<Mutex<AssetState<T>>>);

impl<T> Clone for Asset<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(target_arch = "wasm32")]
pub trait Loadable
where
    Self: Sized + 'static,
{
    fn load(path: &str) -> impl Future<Output = Result<Self, anyhow::Error>>;
}

#[cfg(not(target_arch = "wasm32"))]
pub trait Loadable
where
    Self: Sized + Send + Sync + 'static,
{
    fn load(path: &str) -> impl Future<Output = Result<Self, anyhow::Error>> + Send;
}

pub fn load<T>(path: &str) -> Asset<T>
where
    T: Loadable,
{
    let r = Asset::pending();
    {
        let path = path.to_string();
        let r = r.clone();

        let fut = async move {
            let res = T::load(&path)
                .await
                .map_err(|e| Arc::new(e))
                .map(|v| Arc::new(v));
            *r.0.lock().unwrap() = Some(res);
        };

        #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
        tokio::task::spawn(fut);

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(fut);
    }
    r
}

type Error = Arc<anyhow::Error>;

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
        self.assets.iter().try_fold(0, |acc, x| {
            Ok(acc
                + match x.status() {
                    Some(Ok(())) => 1,
                    Some(Err(e)) => {
                        return Err(e);
                    }
                    None => 0,
                })
        })
    }
}

pub trait Metadata
where
    Self: Sized,
{
    fn load(raw: &[u8]) -> Result<Self, anyhow::Error>;
}
