//! Small filesystem-related utilities. These are used instead of std::fs since these
//! avoids the CString allocation used in every syscall used by std::fs.  

use std::ffi::CStr;

/// Checks if the given path exists
pub fn path_exists(path: impl AsRef<CStr>) -> bool {
    0 == unsafe { libc::access(path.as_ref().as_ptr(), libc::F_OK) }
}
