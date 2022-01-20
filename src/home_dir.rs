use std::path::{Path, PathBuf};

use unixstring::UnixString;

use crate::error::Result;
use crate::ffi;

// Attemps to find the calling user's home directory.
/// Will check for the HOME env. variable first, falling back to
/// checking passwd if HOME isn't set.
pub fn home_dir() -> Option<UnixString> {
    match std::env::var_os("HOME").map(UnixString::from_os_string) {
        Some(Ok(unx)) => Some(unx),
        None => ffi::get_home_dir(),
        Some(Err(_)) => panic!("HOME has an interior nul byte"),
    }
}

/// XDG claims that the trash directory is located at $XDG_DATA_HOME/Trash.
/// Since XDG_DATA_HOME is often undefined by distros, we fallback to $HOME/.local/share/Trash
pub fn home_trash_path(home_dir: impl AsRef<Path>) -> Result<UnixString> {
    Ok(std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Trash"))
        .unwrap_or_else(|| home_dir.as_ref().join(".local/share/Trash"))
        .try_into()?)
}
