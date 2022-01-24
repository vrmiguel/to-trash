//! Small filesystem-related utilities. These are used instead of std::fs since these
//! avoid the CString allocation caused whenever std::fs uses a syscall.

use std::ffi::CStr;

use crate::ffi::Lstat;

/// Checks if the given path exists
pub fn path_exists(path: impl AsRef<CStr>) -> bool {
    0 == unsafe { libc::access(path.as_ref().as_ptr(), libc::F_OK) }
}

/// Returns true if the given path exists and is a directory
pub fn path_is_directory(path: impl AsRef<CStr>) -> bool {
    let is_directory = |lstat: Lstat| lstat.mode() & libc::S_IFMT == libc::S_IFDIR;
    Lstat::lstat(path).map(is_directory).unwrap_or_default()
}

pub fn path_is_regular_file(path: impl AsRef<CStr>) -> bool {
    let is_directory = |lstat: Lstat| lstat.mode() & libc::S_IFMT == libc::S_IFREG;
    Lstat::lstat(path).map(is_directory).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use unixstring::UnixString;

    use crate::light_fs::{path_exists, path_is_directory};

    #[test]
    fn path_exists_works() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();
        let path: UnixString = tempfile.path().to_owned().try_into().unwrap();
        assert_eq!(path_exists(&path), true);

        fs::remove_file(&path).unwrap();
        assert_eq!(path_exists(&path), false);
    }

    #[test]
    fn path_is_directory_works() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();
        let tempdir = tempfile::tempdir().unwrap();

        let file_path: UnixString = tempfile.path().to_owned().try_into().unwrap();
        let dir_path: UnixString = tempdir.path().to_owned().try_into().unwrap();

        assert_eq!(path_is_directory(&file_path), false);
        assert_eq!(path_is_directory(&dir_path), true);
    }
}
