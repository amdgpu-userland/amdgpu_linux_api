use std::{
    os::fd::{AsRawFd, OwnedFd, RawFd},
    path::Path,
};

use crate::drm::{OpenError, ioctl};

pub(super) fn open_file_check_version<P: AsRef<Path>>(path: P) -> Result<OwnedFd, OpenError> {
    let file = std::fs::File::open(path).map_err(OpenError::OpeningFile)?;

    let mut str_buffer = [0u8; 4096];
    let (driver_name, rest) = str_buffer.split_at_mut(1024);
    let (date, desc) = rest.split_at_mut(1024);
    let mut args = ioctl::DrmVersion {
        major: 0,
        minor: 0,
        patchlevel: 0,
        name: driver_name.as_mut_ptr(),
        name_len: driver_name.len(),
        date: date.as_mut_ptr(),
        date_len: date.len(),
        desc: desc.as_mut_ptr(),
        desc_len: desc.len(),
    };
    if let Err(e) = unsafe { ioctl::drm_ioctl_version(file.as_raw_fd(), &mut args) } {
        return Err(OpenError::Unexpected(e));
    }
    if args.major < 3 && args.minor < 64 {
        return Err(OpenError::DriverVersionTooOld);
    }

    let driver_name = str::from_utf8(&driver_name[..args.name_len])
        .expect("Linux returend driver name to be UTF-8 compatible");
    if "amdgpu" != driver_name {
        return Err(OpenError::DifferentDriverFromAmdgpu);
    }

    Ok(file.into())
}

pub(super) fn verify_if_drm_fd_is_authenticated(fd: RawFd) -> bool {
    let mut args = ioctl::DrmClient::default();
    let res = unsafe { ioctl::drm_ioctl_get_client(fd, &mut args) };
    debug_assert!(res.is_ok());

    args.auth != 0
}
