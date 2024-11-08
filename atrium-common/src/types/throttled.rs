use std::sync::Arc;

pub trait Throttleable<P>
where
    Self: std::marker::Sized,
{
    fn throttled(self) -> Throttled<Self, P>;
}

impl<P, T> Throttleable<P> for T
where
    P: Default,
{
    fn throttled(self) -> Throttled<Self, P> {
        Throttled::new(self)
    }
}

pub struct Throttled<T, P> {
    inner: T,
    pending: Arc<P>,
}

impl<T, P> Throttled<T, P>
where
    P: Default,
{
    pub fn new(inner: T) -> Self {
        Self { inner, pending: Arc::new(P::default()) }
    }
}
