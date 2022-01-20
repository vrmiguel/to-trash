mod getpwuid;
mod time;

pub fn effective_user_id() -> u32 {
    // Safety: the POSIX Programmer's Manual states that
    // geteuid will always be successful.
    unsafe { libc::geteuid() }
}

pub use getpwuid::get_home_dir;
pub use time::format_timestamp;
