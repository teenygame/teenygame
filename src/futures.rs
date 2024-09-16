use std::future::Future;

#[cfg(any(
    all(not(target_arch = "wasm32"), feature = "tokio"),
    target_arch = "wasm32"
))]
pub fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
    {
        tokio::task::spawn(fut);
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(fut);
    }
}
