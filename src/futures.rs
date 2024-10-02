//! Utilities for dealing with futures.

use std::future::Future;

use crate::marker::MaybeSend;

/// Spawns a future.
///
/// - On native platforms, the future must be [`Send`] as it will be spawned on another thread.
/// - On WASM, the future does not need to be [`Send`] as it will be spawned on the same thread.
pub fn spawn(fut: impl Future<Output = ()> + MaybeSend + 'static) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
    {
        tokio::task::spawn(fut);
        return;
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "smol"))]
    {
        smol::spawn(fut).detach();
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(fut);
        return;
    }

    #[allow(unreachable_code)]
    {
        _ = fut;
        panic!("no executor available to spawn futures on!");
    }
}
