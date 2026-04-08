#[cfg(target_os = "windows")]
mod windows {
    use anyhow::{Result, bail};
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::prelude::*;
    use std::path::Path;
    use winapi::shared::minwindef::FILETIME;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING, SetFileTime};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::winbase::FILE_FLAG_BACKUP_SEMANTICS;
    use winapi::um::winnt::{
        FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES, HANDLE,
    };

    fn to_win32_wide(s: &OsStr) -> Vec<u16> {
        s.encode_wide().chain(once(0)).collect()
    }

    /// Convert unix epoch ns to Windows FILETIME (100-ns intervals since Jan 1, 1601)
    fn ns_to_filetime(ns: u128) -> FILETIME {
        // Windows epoch starts at 1601-01-01; Unix epoch at 1970-01-01
        // difference in 100-ns units:
        const EPOCH_DIFF_100NS: u128 = 11644473600u128 * 10_000_000u128;
        let units_100ns = ns / 100u128;
        let total = EPOCH_DIFF_100NS + units_100ns;
        FILETIME {
            dwLowDateTime: (total & 0xffff_ffff) as u32,
            dwHighDateTime: ((total >> 32) & 0xffff_ffff) as u32,
        }
    }

    pub(crate) fn set_birthtime(path: &Path, birth_ns: i64) -> Result<()> {
        let path_w = to_win32_wide(path.as_os_str());
        unsafe {
            let handle: HANDLE = CreateFileW(
                path_w.as_ptr(),
                FILE_WRITE_ATTRIBUTES,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_BACKUP_SEMANTICS,
                std::ptr::null_mut(),
            );

            if handle == INVALID_HANDLE_VALUE {
                bail!("CreateFileW failed: {}", std::io::Error::last_os_error());
            }

            let ft = ns_to_filetime(birth_ns as u128);
            // We'll set creation time, leave last access & modified untouched (pass null)
            let ret = SetFileTime(
                handle,
                &ft as *const FILETIME,
                std::ptr::null(),
                std::ptr::null(),
            );
            if ret == 0 {
                bail!("SetFileTime failed: {}", std::io::Error::last_os_error());
            }

            // Close handle
            winapi::um::handleapi::CloseHandle(handle);
        }

        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub(super) use windows::set_birthtime;
