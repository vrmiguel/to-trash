//! The $trash/info directory contains an “information file” for every file and directory in $trash/files. This file MUST have exactly the same name as the file or directory in $trash/files, plus the extension “.trashinfo”7.
//!
//! The format of this file is similar to the format of a desktop entry file, as described in the Desktop Entry Specification . Its first line must be [Trash Info].
//!
//! It also must have two lines that are key/value pairs as described in the Desktop Entry Specification:
//!
//!    * The key “Path” contains the original location of the file/directory, as either an absolute pathname (starting with the slash character “/”) or a relative pathname (starting with any other character). A relative pathname is to be from the directory in which the trash directory resides (for example, from $XDG_DATA_HOME for the “home trash” directory); it MUST not include a “..” directory, and for files not “under” that directory, absolute pathnames must be used. The system SHOULD support absolute pathnames only in the “home trash” directory, not in the directories under $topdir.
//!        - The value type for this key is “string”; it SHOULD store the file name as the sequence of bytes produced by the file system, with characters escaped as in URLs (as defined by RFC 2396, section 2).
//!    * The key “DeletionDate” contains the date and time when the file/directory was trashed. The date and time are to be in the YYYY-MM-DDThh:mm:ss format (see RFC 3339). The time zone should be the user's (or filesystem's) local time. The value type for this key is “string”.

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};

use fs_err::File;
use crate::error::Result;
use crate::ffi;
use crate::trash::Trash;
use std::time::Duration;

/// Builds the name of the info file for a file being trashed.
pub fn build_info_file_path(file_name: &OsStr, trash_info_path: &Path) -> PathBuf {
    let mut file_name = file_name.to_owned();
    file_name.push(".trashinfo");

    trash_info_path.join(file_name)
}

/// The $trash/info directory contains an “information file” for every file and directory in $trash/files. This file MUST have exactly the same name as the file or directory in $trash/files, plus the extension “.trashinfo”7.
///
/// The format of this file is similar to the format of a desktop entry file, as described in the Desktop Entry Specification . Its first line must be [Trash Info].
///
/// It also must have two lines that are key/value pairs as described in the Desktop Entry Specification:
///
///    * The key “Path” contains the original location of the file/directory, as either an absolute pathname (starting with the slash character “/”) or a relative pathname (starting with any other character). A relative pathname is to be from the directory in which the trash directory resides (for example, from $XDG_DATA_HOME for the “home trash” directory); it MUST not include a “..” directory, and for files not “under” that directory, absolute pathnames must be used. The system SHOULD support absolute pathnames only in the “home trash” directory, not in the directories under $topdir.
///        - The value type for this key is “string”; it SHOULD store the file name as the sequence of bytes produced by the file system, with characters escaped as in URLs (as defined by RFC 2396, section 2).
///    * The key “DeletionDate” contains the date and time when the file/directory was trashed. The date and time are to be in the YYYY-MM-DDThh:mm:ss format (see RFC 3339). The time zone should be the user's (or filesystem's) local time. The value type for this key is “string”.
///
/// This function writes the info file for the file given by `file_name`, which was originally in `original_path` (before getting trashed).
///
/// The trash used is given by `trash`.
///
/// The deletion timestamp is given by `deletion_date`, a [`Duration`] starting in UNIX_EPOCH.
///
/// Returns the path of the created info file, if successful.
pub fn write_info_file(
    original_path: &Path,
    file_name: &OsStr,
    trash: &Trash,
    deletion_date: Duration,
) -> Result<PathBuf> {
    // The date and time are to be in the YYYY-MM-DDThh:mm:ss format.
    // The time zone should be the user's (or filesystem's) local time.
    let rfc3339 = ffi::format_timestamp(deletion_date)?;

    // The info file is to be built in $trash/info
    let info_path = trash.info_path();

    // This file MUST have exactly the same name as the file or directory in $trash/files, plus the extension “.trashinfo”.
    let info_file_path = build_info_file_path(file_name, info_path);

    let mut info_file = File::create(&info_file_path)?;

    writeln!(info_file, "[Trash Info]")?;
    // TODO: is this correct when `original_path` isn't valid UTF-8?
    writeln!(info_file, "Path={}", original_path.display())?;
    writeln!(info_file, "DeletionDate={}", &rfc3339)?;

    info_file.sync_all()?;

    Ok(info_file_path)
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::{OsStr, OsString},
        fs::{self, File},
        io::Write,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        ffi,
        home_dir::home_dir,
        info_file::{build_info_file_path, write_info_file},
        tests::dummy_bytes,
        trash::Trash,
    };

    #[test]
    fn builds_info_file_path_correctly() {
        let trash_info = Path::new("/home/dummy/.local/share/Trash/info");
        let file_name = OsStr::new("deleted-file");

        assert_eq!(
            build_info_file_path(file_name, trash_info),
            Path::new("/home/dummy/.local/share/Trash/info/deleted-file.trashinfo")
        );
    }

    #[test]
    fn builds_and_writes_info_file_correctly() {
        let home_dir = home_dir().unwrap();
        let dir = tempfile::tempdir_in(&home_dir).unwrap();
        let dir_path = dir.path();
        let trash = Trash::from_root(dir_path).unwrap();

        fs::create_dir(trash.info_path()).unwrap();

        // The file to be trashed
        let file_name = OsString::from("dummy");
        let dummy_file_path = dir_path.join("dummy");
        let mut dummy = File::create(&dummy_file_path).unwrap();
        dummy.write_all(&dummy_bytes()).unwrap();

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        write_info_file(&dummy_file_path, &file_name, &trash, now).unwrap();

        let info_file_path = trash.info_path().join("dummy.trashinfo");
        let info_file = fs::read_to_string(&info_file_path).unwrap();

        let rfc3339 = ffi::format_timestamp(now).unwrap();

        let info_file_should_be = format!(
            "[Trash Info]\nPath={}\nDeletionDate={}\n",
            dummy_file_path.display(),
            rfc3339
        );

        assert_eq!(info_file, info_file_should_be)
    }
}
