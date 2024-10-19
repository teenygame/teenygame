/// [`Send`] only if native.
pub trait MaybeSend {}
impl<T> MaybeSend for T {}

/// [`Sync`] only if native.
pub trait MaybeSync {}
impl<T> MaybeSync for T {}
