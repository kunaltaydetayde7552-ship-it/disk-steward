//! Free/total/used bytes of the internal data volume via statfs.

use crate::{model::DiskStatus, util::now_rfc3339};
use std::{ffi::CString, io, mem::MaybeUninit};

pub const DATA_VOLUME: &str = "/System/Volumes/Data";

pub fn status() -> io::Result<DiskStatus> {
    let (total, free) = space(DATA_VOLUME)?;
    Ok(DiskStatus {
        volume: DATA_VOLUME.into(),
        total_bytes: total,
        free_bytes: free,
        used_bytes: total.saturating_sub(free),
        measured_at: now_rfc3339(),
    })
}

/// `(total, free)` bytes for the volume holding `path`. Free is what unprivileged users can use (`f_bavail`).
pub fn space(path: &str) -> io::Result<(u64, u64)> {
    let c = CString::new(path).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut st = MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: `c` is a valid NUL-terminated path and `st` is a valid out-pointer.
    if unsafe { libc::statfs(c.as_ptr(), st.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: statfs returned 0, so the struct is initialized.
    let st = unsafe { st.assume_init() };
    let bsize = u64::from(st.f_bsize);
    Ok((st.f_blocks * bsize, st.f_bavail * bsize))
}

#[cfg(test)]
mod tests {
    #[test]
    fn data_volume_is_consistent() {
        let d = super::status().unwrap();
        assert!(d.total_bytes > 0 && d.free_bytes <= d.total_bytes);
        assert_eq!(d.used_bytes, d.total_bytes - d.free_bytes);
    }
}
