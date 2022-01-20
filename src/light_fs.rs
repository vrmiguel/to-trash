//! Small filesystem-related utilities. These are used instead of std::fs since these
//! avoid the CString allocation caused whenever std::fs uses a syscall.

use std::ffi::CStr;

/// Checks if the given path exists
pub fn path_exists(path: impl AsRef<CStr>) -> bool {
    0 == unsafe { libc::access(path.as_ref().as_ptr(), libc::F_OK) }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use unixstring::UnixString;

    use crate::light_fs::path_exists;

    #[test]
    fn test_path_exists() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();
        let path: UnixString = tempfile.path().to_owned().try_into().unwrap();
        assert_eq!(path_exists(&path), true);

        fs::remove_file(&path).unwrap();
        assert_eq!(path_exists(&path), false);
    }
}
