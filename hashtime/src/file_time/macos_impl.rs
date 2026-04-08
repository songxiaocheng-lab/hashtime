#[cfg(target_os = "macos")]
mod macos {
    use anyhow::{Context, Result, bail};
    use libc::{c_char, size_t};
    use std::ffi::CString;
    use std::os::raw::c_int;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    // From macOS headers:
    const ATTR_BIT_MAP_COUNT: u16 = 5;
    const ATTR_CMN_CRTIME: u32 = 0x0000_0200; // ATTR_CMN_CRTIME

    // The attrlist struct
    #[repr(C)]
    struct AttrList {
        bitmapcount: u16,
        reserved: u16,
        commonattr: u32,
        volattr: u32,
        dirattr: u32,
        fileattr: u32,
        forkattr: u32,
    }

    unsafe extern "C" {
        // int setattrlist(const char *path, const struct attrlist *attrList,
        //                 const void *attrBuf, size_t attrBufSize, unsigned int options);
        fn setattrlist(
            path: *const c_char,
            attr_list: *const AttrList,
            attr_buf: *const libc::c_void,
            attr_buf_size: size_t,
            options: u32,
        ) -> c_int;
    }

    /// set creation (birth) time on macOS using setattrlist + ATTR_CMN_CRTIME
    pub(crate) fn set_birthtime(path: &Path, birth_ns: i64) -> Result<()> {
        let p = CString::new(path.as_os_str().as_bytes()).context("CString::new path failed")?;

        let sec = birth_ns / 1_000_000_000;
        let nsec = birth_ns % 1_000_000_000;

        // use libc::timespec (on macOS it's timespec { tv_sec: time_t, tv_nsec: long })
        let ts = libc::timespec {
            tv_sec: sec,
            tv_nsec: nsec as libc::c_long,
        };

        // Build attrlist asking to set creation time
        let al = AttrList {
            bitmapcount: ATTR_BIT_MAP_COUNT,
            reserved: 0,
            commonattr: ATTR_CMN_CRTIME,
            volattr: 0,
            dirattr: 0,
            fileattr: 0,
            forkattr: 0,
        };

        // The attrBuf for ATTR_CMN_CRTIME expects a timespec (per man page).
        let ret = unsafe {
            setattrlist(
                p.as_ptr(),
                &al as *const _,
                &ts as *const _ as *const libc::c_void,
                size_of::<libc::timespec>() as size_t,
                0u32, // options (0 or FSOPT_NOFOLLOW if desired)
            )
        };

        if ret != 0 {
            let errno = std::io::Error::last_os_error();
            bail!("setattrlist failed: {}", errno);
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub(super) use macos::set_birthtime;
