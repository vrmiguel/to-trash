mod error;
mod ffi;
mod home_dir;
mod light_fs;
mod trash;
mod fs;

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
