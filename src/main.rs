mod error;
mod ffi;
mod fs;
mod home_dir;
mod info_file;
mod light_fs;
mod trash;

#[cfg(test)]
mod tests;

use error::Result;
use trash::Trash;

fn main() {
    if let Err(err) = run() {
        eprintln!("tt: error: {}", err);
        std::process::exit(127);
    }
}

fn run() -> Result<()> {
    let home_dir = home_dir::home_dir().expect("failed to obtain user's home directory");
    let home_trash = home_dir::home_trash_path(&home_dir)?;
    let home_trash = Trash::from_root_checked(&home_trash)?;

    Ok(())
}
