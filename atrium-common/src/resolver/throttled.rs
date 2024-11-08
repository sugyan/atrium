use std::{hash::Hash, sync::Arc};

use dashmap::{DashMap, Entry};
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Mutex;

use crate::types::throttled::Throttled;

use super::Resolver;

pub type SenderMap<R> =
    DashMap<<R as Resolver>::Input, Arc<Mutex<Sender<Option<<R as Resolver>::Output>>>>>;

pub type ThrottledResolver<R> = Throttled<R, SenderMap<R>>;

impl<R> Resolver for Throttled<R, SenderMap<R>>
where
    R: Resolver + Send + Sync + 'static,
    R::Input: Clone + Hash + Eq + Send + Sync + 'static,
    R::Output: Clone + Send + Sync + 'static,
{
    type Input = R::Input;
    type Output = R::Output;
    type Error = R::Error;

    async fn resolve(&self, input: &Self::Input) -> Result<Option<Self::Output>, Self::Error> {
        match self.pending.entry(input.clone()) {
            Entry::Occupied(occupied) => {
                let tx = occupied.get().lock().await.clone();
                drop(occupied);
                Ok(tx.subscribe().recv().await.expect("recv"))
            }
            Entry::Vacant(vacant) => {
                let (tx, _) = channel(1);
                vacant.insert(Arc::new(Mutex::new(tx.clone())));
                let result = self.inner.resolve(input).await;
                let _ = tx.send(result.as_ref().cloned().transpose().and_then(Result::ok));
                self.pending.remove(input);
                result
            }
        }
    }
}
