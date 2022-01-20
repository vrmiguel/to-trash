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
    pub fn from_root(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();

        let files = UnixString::from_pathbuf(root.join("files"))?;
        let directory_sizes = UnixString::from_pathbuf(root.join("directorysizes"))?;
        let info = UnixString::from_pathbuf(root.join("info"))?;

        Ok(Self {
            files,
            directory_sizes,
            info,
        })
    }

    /// Checks that the directories of this trash exist.
    ///
    /// Doesn't check for `$trash/directorysizes` since it was added in a later version of the spec
    /// so it might have been created.
    pub fn assert_exists(&self) -> Result<()> {
        if !path_exists(&self.info) || !path_exists(&self.files) {
            let root = self
                .files
                .as_path()
                .parent()
                .expect("catastrophe: path ends with a root or prefix");
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
    fn trash_from_root() -> Result<()> {
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
