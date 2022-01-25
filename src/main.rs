mod directorysizes;
mod error;
mod ffi;
mod fs;
mod home_dir;
mod info_file;
mod light_fs;
mod trash;

#[cfg(test)]
mod tests;

use std::{
    env,
    path::{Path, PathBuf},
};

use lazy_static::lazy_static;

pub use error::{Error, Result};
use trash::Trash;
use unixstring::UnixString;

use crate::ffi::real_user_id;
use crate::ffi::MountPoint;

lazy_static! {
    // TODO: add a set of trashes of other mount points
    pub static ref HOME_DIR: UnixString = home_dir::home_dir().unwrap();
    pub static ref HOME_TRASH_PATH: UnixString =
        home_dir::home_trash_path(&*HOME_DIR).expect("failed to obtain user's home directory!");
    pub static ref MOUNT_POINTS: Vec<MountPoint> =
        ffi::probe_mount_points().expect("failed to probe mount points!");
    pub static ref HOME_TRASH: Trash =
        Trash::from_root(&*HOME_TRASH_PATH).expect("failed to probe mount points!");
}

fn find_mount_point_of_file(path: &Path) -> Result<&MountPoint> {
    MOUNT_POINTS
        .iter()
        .find(|mount_point| mount_point.contains(path))
        .ok_or(Error::FailedToObtainMountPoints)
}

fn main() {
    if let Err(err) = run() {
        eprintln!("tt: error: {}", err);
        std::process::exit(127);
    }
}

fn run() -> Result<()> {
    for file in env::args_os().skip(1) {
        let file = PathBuf::from(file).canonicalize()?;
        if file.starts_with("/home") {
            // The file is located at home so we'll send it to the home trash
            HOME_TRASH.send_to_trash(&file)?;
        } else {
            trash_file_in_other_mount_point(file)?;
        }
    }

    Ok(())
}

/// Tries to trash a file (given by `path` which is located in a non-home mount point)
fn trash_file_in_other_mount_point(path: PathBuf) -> Result<()> {
    // Try to find the mount point of this file
    let mount_point = find_mount_point_of_file(&path)?;
    let topdir = &mount_point.fs_path_prefix;

    // Check if a valid trash already exists in this mount point
    if let Ok(trash) = Trash::from_root_checked(topdir) {
        trash.send_to_trash(&path)?;
        return Ok(());
    };

    // If a $topdir/.Trash does not exist or has not passed the checks, check if `$topdir/.Trash-$uid` exists.
    // If a $topdir/.Trash-$uid directory does not exist, the implementation must immediately create it, without any warnings or delays for the user.
    // TODO: should we use the effective user ID here?
    let uid = real_user_id();

    let trash_uid_path = topdir.join(format!(".Trash-{}", uid));

    let trash = if let Ok(trash) = Trash::from_root_checked(&trash_uid_path) {
        trash
    } else {
        let trash = Trash::from_root(&trash_uid_path)?;
        fs_err::create_dir(&trash.info)?;
        fs_err::create_dir(&trash.files)?;
        fs_err::File::create(&trash.directory_sizes)?;

        trash
    };

    trash.send_to_trash(&path)?;

    Ok(())
}
