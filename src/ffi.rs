mod getpwuid;
mod lstat;
mod mount_point;
mod time;

pub fn effective_user_id() -> u32 {
    // Safety: the POSIX Programmer's Manual states that
    // geteuid will always be successful.
    unsafe { libc::geteuid() }
}

pub use getpwuid::get_home_dir;
pub use lstat::Lstat;
pub use mount_point::{probe_mount_points, probe_mount_points_in, MountPoint};
pub use time::format_timestamp;
