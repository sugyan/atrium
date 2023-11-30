use async_trait::async_trait;
use atrium_api::agent::{store::SessionStore, Session};
use std::path::{Path, PathBuf};
use tokio::fs::{remove_file, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct SimpleJsonFileSessionStore<T = PathBuf>
where
    T: AsRef<Path>,
{
    path: T,
}

impl<T> SimpleJsonFileSessionStore<T>
where
    T: AsRef<Path>,
{
    pub fn new(path: T) -> Self {
        Self { path }
    }
}

#[async_trait]
impl<T> SessionStore for SimpleJsonFileSessionStore<T>
where
    T: AsRef<Path> + Send + Sync + 'static,
{
    async fn get_session(&self) -> Option<Session> {
        let mut file = File::open(self.path.as_ref()).await.ok()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.ok()?;
        serde_json::from_slice(&buffer).ok()
    }
    async fn set_session(&self, session: Session) {
        let mut file = File::create(self.path.as_ref()).await.unwrap();
        let buffer = serde_json::to_vec_pretty(&session).ok().unwrap();
        file.write_all(&buffer).await.ok();
    }
    async fn clear_session(&self) {
        remove_file(self.path.as_ref()).await.ok();
    }
}
