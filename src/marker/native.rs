/// [`Send`] only if native.
pub trait MaybeSend
where
    Self: Send,
{
}
impl<T> MaybeSend for T where T: Send {}

/// [`Sync`] only if native.
pub trait MaybeSync
where
    Self: Sync,
{
}
impl<T> MaybeSync for T where T: Sync {}
