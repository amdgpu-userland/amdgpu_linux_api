use std::os::fd::AsRawFd;

pub const KFD_FILE_PATH: &str = "/dev/kfd";

pub struct Kfd {
    file: std::fs::File,
    version: KfdVersion,
}

impl Kfd {
    pub fn open() -> std::io::Result<Self> {
        let file = std::fs::File::open(KFD_FILE_PATH)?;

        // Let's do version ioctl to check if we got the right file
        // also we can cache the result, because the module cannot be unloaded
        // while the kfd file is still in use
        let version = unsafe { amdkfd_ioc_get_version(file.as_raw_fd()) };

        Ok(Self { file, version })
    }

    pub fn version(&self) -> &KfdVersion {
        &self.version
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdVersion {
    pub major_version: u32,
    pub minor_version: u32,
}

const AMDKFD_IOCTL_BASE: u32 = 'K' as u32;

const AMDKFD_IOC_GET_VERSION: libc::Ioctl = libc::_IOR::<KfdVersion>(AMDKFD_IOCTL_BASE, 0x1);
unsafe fn amdkfd_ioc_get_version(fd: libc::c_int) -> KfdVersion {
    let mut out = KfdVersion::default();
    let res = unsafe { libc::ioctl(fd, AMDKFD_IOC_GET_VERSION, &raw mut out) };
    if res != 0 {
        todo!("error");
    }
    out
}
