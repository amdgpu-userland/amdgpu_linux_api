use std::os::fd::BorrowedFd;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};

pub type GemHandle = u32;
pub type SyncobjHandle = u32;
pub type FlinkName = u32;

pub mod amdgpu;
pub mod auth;
pub mod driver_capabilities;
pub mod gem;
mod hidden;
pub mod ioctl;
pub mod lease;
mod set_client_name;
pub mod sysfs;

use hidden::open_file_check_version;
use hidden::verify_if_drm_fd_is_authenticated;

pub use set_client_name::ClientName;
pub use set_client_name::ClientNameError;
pub use set_client_name::set_client_name;

use crate::drm::sysfs::DrmDeviceIdentity;

/// Any /dev/dri/* file
///
/// # SAFETY
/// Any file accepting DRM ioctls
pub unsafe trait DrmFile: AsFd {}

/// Any /dev/dri/card%d file
///
/// When opening a primary client it might be already a master and therefore authenticated
/// but we need to make sure.
/// Use try_from or try_into to get MasterPrimaryClient.
///
/// # SAFETY
/// Must be a primary client
pub unsafe trait DrmPrimaryFile: DrmFile {}

/// Any /dev/dri/renderD%d file
///
/// # SAFETY
/// Must be a render client
pub unsafe trait DrmRenderFile: DrmFile {}

/// Any /dev/dri/* file which is confirmed to be from amdgpu
///
/// # SAFETY
/// Must be a drm file handled by amdgpu driver
pub unsafe trait AmdgpuDrmFile: DrmFile {}

pub struct DrmDevice {}

pub struct PrimaryClient<
    Auth = auth::Unknown,
    Origin = auth::Local,
    Access = auth::Exclusive,
    Driver = amdgpu::Amdgpu,
> {
    file: OwnedFd,
    // Cannot be PhantomData because Leassed client has restricted permissions to specific objects
    _auth: Auth,
    _origin: std::marker::PhantomData<Origin>,
    _access: std::marker::PhantomData<Access>,
    _driver_specific: Driver,
    /// Cached canonicalized path to system device
    drm_device: DrmDeviceIdentity,
}

pub struct RenderClient<Driver> {
    file: OwnedFd,
    _driver_specific: Driver,
}

#[derive(Debug)]
pub enum OpenError {
    OpeningFile(std::io::Error),
    DriverVersionTooOld,
    DifferentDriverFromAmdgpu,
    Unexpected(libc::c_int),
}

impl<Auth, Origin, Access, Driver> AsFd for PrimaryClient<Auth, Origin, Access, Driver> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

/// Creating GEM objects
///
/// The resulting Gem object doesn't have to have the parameters you set here.
/// You need to check the gem's properties later.
///
/// Mmio_remap domain is not allowed
///
/// I suspect this trait will be split into separate ones for versioning
pub trait AmdgpuGemCreate: AmdgpuDrmFile {
    fn gem_create_cpu(&self, size_in_pages: usize) {
        let fd = self.as_fd().as_raw_fd();
        let mut args = ioctl::amd::GemCreate {
            input: ioctl::amd::GemCreateIn {
                bo_size: size_in_pages * 4096,
                alignment: 0,
                domains: ioctl::amd::gem_domain::CPU,
                domain_flags: 0,
            },
        };
        if let Err(e) = unsafe { ioctl::amd::gem_create(fd, &mut args) } {
            let _ = e;
            todo!()
        }
    }
    fn gem_create_gtt() {}
    fn gem_create_vram() {}
    fn gem_create_gds() {}
    fn gem_create_gws() {}
    fn gem_create_oa() {}
    fn gem_create_doorbell() {}
}

pub trait AmdgpuGemMetadata: AmdgpuDrmFile {}
