use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    ffi::CStr,
    path::{Path, PathBuf},
};

use crate::error::{Error, Result};
use cstr::cstr;
use libc::{getmntent, setmntent};
use unixstring::UnixString;

#[derive(Debug, PartialEq, Eq)]
pub struct MountPoint {
    pub fs_name: String,
    pub fs_path_prefix: PathBuf,
}

#[allow(dead_code)]
impl MountPoint {
    pub fn is_root(&self) -> bool {
        self.fs_path_prefix == Path::new("/")
    }

    pub fn is_home(&self) -> bool {
        self.fs_path_prefix == Path::new("/home")
    }

    pub fn contains(&self, path: &Path) -> bool {
        path.starts_with(&self.fs_path_prefix)
    }
}

#[cfg(test)]
mod mount_point_fns {

    use crate::ffi::MountPoint;

    fn root() -> MountPoint {
        MountPoint {
            fs_name: "/dev/sda2".into(),
            fs_path_prefix: "/".into(),
        }
    }

    fn home() -> MountPoint {
        MountPoint {
            fs_name: "/dev/sda2".into(),
            fs_path_prefix: "/home".into(),
        }
    }

    #[test]
    fn is_root() {
        assert!(root().is_root());
        assert!(!home().is_root());
    }

    #[test]
    fn is_home() {
        assert!(!root().is_home());
        assert!(home().is_home());
    }
}

impl PartialOrd for MountPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MountPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fs_path_prefix
            .as_os_str()
            .len()
            .cmp(&other.fs_path_prefix.as_os_str().len())
    }
}

/// Parses `/etc/mtab` (symlink to `/proc/self/mounts`) to list currently mounted file systems`
pub fn probe_mount_points() -> Result<Vec<MountPoint>> {
    let path = cstr!("/etc/mtab");

    probe_mount_points_in(path)
}

/// Parses the mounted file systems table given by `path`
pub fn probe_mount_points_in(path: &CStr) -> Result<Vec<MountPoint>> {
    let mut mount_points = BinaryHeap::new();

    let read_arg = cstr!("r");
    let file = unsafe { setmntent(path.as_ptr(), read_arg.as_ptr()) };

    if file.is_null() {
        return Err(Error::FailedToObtainMountPoints);
    }

    loop {
        let entry = unsafe { getmntent(file) };
        if entry.is_null() {
            break;
        }
        // We just made sure `entry` is not null,
        // so this deref must be safe (I guess?)
        let fs_name = unsafe { (*entry).mnt_fsname };
        let fs_dir = unsafe { (*entry).mnt_dir };

        let fs_name = unsafe { UnixString::from_ptr(fs_name) };

        let fs_dir = unsafe { UnixString::from_ptr(fs_dir) };

        let mount_point = MountPoint {
            fs_name: fs_name.into_string_lossy().into(),
            fs_path_prefix: fs_dir.into(),
        };
        mount_points.push(Reverse(mount_point));
    }

    Ok(mount_points
        .into_sorted_vec()
        .into_iter()
        .map(|rev_mount_point| rev_mount_point.0)
        .collect())
}

#[cfg(test)]
mod mount_point_probing_tests {
    use tempfile::NamedTempFile;

    use std::{
        collections::BTreeSet, ffi::CString, io::Write, os::unix::prelude::OsStrExt, time::Duration,
    };

    use crate::ffi::{probe_mount_points_in, MountPoint};

    const TEST_MTAB: &str = r#"
    proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
    sys /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
    dev /dev devtmpfs rw,nosuid,relatime,size=10574240k,nr_inodes=5743635,mode=755,inode64 0 0
    run /run tmpfs rw,nosuid,nodev,relatime,mode=755,inode64 0 0
    efivarfs /sys/firmware/efi/efivars efivarfs rw,nosuid,nodev,noexec,relatime 0 0
    /dev/sda2 / ext4 rw,noatime 0 0
    securityfs /sys/kernel/security securityfs rw,nosuid,nodev,noexec,relatime 0 0
    tmpfs /dev/shm tmpfs rw,nosuid,nodev,inode64 0 0
    devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=000 0 0
"#;

    #[test]
    // TODO: this test sometimes fails for weird reasons
    fn test_mount_point_probing() {
        // getmntent is not reentrant so this is currently needed to sort out multi-threaded weirdness
        std::thread::sleep(Duration::from_secs(1));

        let mut temp = NamedTempFile::new().unwrap();

        let temp_path = temp.path();
        let temp_path_cstr = CString::new(temp_path.as_os_str().as_bytes()).unwrap();

        write!(temp, "{}", TEST_MTAB).unwrap();

        let mount_points = probe_mount_points_in(&temp_path_cstr).unwrap();

        let mount_points: BTreeSet<_> = mount_points.into_iter().collect();

        let expected = vec![
            MountPoint {
                fs_name: "efivarfs".into(),
                fs_path_prefix: "/sys/firmware/efi/efivars".into(),
            },
            MountPoint {
                fs_name: "securityfs".into(),
                fs_path_prefix: "/sys/kernel/security".into(),
            },
            MountPoint {
                fs_name: "devpts".into(),
                fs_path_prefix: "/dev/pts".into(),
            },
            MountPoint {
                fs_name: "tmpfs".into(),
                fs_path_prefix: "/dev/shm".into(),
            },
            MountPoint {
                fs_name: "proc".into(),
                fs_path_prefix: "/proc".into(),
            },
            MountPoint {
                fs_name: "run".into(),
                fs_path_prefix: "/run".into(),
            },
            MountPoint {
                fs_name: "dev".into(),
                fs_path_prefix: "/dev".into(),
            },
            MountPoint {
                fs_name: "sys".into(),
                fs_path_prefix: "/sys".into(),
            },
            MountPoint {
                fs_name: "/dev/sda2".into(),
                fs_path_prefix: "/".into(),
            },
        ];

        let expected: BTreeSet<_> = expected.into_iter().collect();

        assert_eq!(mount_points, expected);
    }
}

#[cfg(test)]
mod mount_point_ordering_tests {
    use std::cmp::Reverse;

    use super::MountPoint;

    #[test]
    fn mount_point_cmp() {
        let first = MountPoint {
            fs_name: "portal".into(),
            fs_path_prefix: "/run/user/1000".into(),
        };

        let second = MountPoint {
            fs_name: "portal".into(),
            fs_path_prefix: "/run/user/1001/doc".into(),
        };

        assert!(first < second);

        assert!(Reverse(first) > Reverse(second))
    }

    #[test]
    fn mount_point_neq() {
        // 1st case: same `fs_name` but differing prefix
        let first = MountPoint {
            fs_name: "portal".into(),
            fs_path_prefix: "/run/user/1000/doc".into(),
        };

        let second = MountPoint {
            fs_name: "portal".into(),
            fs_path_prefix: "/run/user/1001/doc".into(),
        };

        assert!(first != second);

        // 2nd case: differing `fs_name` but same prefix
        let first = MountPoint {
            fs_name: "portal2".into(),
            fs_path_prefix: "/run/user/1000/doc".into(),
        };

        let second = MountPoint {
            fs_name: "portal".into(),
            fs_path_prefix: "/run/user/1000/doc".into(),
        };

        assert!(first != second);

        // 3rd case: both properties differ
        let first = MountPoint {
            fs_name: "portal2".into(),
            fs_path_prefix: "/run/user/1000/doc".into(),
        };

        let second = MountPoint {
            fs_name: "portal".into(),
            fs_path_prefix: "/run/user/1001/doc".into(),
        };

        assert!(first != second);
    }

    #[test]
    fn probing_returns_ordered_mount_points() {
        let mount_points = super::probe_mount_points().unwrap();

        if mount_points.len() < 2 {
            // We didn't get enough data in order to test this :C
            //
            // TODO: check if it's possible to mock `probe_mount_points`.
            panic!();
        }

        assert!(mount_points.windows(2).all(|w| w[0] >= w[1]));
    }
}
