use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use fs_err as fs;
use unixstring::UnixString;

use crate::{
    directorysizes::update_directory_sizes,
    error::{Error, Result},
    fs::{build_unique_file_name, directory_size},
    info_file::write_info_file,
    light_fs::path_exists,
};

#[derive(Debug)]
/// A trash directory contains three subdirectories, named `info`, `directorysizes` and `files`.
pub struct Trash {
    /// The $trash/files directory contains the files and directories that were trashed. When a file or directory is trashed, it must be moved into this directory.
    pub files: UnixString,
    /// The $trash/directorysizes directory is a cache of the sizes of the directories that were trashed
    /// in this trash directory. Individual trashed files are not present in this cache, since their size can be determined with a call to stat().
    pub directory_sizes: UnixString,
    /// The $trash/info directory contains an “information file” for every file and directory in $trash/files.
    /// This file must have exactly the same name as the file or directory in $trash/files, plus the extension “.trashinfo”
    pub info: UnixString,
}

impl Trash {
    /// Builds a trash directory rooted at `root`.
    ///
    /// Does not check if the directories of this trash directory exist.
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();

        let files = root.join("files").try_into()?;
        let directory_sizes = root.join("directorysizes").try_into()?;
        let info = root.join("info").try_into()?;

        Ok(Self {
            files,
            directory_sizes,
            info,
        })
    }

    /// Builds a trash directory rooted at `root` checking if the directories of this trash directory exist.
    pub fn from_root_checked(root: impl AsRef<Path>) -> Result<Self> {
        let trash = Self::from_root(root)?;
        trash.assert_exists()?;
        Ok(trash)
    }

    /// The path of the `info` folder for this trash directory
    pub fn info_path(&self) -> &Path {
        self.info.as_path()
    }

    /// Checks that the directories of this trash exist.
    ///
    /// Doesn't check for `$trash/directorysizes` since it was added in a later version of the spec
    /// so it might have been created.
    fn assert_exists(&self) -> Result<()> {
        if !path_exists(&self.info) || !path_exists(&self.files) {
            let root = self
                .files
                .as_path()
                .parent()
                .expect("catastrophe: trash root ends with a root or prefix");
            return Err(Error::TrashDirDoesNotExist(root.to_owned()));
        }

        Ok(())
    }

    /// Sends the file given by `path` to the given trash structure
    ///
    ///
    /// In case of success, returns the name of the trashed file
    /// exactly as sent to `TRASH/files`.
    ///
    /// # Note:
    ///
    /// From the FreeDesktop Trash spec 1.0:
    ///
    ///```
    ///   When trashing a file or directory, the implementation
    ///   MUST create the corresponding file in $trash/info first
    ///```
    /// Our implementation respects this by calling `build_info_file` before `move_file`
    pub fn send_to_trash(&self, to_be_removed: &Path) -> Result<PathBuf> {
        // How much time has passed since Jan 1st 1970?
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;

        // If we're trashing a directory, we must calculate its size
        let directory_size = if to_be_removed.is_dir() {
            let unx = to_be_removed.to_owned().try_into()?;
            Some(directory_size(unx)?)
        } else {
            None
        };

        // The name of the file to be removed
        let file_name = to_be_removed
            .file_name()
            .ok_or_else(|| Error::FailedToObtainFileName(to_be_removed.into()))?;

        // Where the file will be sent to once trashed
        let file_in_trash = self.files.as_path().join(&file_name);

        // According to the trash-spec 1.0 states that, a file in the trash
        // must not be overwritten by a newer file with the same filename.
        //
        // For this reason, we'll make a new unique filename for the file we're deleting if this
        // occurs
        let file_name = if file_in_trash.exists() {
            build_unique_file_name(&file_name, &self.files.as_path())
        } else {
            file_name.to_owned()
        };

        // The path of the trashed file in `$trash/files`
        let trash_file_path = self.files.as_path().join(&file_name);

        // Writes the info file for the file being trashed in `$trash/info`.
        // This must be done before deleting the original file, as per the spec.
        let info_file_path = write_info_file(&to_be_removed, &file_name, self, now)?;

        // Send the file being trashed... to the trash
        if let Err(err) = crate::fs::move_file(to_be_removed, &*trash_file_path) {
            // Remove the info file if moving the file fails
            fs::remove_file(info_file_path)?;
            eprintln!(
                "failed to move {} to {}",
                to_be_removed.display(),
                trash_file_path.display()
            );
            return Err(err);
        }

        // If we just trashed a directory, update `$trash/directorysizes`.
        if let Some(directory_size) = directory_size {
            update_directory_sizes(
                // The trash the directory was sent to
                self,
                // The size of this directory, in bytes
                directory_size,
                // The name of this directory in $trash/files
                &file_name,
                // When this directory was trashed
                now,
            )?;
        }

        Ok(file_name.into())
    }
}

#[cfg(test)]
mod tests {
    use super::Trash;
    use crate::error::Result;

    #[test]
    fn trash_from_root_has_correct_paths() -> Result<()> {
        let trash = Trash::from_root("/home/vrmiguel/.Trash")?;

        assert_eq!(trash.files, "/home/vrmiguel/.Trash/files");

        assert_eq!(
            trash.directory_sizes,
            "/home/vrmiguel/.Trash/directorysizes"
        );

        assert_eq!(trash.info, "/home/vrmiguel/.Trash/info");

        Ok(())
    }
}
