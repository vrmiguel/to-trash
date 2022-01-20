use std::path::Path;

use unixstring::UnixString;

use crate::{
    error::{Error, Result},
    light_fs::path_exists,
};

#[derive(Debug)]
/// A trash directory contains three subdirectories, named `info`, `directorysizes` and `files`.
pub struct Trash {
    /// The $trash/files directory contains the files and directories that were trashed. When a file or directory is trashed, it must be moved into this directory.
    files: UnixString,
    /// The $trash/directorysizes directory is a cache of the sizes of the directories that were trashed into this trash da cache of the sizes of the directories that were trashed into this trash directory. Individual trashed files are not present in this cache, since their size can be determined with a call to stat().irectory.
    /// Individual trashed files are not present in this cache, since their size can be determined with a call to stat().
    directory_sizes: UnixString,
    /// The $trash/info directory contains an “information file” for every file and directory in $trash/files.
    /// This file must have exactly the same name as the file or directory in $trash/files, plus the extension “.trashinfo”
    info: UnixString,
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
