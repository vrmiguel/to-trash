use std::{mem, ptr};

use libc::{getpwuid_r, passwd};
use unixstring::UnixString;

use super::effective_user_id;

pub fn get_home_dir() -> Option<UnixString> {
    let mut buf = [0; 2048];
    let mut result = ptr::null_mut();
    let mut passwd: passwd = unsafe { mem::zeroed() };

    let uid = effective_user_id();

    let getpwuid_r_code =
        unsafe { getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut result) };

    if getpwuid_r_code == 0 && !result.is_null() {
        // If getpwuid_r succeeded, let's get the username from it

        let home_dir = unsafe { UnixString::from_ptr(passwd.pw_dir) };

        return Some(home_dir);
    }

    None
}
