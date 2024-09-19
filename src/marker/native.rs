pub trait MaybeSend: Send {}
impl<T> MaybeSend for T where T: Send {}

pub trait MaybeSync: Sync {}
impl<T> MaybeSync for T where T: Sync {}
