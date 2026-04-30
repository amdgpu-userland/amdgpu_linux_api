use std::os::fd::{AsFd, AsRawFd, OwnedFd};

pub type GemHandle = u32;
pub type SyncobjHandle = u32;

pub mod auth;
mod hidden;
pub mod ioctl;
mod set_client_name;

use hidden::open_file_check_version;
use hidden::verify_if_drm_fd_is_authenticated;

pub use set_client_name::ClientName;
pub use set_client_name::ClientNameError;
pub use set_client_name::set_client_name;

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

pub struct PrimaryClient<Auth, Origin, Access> {
    file: OwnedFd,
    // Cannot be PhantomData because Leassed client has restricted permissions to specific objects
    _auth: Auth,
    _origin: std::marker::PhantomData<Origin>,
    _access: std::marker::PhantomData<Access>,
}

pub struct AmdgpuDrmRender3_64 {
    fd: OwnedFd,
}
unsafe impl AmdgpuDrmFile for AmdgpuDrmRender3_64 {}
unsafe impl DrmRenderFile for AmdgpuDrmRender3_64 {}
unsafe impl DrmFile for AmdgpuDrmRender3_64 {}

impl AmdgpuDrmRender3_64 {
    pub fn open(number: i32) -> Result<Self, OpenError> {
        Ok(Self {
            fd: open_file_check_version(format!("/dev/dri/renderD{number}"), 3, 64)?,
        })
    }
}

impl AsFd for AmdgpuDrmRender3_64 {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

pub struct AmdgpuDrmPrimary3_64 {
    fd: OwnedFd,
}

#[derive(Debug)]
pub enum OpenError {
    OpeningFile(std::io::Error),
    DriverVersionTooOld,
    DifferentDriverFromAmdgpu,
    Unexpected(libc::c_int),
}

impl AmdgpuDrmPrimary3_64 {
    pub fn open(num: i32) -> Result<Self, OpenError> {
        Ok(Self {
            fd: open_file_check_version(format!("/dev/dri/card{num}"), 3, 64)?,
        })
    }
}

impl AsFd for AmdgpuDrmPrimary3_64 {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

unsafe impl DrmFile for AmdgpuDrmPrimary3_64 {}
unsafe impl DrmPrimaryFile for AmdgpuDrmPrimary3_64 {}
unsafe impl AmdgpuDrmFile for AmdgpuDrmPrimary3_64 {}

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

impl AmdgpuGemCreate for AmdgpuDrmRender3_64 {}

pub trait AmdgpuGemMetadata: AmdgpuDrmFile {}
