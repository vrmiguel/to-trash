use std::{
    ffi::OsString,
    fs::{self},
    path::Path,
};

use tempfile::NamedTempFile;
use unixstring::UnixString;
use uuid::Uuid;

use crate::{error::Result, ffi::Lstat, light_fs::{path_is_directory, path_is_regular_file}, trash::Trash};

/// Assuming that a file with path `path` exists in the directory `dir`,
/// this function appends to `path` an UUID in order to make its path unique.
///
/// This is needed whenever we want to send a file to $trash/files but it already contains a file with the same path.
pub fn build_unique_file_name(path: impl AsRef<Path>, _dir: impl AsRef<Path>) -> OsString {
    // debug_assert!(dir.join(path).exists());

    let uuid = Uuid::new_v4().to_string();
    let mut new_file_name = path.as_ref().as_os_str().to_owned();
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

/// Makes a temporary copy of `$trash/directorysizes`.
pub fn copy_directorysizes(path: &Trash) -> Result<NamedTempFile> {
    let temp = NamedTempFile::new_in(path.files.as_path())?;

    // Copy the directorysizes to our new path
    fs::copy(path.directory_sizes.as_path(), temp.path())?;

    // let file = OpenOptions::new()
    //     .write(true)
    //     .append(true)
    //     .open(&copy_path)?;

    Ok(temp)
}

/// Scans a directory recursively adding up the total of bytes it contains.
///
/// Symlinks found are not followed.
pub fn directory_size(path: UnixString) -> Result<u64> {
    let mut size = 0;

    let lstat_size = |path: &UnixString| -> crate::Result<u64> { Ok(Lstat::lstat(path)?.size()) };

    if path.as_path().is_dir() {
        for entry in fs::read_dir(&path)? {
            let entry: UnixString = entry?.path().try_into()?;
            if path_is_regular_file(&entry) {
                size += lstat_size(&entry)?;
            } else if path_is_directory(&entry) {
                size += directory_size(entry)?;
            }
        }
    } else {
        size = lstat_size(&path)?;
    }

    Ok(size)
}

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
        File::create(&file_path)
            .unwrap()
            .write_all(&contents)
            .unwrap();
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
