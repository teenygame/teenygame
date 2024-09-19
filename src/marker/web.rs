pub trait MaybeSend {}
impl<T> MaybeSend for T {}

pub trait MaybeSync {}
impl<T> MaybeSync for T {}
