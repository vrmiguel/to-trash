use std::{ffi::OsString, fs, path::Path};

use uuid::Uuid;

use crate::error::Result;


/// Assuming that a file with path `path` exists in the directory `dir`,
/// this function appends to `path` an UUID in order to make its path unique.
/// 
/// This is needed whenever we want to send a file to $trash/files but it already contains a file with the same path.
pub fn build_unique_file_name(path: &Path, dir: &Path) -> OsString {
    debug_assert!(dir.join(path).exists());

    let uuid = Uuid::new_v4().to_string();
    let mut new_file_name = path.as_os_str().to_owned();
    new_file_name.push(uuid);
    new_file_name
}

/// Tries to rename a file from `from` to `to`.
/// 
/// If renaming fails, copies the contents of the file to the new path and removes the original source.
pub fn move_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    // TODO: add rename to light-fs and switch these arguments to impl AsRef<CStr>
    if fs::rename(&from, &to).is_err() {
        // rename(2) failed, likely because the files are in different mount points
        // or are on separate filesystems.
        copy_and_remove(from, to)?;
    }

    Ok(())
}


/// Will copy the contents of `from` into `to`.
/// 
/// The file in `from` is then deleted.
fn copy_and_remove(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
    fs::copy(from.as_ref(), to.as_ref())?;
    if from.as_ref().is_dir() {
        fs::remove_dir_all(from)?;
    } else {
        fs::remove_file(from)?;
    }

    Ok(())
}