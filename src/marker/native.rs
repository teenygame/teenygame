/// [`Send`] only if native.
pub trait WasmNotSend
where
    Self: Send,
{
}
impl<T> WasmNotSend for T where T: Send {}

/// [`Sync`] only if native.
pub trait WasmNotSync
where
    Self: Sync,
{
}
impl<T> WasmNotSync for T where T: Sync {}
