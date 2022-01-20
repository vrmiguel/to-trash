use std::mem;
use std::os::unix::fs::PermissionsExt;
use std::{ffi::CStr, fs::Permissions};

use libc::lstat;

use crate::error::{Error, Result};

pub struct Lstat {
    inner: libc::stat,
}

#[allow(dead_code)]
impl Lstat {
    pub fn lstat(path: impl AsRef<CStr>) -> Result<Self> {
        Ok(Self {
            inner: _lstat(path)?,
        })
    }

    pub const fn mode(&self) -> u32 {
        self.inner.st_mode
    }

    pub const fn size(&self) -> i64 {
        self.inner.st_size
    }

    pub const fn block_size(&self) -> i64 {
        self.inner.st_blksize
    }

    pub fn permissions(&self) -> Permissions {
        Permissions::from_mode(self.mode())
    }

    pub const fn blocks(&self) -> i64 {
        self.inner.st_blocks
    }

    pub const fn accessed(&self) -> u64 {
        self.inner.st_atime as u64
    }

    pub const fn modified(&self) -> u64 {
        self.inner.st_mtime as u64
    }

    pub const fn owner_user_id(&self) -> u32 {
        self.inner.st_uid
    }

    pub const fn owner_group_id(&self) -> u32 {
        self.inner.st_gid
    }
}

fn _lstat(path: impl AsRef<CStr>) -> Result<libc::stat> {
    // Safety: The all-zero byte-pattern is a valid `struct stat`
    let mut stat_buf = unsafe { mem::zeroed() };

    if -1 == unsafe { lstat(path.as_ref().as_ptr(), &mut stat_buf) } {
        let io_err = std::io::Error::last_os_error();
        Err(Error::Io(io_err))
    } else {
        Ok(stat_buf)
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::TryFrom, time::UNIX_EPOCH};

    use tempfile::NamedTempFile;
    use unixstring::UnixString;

    use super::Lstat;

    #[test]
    fn permissions() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_owned();
        let permissions = path.metadata().unwrap().permissions();
        let path = UnixString::try_from(path).unwrap();

        assert_eq!(permissions, Lstat::lstat(&path).unwrap().permissions());
    }

    #[test]
    fn time_of_last_modification() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        let mod_timestamp = path
            .metadata()
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let unx = UnixString::try_from(path.to_owned()).unwrap();
        let stat = Lstat::lstat(&unx).unwrap();

        assert_eq!(mod_timestamp, stat.modified());
    }
}
