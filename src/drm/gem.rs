use std::{
    any::type_name,
    os::fd::{AsFd, AsRawFd, BorrowedFd},
};

use crate::drm::{
    FlinkName, GemHandle, PrimaryClient,
    auth::{AtLeastAuthenticated, Exclusive},
    driver_capabilities::Gem,
    ioctl,
    sysfs::DrmDeviceIdentity,
};

pub trait DrmClient: AsFd {
    type Driver;

    /// Please cache the result of this
    fn drm_device(&self) -> &DrmDeviceIdentity;
}

pub(crate) enum GemCloseError {
    DriverWithoutGem,
    InvalidHandle,
}

pub(crate) enum GemFlinkError {
    ClientNotAuthenticated,
    DriverWithoutGem,
    InvalidHandle,
    NoFreeName,
}
pub(crate) enum GemOpenError {
    ClientNotAuthenticated,
    DriverWithoutGem,
    InvalidHandle,
    NoFreeHandle,
}

pub(crate) fn gem_flink(fd: BorrowedFd<'_>, gem: GemHandle) -> Result<FlinkName, GemFlinkError> {
    let mut args = ioctl::drm::GemFlink {
        handle: gem,
        name: 0,
    };
    match unsafe { ioctl::drm::gem_flink(fd.as_raw_fd(), &mut args) } {
        Ok(_) => Ok(args.name),
        Err(libc::EACCES) => Err(GemFlinkError::ClientNotAuthenticated),
        Err(libc::EOPNOTSUPP) => Err(GemFlinkError::DriverWithoutGem),
        Err(libc::EINVAL) => Err(GemFlinkError::InvalidHandle),
        Err(libc::ENOSPC) => Err(GemFlinkError::NoFreeName),
        Err(e) => panic!("gem_flink: {e}"),
    }
}

pub(crate) fn gem_close(fd: BorrowedFd<'_>, gem: GemHandle) -> Result<(), GemCloseError> {
    let mut args = ioctl::drm::GemClose {
        handle: gem,
        pad: 0,
    };
    match unsafe { ioctl::drm::gem_close(fd.as_raw_fd(), &mut args) } {
        Ok(_) => Ok(()),
        Err(libc::EOPNOTSUPP) => Err(GemCloseError::DriverWithoutGem),
        Err(libc::EINVAL) => Err(GemCloseError::InvalidHandle),
        Err(e) => panic!("gem_close: {e}"),
    }
}

pub(crate) fn gem_open(
    fd: BorrowedFd<'_>,
    flink: FlinkName,
) -> Result<(GemHandle, usize), GemOpenError> {
    let mut args = ioctl::drm::GemOpen {
        name: flink,
        handle: 0,
        size: 0,
    };
    match unsafe { ioctl::drm::gem_open(fd.as_raw_fd(), &mut args) } {
        Ok(_) => Ok((args.handle, args.size)),
        Err(libc::EACCES) => Err(GemOpenError::ClientNotAuthenticated),
        Err(libc::EOPNOTSUPP) => Err(GemOpenError::DriverWithoutGem),
        Err(libc::ENOENT) => Err(GemOpenError::InvalidHandle),
        Err(libc::ENOSPC) => Err(GemOpenError::NoFreeHandle),
        Err(libc::ENOMEM) => panic!("gem_open: out of memory"),
        Err(e) => panic!("gem_open: {e}"),
    }
}

pub trait GemObject {
    /// A gem object has a fixed size, set during creation
    fn size(&self);
}

pub trait DrmDevice {}

pub struct GemReference<'client, Client, Driver>
where
    Client: DrmClient<Driver = Driver>,
    Driver: Gem,
{
    handle: GemHandle,
    client: &'client Client,
    size_in_bytes: usize,
}

impl<'client, Client, Driver> GemReference<'client, Client, Driver>
where
    Client: DrmClient<Driver = Driver>,
    Driver: Gem,
{
    pub fn size(&self) -> usize {
        self.size_in_bytes
    }
}

/// A flink is like a weak reference to a gem object.
/// It can be used to import this gem into another client's namespace.
/// Both exporting and importing clients must be use the same device.
/// A flink is automatically destroyed when gem object is destroyed (refcount == 0).
pub struct Flink<'dev> {
    name: FlinkName,
    /// For comparison
    source_dev: &'dev DrmDeviceIdentity,
}

impl<'client, Auth, Origin, Driver>
    GemReference<'client, PrimaryClient<Auth, Origin, Exclusive, Driver>, Driver>
where
    Driver: Gem,
    Auth: AtLeastAuthenticated,
{
    pub fn flink(&self) -> Flink<'client> {
        let source_dev = self.client.drm_device();
        match gem_flink(self.client.as_fd(), self.handle) {
            Ok(flink) => Flink {
                name: flink,
                source_dev,
            },
            Err(GemFlinkError::ClientNotAuthenticated) => client_not_authenticated(self.client),
            Err(GemFlinkError::DriverWithoutGem) => driver_without_gem::<Driver>(),
            Err(GemFlinkError::InvalidHandle) => panic!("This gem handle is supposed to be valid"),
            Err(GemFlinkError::NoFreeName) => panic!(
                "There is no free flinks left, this program or some other process created way too many"
            ),
        }
    }
}

impl<Auth, Origin, Access, Driver> PrimaryClient<Auth, Origin, Access, Driver>
where
    Auth: AtLeastAuthenticated,
    Driver: Gem,
{
    pub fn open<'client>(
        &'client self,
        flink: &Flink<'client>,
    ) -> Option<GemReference<'client, Self, Driver>> {
        if self.drm_device != *flink.source_dev {
            return None;
        }
        match gem_open(self.as_fd(), flink.name) {
            Ok((handle, size)) => Some(GemReference {
                handle,
                client: self,
                size_in_bytes: size,
            }),
            Err(GemOpenError::ClientNotAuthenticated) => client_not_authenticated(self),
            Err(GemOpenError::DriverWithoutGem) => driver_without_gem::<Driver>(),
            Err(GemOpenError::InvalidHandle) => None,
            Err(GemOpenError::NoFreeHandle) => panic!("too many gem objects"),
        }
    }
}

fn client_not_authenticated<Client: DrmClient>(fd: &Client) -> ! {
    panic!(
        "logic error: This client ({}) fd: {} is supposed to be authenticated",
        type_name::<Client>(),
        fd.as_fd().as_raw_fd()
    )
}

fn driver_without_gem<Driver>() -> ! {
    panic!(
        "logic error: The driver {} is supposed to support GEM if this gem handle is valid",
        type_name::<Driver>()
    )
}

impl<'client, Client, Driver> Drop for GemReference<'client, Client, Driver>
where
    Client: DrmClient<Driver = Driver>,
    Driver: Gem,
{
    fn drop(&mut self) {
        match gem_close(self.client.as_fd(), self.handle) {
            Ok(_) => (),
            Err(GemCloseError::DriverWithoutGem) => driver_without_gem::<Driver>(),
            Err(GemCloseError::InvalidHandle) => {
                eprintln!(
                    "somebody is messing with drm client, gem {} is supposed to be alive",
                    self.handle
                )
            }
        }
    }
}

impl<Auth, Origin, Access, Driver> DrmClient for PrimaryClient<Auth, Origin, Access, Driver> {
    type Driver = Driver;

    fn drm_device(&self) -> &DrmDeviceIdentity {
        &self.drm_device
    }
}
