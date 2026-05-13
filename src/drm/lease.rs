use crate::drm::{PrimaryClient, auth::Leased, ioctl::drm as ioctl};
use std::os::fd::{AsRawFd, BorrowedFd};

pub(crate) enum GetLeaseError {
    /// The lessor is no longer master, but it could become master again and the lease would be
    /// valid again
    LostMasterStatus,
    DriverDoesntSupport,
}

pub(crate) fn get_lease<'buffer>(
    fd: BorrowedFd<'_>,
    buffer: &'buffer mut [u32],
) -> Result<&'buffer mut [u32], GetLeaseError> {
    let mut args = ioctl::GetLease {
        count_objects: buffer.len().try_into().unwrap(),
        pad: 0,
        objects_ptr: buffer.as_mut_ptr(),
    };
    match unsafe { ioctl::mode_get_lease(fd.as_raw_fd(), &mut args) } {
        Ok(_) => Ok(&mut buffer[..(args.count_objects.try_into().unwrap())]),
        Err(libc::EACCES) => Err(GetLeaseError::LostMasterStatus),
        Err(libc::EOPNOTSUPP) => Err(GetLeaseError::DriverDoesntSupport),
        Err(e) => panic!("mode_get_lease: {e}"),
    }
}
