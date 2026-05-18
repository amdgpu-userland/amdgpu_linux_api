use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::Path;
use std::{fs, io};
use std::{os::fd::AsRawFd, path::PathBuf};

pub const DRM_CLASS: &str = "/sys/class/drm";
pub const DEV_DRI: &str = "/dev/dri";
pub const DEBUGFS_DRI: &str = "/sys/kernel/debug/dri";
const SYS_DEV_CHAR: &str = "/sys/dev/char";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DrmDeviceIdentity {
    sysfs_device: PathBuf,
}

impl DrmDeviceIdentity {
    pub fn from_fd(fd: &(impl AsRawFd + ?Sized)) -> io::Result<Self> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let ret = unsafe { libc::fstat(fd.as_raw_fd(), stat.as_mut_ptr()) };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }

        let stat = unsafe { stat.assume_init() };
        if stat.st_mode & libc::S_IFMT != libc::S_IFCHR {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "file descriptor is not a character device",
            ));
        }

        identity_from_dev_t(stat.st_rdev)
    }

    pub fn from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        if !metadata.file_type().is_char_device() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "path is not a character device",
            ));
        }

        identity_from_dev_t(metadata.rdev())
    }

    pub fn same_device(&self, other: &Self) -> bool {
        self == other
    }

    pub fn sysfs_device(&self) -> &Path {
        &self.sysfs_device
    }
}

fn identity_from_dev_t(dev_t: libc::dev_t) -> io::Result<DrmDeviceIdentity> {
    identity_from_dev_t_with_sys_dev_char(dev_t, Path::new(SYS_DEV_CHAR))
}

fn identity_from_dev_t_with_sys_dev_char(
    dev_t: libc::dev_t,
    sys_dev_char: &Path,
) -> io::Result<DrmDeviceIdentity> {
    let drm_node = sys_dev_char.join(format!(
        "{}:{}",
        linux_dev_major(dev_t),
        linux_dev_minor(dev_t)
    ));
    let sysfs_device = fs::canonicalize(drm_node.join("device"))?;

    Ok(DrmDeviceIdentity { sysfs_device })
}

fn linux_dev_major(dev_t: libc::dev_t) -> u32 {
    let dev_t = dev_t as u64;
    (((dev_t >> 8) & 0xfff) | ((dev_t >> 32) & !0xfff)) as u32
}

fn linux_dev_minor(dev_t: libc::dev_t) -> u32 {
    let dev_t = dev_t as u64;
    ((dev_t & 0xff) | ((dev_t >> 12) & !0xff)) as u32
}
