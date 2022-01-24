use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::prelude::OsStrExt;
use std::time::Duration;

use fs_err as fs;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use crate::fs::copy_directorysizes;
use crate::trash::Trash;

/// Updates the $trash/directorysizes file with the information
/// of a directory being trashed.
// TODO: receive the that this directory will have in the trash?
// TODO: add test
pub fn update_directory_sizes(
    // The trash that this directory was sent to
    trash: &Trash,
    // The total size of the directory and its contents, in bytes
    directory_size: u64,
    // The name of this directory in `$trash/files`
    file_name_in_trash: &OsStr,
    // When this file was trashed
    deletion_time: Duration,
) -> crate::Result<()> {
    // The name of this directory (after trashed), in bytes
    let file_name = file_name_in_trash.as_bytes();

    // The percent encoded name of this directory
    let percent_encoded = percent_encode(file_name, NON_ALPHANUMERIC);

    // Unix timestamp of when this directory was deleted
    let deletion_time = deletion_time.as_secs();

    // Copy $trash/directorysizes to temp file
    let mut temp = copy_directorysizes(&trash)?;

    // Append to temp file
    writeln!(temp.as_file_mut(), "{directory_size} {deletion_time} {percent_encoded}")?;

    // Atomic rename to actual directorysizes file
    fs::rename(temp.path(), trash.directory_sizes.as_path())?;

    // Remove temp file
    // fs::remove_file(file_name)?;

    Ok(())
}
