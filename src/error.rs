use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Interior nul byte found in CString")]
    InteriorNulByte(#[from] unixstring::Error),
    #[error("Path {0} does not contain a working trash directory")]
    TrashDirDoesNotExist(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;
