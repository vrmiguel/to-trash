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
            let mount_point = find_mount_point_of_file(&file)?;
        }
    }

    Ok(())
}
