use super::Resolver;
use crate::error::{Error, Result};
use async_trait::async_trait;
use dashmap::{DashMap, Entry};
use std::hash::Hash;
use std::{fmt::Debug, sync::Arc};
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Mutex;

type SharedSender<T> = Arc<Mutex<Sender<T>>>;

pub struct ThrottledResolver<R, I, O> {
    resolver: R,
    senders: Arc<DashMap<I, SharedSender<Option<O>>>>,
}

impl<R, I, O> ThrottledResolver<R, I, O>
where
    I: Clone + Hash + Eq + Send + Sync + 'static + Debug,
{
    pub fn new(resolver: R) -> Self {
        Self {
            resolver,
            senders: Arc::new(DashMap::new()),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<R, I, O> Resolver for ThrottledResolver<R, I, O>
where
    R: Resolver<Input = I, Output = O> + Send + Sync + 'static,
    I: Clone + Hash + Eq + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
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
