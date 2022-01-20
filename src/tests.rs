use std::{fs::File, io::Write};

use rand::{prelude::SmallRng, RngCore, SeedableRng};

use crate::{home_dir::home_dir, trash::Trash};

pub fn dummy_bytes() -> Vec<u8> {
    let mut rng = SmallRng::from_entropy();
    let quantity = 1024 + rng.next_u32() % 1024;
    let mut vec = vec![0; quantity as usize];
    rng.fill_bytes(&mut vec);
    vec
}

#[test]
/// TODO: check for info file
/// TODO: add test for directorysizes
fn sends_file_to_trash() -> crate::Result<()> {
    let home_dir = home_dir().unwrap();
    let dir = tempfile::tempdir_in(&home_dir).unwrap();
    let dir_path = dir.path();
    let trash = Trash::from_root(dir_path)?;

    std::fs::create_dir(&trash.directory_sizes)?;
    std::fs::create_dir(&trash.files)?;
    std::fs::create_dir(&trash.info)?;

    let dummy_path = dir_path.join("dummy");
    let mut dummy = File::create(&*dummy_path).unwrap();
    dummy.write_all(&dummy_bytes()).unwrap();

    trash.send_to_trash(&dummy_path)?;

    // This path should no longer exist!
    assert!(!dummy_path.exists());

    // The file should now be in the trash
    let new_path = trash.files.as_path().join("dummy");
    dbg!(&new_path);

    // The new file (now in the trash) should now exist
    assert!(new_path.exists());

    Ok(())
}
