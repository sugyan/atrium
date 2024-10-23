use super::Resolver;
use crate::error::{Error, Result};
use dashmap::{DashMap, Entry};
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Mutex;

type SharedSender<T> = Arc<Mutex<Sender<Option<T>>>>;

pub struct ThrottledResolver<R>
where
    R: Resolver,
    R::Input: Sized,
{
    resolver: R,
    senders: Arc<DashMap<R::Input, SharedSender<R::Output>>>,
}

impl<R> ThrottledResolver<R>
where
    R: Resolver,
    R::Input: Clone + Hash + Eq + Send + Sync + 'static,
{
    pub fn new(resolver: R) -> Self {
        Self { resolver, senders: Arc::new(DashMap::new()) }
    }
}

impl<R> Resolver for ThrottledResolver<R>
where
    R: Resolver + Send + Sync + 'static,
    R::Input: Clone + Hash + Eq + Send + Sync + 'static,
    R::Output: Clone + Send + Sync + 'static,
{
    type Input = R::Input;
    type Output = R::Output;

    async fn resolve(&self, input: &Self::Input) -> Result<Self::Output> {
        match self.senders.entry(input.clone()) {
            Entry::Occupied(occupied) => {
                let tx = occupied.get().lock().await.clone();
                drop(occupied);
                match tx.subscribe().recv().await.expect("recv") {
                    Some(result) => Ok(result),
                    None => Err(Error::NotFound),
                }
            }
            Entry::Vacant(vacant) => {
                let (tx, _) = channel(1);
                vacant.insert(Arc::new(Mutex::new(tx.clone())));
                let result = self.resolver.resolve(input).await;
                tx.send(result.as_ref().ok().cloned()).ok();
                self.senders.remove(input);
                result
            }
        }
    }
}
