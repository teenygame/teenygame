pub trait ConditionalSend {}
impl<T> ConditionalSend for T {}

pub trait ConditionalSync {}
impl<T> ConditionalSync for T {}
