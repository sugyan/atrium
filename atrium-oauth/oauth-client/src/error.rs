use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Resolver(#[from] crate::resolver::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
