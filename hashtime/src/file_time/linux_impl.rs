#[cfg(target_os = "linux")]
pub(super) mod linux {
    use anyhow::{Context, Result, bail};
    use std::fs;
    use std::os::linux::fs::MetadataExt;
    use std::path::Path;
    use std::process::Command;

    fn get_block_device_for_file(path: &Path) -> Result<String> {
        let meta =
            fs::metadata(path).with_context(|| format!("failed to stat {}", path.display()))?;
        let dev_id = meta.st_dev();

        let mounts =
            fs::read_to_string("/proc/self/mounts").context("failed to read /proc/self/mounts")?;

        for line in mounts.lines() {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() < 2 {
                continue;
            }
            let device = fields[0];
            let mount_point = fields[1];

            if let Ok(mp_meta) = fs::metadata(mount_point)
                && mp_meta.st_dev() == dev_id
            {
                return Ok(device.to_string());
            }
        }

        bail!("could not find device for path {}", path.display())
    }

    fn set_birthtime_impl(path: &Path, birth_ns: i64) -> Result<()> {
        // Simple check for debugfs and root
        if !nix::unistd::Uid::effective().is_root() {
            bail!("need root to run debugfs approach");
        }

        // This implementation is intentionally conservative:
        // we don't implement all the device mapping logic here,
        // because it differs per distro; instead we detect debugfs and return helpful error.

        if which::which("debugfs").is_err() {
            bail!("debugfs not found in PATH");
        }

        let device = get_block_device_for_file(path)?;

        let birthtime_seconds = birth_ns / 1_000_000_000;
        let birthtime_nanoseconds = birth_ns % 1_000_000_000;

        let crtime_cmd = format!(
            "set_inode_field \"{}\" crtime 0x{:08x}",
            path.display(),
            birthtime_seconds
        );

        let crtime_output = Command::new("debugfs")
            .arg("-w")
            .arg("-R")
            .arg(&crtime_cmd)
            .arg(&device)
            .output()
            .with_context(|| format!("failed to run debugfs: {}", crtime_cmd))?;

        if !crtime_output.status.success() {
            let stderr = String::from_utf8_lossy(&crtime_output.stderr);
            if !stderr.contains("File not found") {
                bail!("debugfs failed (crtime): {}", stderr);
            }
        }

        let crtime_extra_value = (birthtime_nanoseconds as u32) << 2;
        let crtime_extra_cmd = format!(
            "set_inode_field \"{}\" crtime_extra 0x{:08x}",
            path.display(),
            crtime_extra_value
        );

        let crtime_extra_output = Command::new("debugfs")
            .arg("-w")
            .arg("-R")
            .arg(&crtime_extra_cmd)
            .arg(&device)
            .output()
            .with_context(|| format!("failed to run debugfs: {}", crtime_extra_cmd))?;

        if !crtime_extra_output.status.success() {
            let stderr = String::from_utf8_lossy(&crtime_extra_output.stderr);
            if !stderr.contains("File not found") {
                bail!("debugfs failed (crtime_extra): {}", stderr);
            }
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_birthtime(path: &Path, birth_ns: i64) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        // 1. Run sync
        unsafe {
            libc::sync();
        }

        set_birthtime_impl(path, birth_ns)?;

        // 2. Write '3' to drop_caches
        // OpenOptions allows us to write to the system file
        let mut file = OpenOptions::new()
            .write(true)
            .open("/proc/sys/vm/drop_caches")?;
        file.write_all(b"3\n")?;
        file.flush()?;

        Ok(())
    }

    /// Best-effort: Try to set birthtime via debugfs. This requires:
    /// 1) debugfs binary installed
    /// 2) running as root
    /// 3) mapping the file path to the underlying block device (e.g., /dev/sda2)
    ///
    /// If any of the prerequisites are missing, we return an error (caller may treat as unsupported).
    #[cfg(not(test))]
    pub(crate) fn set_birthtime(path: &Path, birth_ns: i64) -> Result<()> {
        set_birthtime_impl(path, birth_ns)
    }
}

#[cfg(target_os = "linux")]
pub(super) use linux::set_birthtime;
