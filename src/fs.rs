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
    let (from, to) = (from.as_ref(), to.as_ref());
    fs::copy(from, to)?;
    if from.is_dir() {
        fs::remove_dir_all(from)?;
    } else {
        fs::remove_file(from)?;
    }

    Ok(())
}

// TODO: copy over tests from old to-trash implementation

#[cfg(test)]
mod tests {
    use std::convert::TryInto;
    use std::fs::File;
    use std::io::Write;

    use unixstring::UnixString;

    use crate::ffi::Lstat;
    use crate::fs::{copy_and_remove, move_file};
    use crate::tests::dummy_bytes;

    #[test]
    fn test_clone_and_delete() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path();

        let contents = dummy_bytes();

        let file_path: UnixString = dir_path.join("dummy").try_into().unwrap();
        {
            let mut file = File::create(&file_path).unwrap();
            file.write_all(&contents).unwrap();
        }
        assert!(file_path.as_path().exists());

        let prev_stat = Lstat::lstat(&file_path).unwrap();

        let new_path: UnixString = dir_path.join("moved_dummy").try_into().unwrap();
        // There shouldn't be anything here yet
        assert!(!new_path.as_path().exists());
        copy_and_remove(file_path.as_path(), new_path.as_path()).unwrap();

        // This file shouldn't exist anymore!
        assert!(!file_path.as_path().exists());
        // And this one should now exist
        assert!(new_path.as_path().exists());

        let new_stat = Lstat::lstat(&new_path).unwrap();

        assert_eq!(contents, std::fs::read(new_path).unwrap());

        // Make sure that permission bits, accessed & modified times were maintained
        assert_eq!(prev_stat.permissions(), new_stat.permissions());

        assert_eq!(prev_stat.modified(), new_stat.modified());

        assert_eq!(prev_stat.accessed(), new_stat.accessed());
    }

    #[test]
    fn test_move_file() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path();

        let contents = dummy_bytes();

        let file_path: UnixString = dir_path.join("dummy").try_into().unwrap();
        {
            let mut file = File::create(&file_path).unwrap();
            file.write_all(&contents).unwrap();
        }
        assert!(file_path.as_path().exists());

        let prev_stat = Lstat::lstat(&file_path).unwrap();

        let new_path: UnixString = dir_path.join("moved_dummy").try_into().unwrap();
        // There shouldn't be anything here yet
        assert!(!new_path.as_path().exists());
        move_file(&file_path, &new_path).unwrap();

        // This file shouldn't exist anymore!
        assert!(!file_path.as_path().exists());
        // And this one should now exist
        assert!(new_path.as_path().exists());

        let new_stat = Lstat::lstat(&new_path).unwrap();

        assert_eq!(contents, std::fs::read(new_path).unwrap());

        // Make sure that permission bits, accessed & modified times were maintained
        assert_eq!(prev_stat.permissions(), new_stat.permissions());

        assert_eq!(prev_stat.modified(), new_stat.modified());

        assert_eq!(prev_stat.accessed(), new_stat.accessed());
    }
}