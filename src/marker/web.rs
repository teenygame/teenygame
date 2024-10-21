/// [`Send`] only if native.
pub trait WasmNotSend {}
impl<T> WasmNotSend for T {}

/// [`Sync`] only if native.
pub trait WasmNotSync {}
impl<T> WasmNotSync for T {}
