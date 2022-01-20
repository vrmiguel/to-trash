use std::{mem, time::Duration};

use cstr::cstr;
use libc::{c_char, localtime_r, size_t, time, tm};
use unixstring::UnixString;

use crate::error::Result;

// crate libc doesn't have bindings to those yet
extern "C" {
    pub fn strftime(
        s: *mut c_char,
        maxsize: size_t,
        format: *const c_char,
        timeptr: *const tm,
    ) -> size_t;

    pub fn tzset();
}

const BUF_SIZ: usize = 64;

/// Formats a timestamp (represented as a [`Duration`] since UNIX_EPOCH) into a YYYY-MM-DDThh:mm:ss format
pub fn format_timestamp(now: Duration) -> Result<String> {
    let mut timestamp = now.as_secs();

    // Safety: the all-zero byte-pattern is valid struct tm
    let mut new_time: tm = unsafe { mem::zeroed() };

    // Safety: time is memory-safe
    // TODO: it'd be better to call `time(NULL)` here
    let ltime = unsafe { time(&mut timestamp as *mut _ as *mut _) };

    unsafe { tzset() };

    // Safety: localtime_r is memory safe, threadsafe.
    unsafe { localtime_r(&ltime as *const i64, &mut new_time as *mut tm) };

    let mut char_buf: [c_char; BUF_SIZ] = [0; BUF_SIZ];

    // RFC3339 timestamp
    let format = cstr!("%Y-%m-%dT%T");

    unsafe {
        strftime(
            char_buf.as_mut_ptr(),
            BUF_SIZ,
            format.as_ptr(),
            &new_time as *const tm,
        )
    };

    let unx = unsafe { UnixString::from_ptr(char_buf.as_ptr()) };

    Ok(unx.to_string_lossy().into())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use chrono::Local;

    use crate::ffi::time::format_timestamp;

    #[test]
    fn formats_timestamp_into_valid_rfc3339() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        // We'll use the chrono crate to make sure that
        // our own formatting (done through libc's strftime) works
        let date_time = Local::now();

        // YYYY-MM-DDThh:mm:ss
        let rfc3339 = date_time.format("%Y-%m-%dT%T").to_string();

        assert_eq!(&rfc3339, &format_timestamp(now).unwrap());
    }
}
