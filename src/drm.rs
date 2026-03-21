use std::{
    ops::Deref,
    os::fd::{AsFd, AsRawFd, OwnedFd},
};

pub type GemHandle = u32;
pub type SyncobjHandle = u32;

mod hidden;
pub mod ioctl;

use hidden::open_file_check_version;
use hidden::verify_if_drm_fd_is_authenticated;

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

impl VerifyAuthenticated for AmdgpuDrmPrimary3_64 {}
impl AcquireMaster for AmdgpuDrmPrimary3_64 {}

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

/// A primary client whose TID != current_tid
///
/// It has restrictions around MASTER status
#[derive(Debug)]
pub struct ForeignPrimaryClient {}

#[derive(Debug)]
pub enum SetMasterError {
    RootPermissionsRequired,
    OtherMasterAlreadySet,
    ThisDrmClientDoesntHaveAMasterLinked,
    RunOutOfMemory,
}

pub trait VerifyAuthenticated: DrmPrimaryFile + Sized {
    fn verify(self) -> Result<Authenticated<Self>, Self> {
        let fd = self.as_fd().as_raw_fd();
        if !verify_if_drm_fd_is_authenticated(fd) {
            return Err(self);
        }
        Ok(Authenticated(self))
    }
}

pub trait AcquireMaster: DrmPrimaryFile + Sized {
    fn acquire(self) -> Result<Master<Self>, (Self, SetMasterError)> {
        if let Err(e) = unsafe { ioctl::drm::set_master(self.as_fd().as_raw_fd()) } {
            let err = match e {
                libc::EACCES => SetMasterError::RootPermissionsRequired,
                libc::EBUSY => SetMasterError::OtherMasterAlreadySet,
                libc::EINVAL => SetMasterError::ThisDrmClientDoesntHaveAMasterLinked,
                libc::ENOMEM => SetMasterError::RunOutOfMemory,
                _ => todo!("set_master: {e}"),
            };
            return Err((self, err));
        }
        Ok(Master(self))
    }
}

pub struct Master<T: DrmPrimaryFile + Sized>(T);

impl<T: DrmPrimaryFile> Deref for Master<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DrmPrimaryFile + Sized> Master<T> {
    pub fn drop_master(self) -> Authenticated<T> {
        if let Err(e) = unsafe { ioctl::drm::drop_master(self.0.as_fd().as_raw_fd()) } {
            panic!("Unexpected drop_master: {e}");
        }

        let Master(inner) = self;
        Authenticated(inner)
    }
}

pub struct Authenticated<T: DrmPrimaryFile>(T);

impl<T: DrmPrimaryFile> Deref for Authenticated<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T: AmdgpuDrmFile + DrmPrimaryFile> AmdgpuDrmFile for Authenticated<T> {}
unsafe impl<T: DrmPrimaryFile> DrmPrimaryFile for Authenticated<T> {}
unsafe impl<T: DrmPrimaryFile> DrmFile for Authenticated<T> {}
impl<T: DrmPrimaryFile> AsFd for Authenticated<T> {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.0.as_fd()
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

impl AmdgpuGemCreate for Authenticated<AmdgpuDrmPrimary3_64> {}
impl AmdgpuGemCreate for AmdgpuDrmRender3_64 {}

pub trait AmdgpuGemMetadata: AmdgpuDrmFile {}
