use concread::{cowcell::CowCellReadTxn, CowCell};
use std::sync::Arc;

pub struct Resource<T>(Arc<CowCell<ResourceState<T>>>);

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

type ResourceState<T> = Option<Arc<Result<T, Box<dyn std::error::Error + Send + Sync>>>>;

impl<T> Resource<T> {
    fn empty() -> Self {
        Self(Arc::new(CowCell::new(None)))
    }

    pub fn get(&self) -> CowCellReadTxn<ResourceState<T>> {
        self.0.read()
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub fn load_bytes(path: &str) -> Resource<Vec<u8>> {
    let r = Resource::empty();
    {
        let path = path.to_string();
        let r = r.clone();

        tokio::spawn(async move {
            let res = tokio::fs::read(path).await;
            let mut txn = r.0.write();
            *txn = Some(Arc::new(res.map_err(|e| Box::new(e).into())));
            txn.commit()
        });
    }
    r
}

#[cfg(target_arch = "wasm32")]
pub fn load_bytes(path: &str) -> Resource<Vec<u8>> {
    let r = Resource::empty();
    {
        let path = path.to_string();
        let r = r.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let res = async {
                Ok(gloo_net::http::Request::get(&path)
                    .send()
                    .await?
                    .binary()
                    .await?)
            }
            .await;
            let mut txn = r.0.write();
            *txn = Some(Arc::new(res));
            txn.commit()
        });
    }
    r
}
