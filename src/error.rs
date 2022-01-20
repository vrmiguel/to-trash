use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Interior nul byte found in CString")]
    InteriorNulByte(#[from] unixstring::Error),
    #[error("Path {0} does not contain a working trash directory")]
    TrashDirDoesNotExist(PathBuf),
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse mount points")]
    FailedToObtainMountPoints,
    #[error("Clock went backwards: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("Failed to obtain filename of path {0}")]
    FailedToObtainFileName(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;
