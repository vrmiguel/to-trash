use std::ffi::OsStr;
use std::io::Write;
use std::os::unix::prelude::OsStrExt;
use std::time::Duration;

use fs_err as fs;
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

use crate::fs::copy_directorysizes;
use crate::trash::Trash;

/// Updates the $trash/directorysizes file with the information
/// of a directory being trashed.
// TODO: receive the that this directory will have in the trash?
// TODO: add test
pub fn update_directory_sizes(
    // The trash that this directory was sent to
    trash: &Trash,
    // The total size of the directory and its contents, in bytes
    directory_size: u64,
    // The name of this directory in `$trash/files`
    file_name_in_trash: &OsStr,
    // When this file was trashed
    deletion_time: Duration,
) -> crate::Result<()> {
    // The name of this directory (after trashed), in bytes
    let file_name = file_name_in_trash.as_bytes();

    // The percent encoded name of this directory
    let percent_encoded = percent_encode(file_name, NON_ALPHANUMERIC);

    // Unix timestamp of when this directory was deleted
    let deletion_time = deletion_time.as_secs();

    // Copy $trash/directorysizes to temp file
    let _temp = copy_directorysizes(trash)?;

    // Even though we already have a handle to this file (right above),
    // we'll reopen it in order to be able to append to it, instead of overwriting its contents
    let mut temp = fs::OpenOptions::new().append(true).open(_temp.path())?;

    // Append to temp file
    writeln!(temp, "{directory_size} {deletion_time} {percent_encoded}")?;

    // Atomic rename to actual directorysizes file
    fs::rename(temp.path(), trash.directory_sizes.as_path())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        os::unix::prelude::OsStrExt,
    };

    use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
    use tempfile::TempDir;

    use crate::{fs::directory_size, tests::dummy_bytes, trash::Trash};

    fn dummy_dir() -> crate::Result<(TempDir, Vec<File>)> {
        let dir = tempfile::tempdir()?;
        let mut files = Vec::with_capacity(5);

        for _ in 0..5 {
            let mut file = tempfile::tempfile_in(dir.path())?;
            file.write_all(&dummy_bytes())?;
            files.push(file);
        }

        Ok((dir, files))
    }

    #[test]
    fn updates_directorysizes_correctly_when_trashing() -> crate::Result<()> {
        let (dir_to_trash, _files) = dummy_dir()?;

        let directory_size = directory_size(dir_to_trash.path().to_owned().try_into()?)?;

        let temp_trash = tempfile::tempdir()?;
        let trash = Trash::from_root(temp_trash.path())?;

        fs::create_dir(&trash.files)?;
        fs::create_dir(&trash.info)?;

        const FIRST_LINE: &str = "16384 15803468 Documents";

        {
            let mut directorysizes = std::fs::File::create(&trash.directory_sizes)?;
            writeln!(directorysizes, "{FIRST_LINE}")?;
        }

        let trashed_file_name = trash.send_to_trash(dir_to_trash.path())?;
        let percent_encoded =
            percent_encode(trashed_file_name.as_os_str().as_bytes(), NON_ALPHANUMERIC);

        let directorysizes = fs::read_to_string(&trash.directory_sizes)?;
        dbg!(&directorysizes);
        let mut lines = directorysizes.lines();

        // Must not have overwritten the first line
        assert!(lines.next().unwrap().trim() == FIRST_LINE);
        let second_line = lines.next().unwrap();
        assert!(lines.next().is_none());

        let mut second_line_items = second_line.split_ascii_whitespace();

        assert_eq!(
            second_line_items.next().unwrap().parse::<u64>().unwrap(),
            directory_size
        );

        // TODO: We don't know what the timestamp is exactly. Maybe make `send_to_trash` return it.
        assert!(second_line_items.next().unwrap().parse::<u64>().is_ok());

        assert_eq!(
            second_line_items.next().unwrap().trim(),
            percent_encoded.to_string()
        );

        Ok(())
    }
}
