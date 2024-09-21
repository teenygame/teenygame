use std::future::Future;

use crate::marker::MaybeSend;

pub fn spawn(fut: impl Future<Output = ()> + MaybeSend + 'static) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
    tokio::task::spawn(fut);

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(fut);
}
