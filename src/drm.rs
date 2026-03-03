use std::{
    ffi::{OsStr, OsString},
    ops::Deref,
    os::{
        fd::{AsFd, AsRawFd, OwnedFd, RawFd},
        unix::ffi::OsStrExt,
    },
};

pub type GemHandle = u32;

pub mod ioctl;

/// Any /dev/dri/* file
pub unsafe trait DrmFile: AsFd {}

/// Any /dev/dri/card%d file
pub unsafe trait DrmPrimaryFile: DrmFile {}

/// Any /dev/dri/renderD%d file
pub unsafe trait DrmRenderFile: DrmFile {}

/// Any /dev/dri/* file which is confirmed to be from amdgpu
pub unsafe trait AmdgpuDrmFile: DrmFile {}

pub struct AmdgpuDrmRender3_64 {
    fd: OwnedFd,
}

pub struct AmdgpuDrmPrimary3_64 {
    fd: OwnedFd,
}

pub enum OpenError {
    OpeningFile(std::io::Error),
    DriverVersionTooOld,
    DifferentDriverFromAmdgpu,
    Unexpected(libc::c_int),
}

impl AmdgpuDrmPrimary3_64 {
    pub fn open(num: i32) -> Result<Self, OpenError> {
        let file =
            std::fs::File::open(format!("/dev/dri/card{num}")).map_err(OpenError::OpeningFile)?;

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
        if "amdgpu" != OsStr::from_bytes(driver_name).to_string_lossy() {
            return Err(OpenError::DifferentDriverFromAmdgpu);
        }

        Ok(Self { fd: file.into() })
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

/// When oopening a primary client it might be already a master and therefore authenticated
/// but we need to make sure.
/// Use try_from or try_into to get MasterPrimaryClient.
#[derive(Debug)]
pub struct PrimaryClient {
    file: std::fs::File,
}

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

impl TryFrom<PrimaryClient> for MasterPrimaryClient {
    type Error = (PrimaryClient, SetMasterError);

    fn try_from(value: PrimaryClient) -> Result<Self, Self::Error> {
        if let Err(e) = unsafe { ioctl::drm_ioctl_set_master(value.file.as_raw_fd()) } {
            let err = match e {
                libc::EACCES => SetMasterError::RootPermissionsRequired,
                libc::EBUSY => SetMasterError::OtherMasterAlreadySet,
                libc::EINVAL => SetMasterError::ThisDrmClientDoesntHaveAMasterLinked,
                libc::ENOMEM => SetMasterError::RunOutOfMemory,
                _ => todo!("set_master: {e}"),
            };
            return Err((value, err));
        }
        Ok(Self(value))
    }
}

pub fn verify_if_drm_fd_is_authenticated(fd: RawFd) -> bool {
    let mut args = ioctl::DrmClient::default();
    let res = unsafe { ioctl::drm_ioctl_get_client(fd, &mut args) };
    debug_assert!(res.is_ok());

    args.auth != 0
}

/// amdgpu specific && DRM_AUTH | DRM_RENDER_ALLOW
pub trait AmdgpuAuthenticatedRender {
    //fn create_gem();
}

pub trait AuthenticatedClient {}

pub struct AuthenticatedPrimaryClient(PrimaryClient);

impl AuthenticatedClient for AuthenticatedPrimaryClient {}

impl Deref for AuthenticatedPrimaryClient {
    type Target = PrimaryClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<MasterPrimaryClient> for AuthenticatedPrimaryClient {
    fn from(value: MasterPrimaryClient) -> Self {
        if let Err(e) = unsafe { ioctl::drm_ioctl_drop_master(value.file.as_raw_fd()) } {
            match e {
                libc::EACCES | libc::EINVAL => panic!(
                    "MasterPrimaryNode was supposed to be current master and belong to the current thread group"
                ),
                _ => todo!("drop_master: {e}"),
            };
        }
        let MasterPrimaryClient(client) = value;
        Self(client)
    }
}

#[derive(Debug)]
pub struct MasterPrimaryClient(PrimaryClient);

impl AuthenticatedClient for MasterPrimaryClient {}

impl Deref for MasterPrimaryClient {
    type Target = PrimaryClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for PrimaryClient {
    fn drop(&mut self) {
        let _ = unsafe { ioctl::drm_ioctl_drop_master(self.file.as_raw_fd()) };
    }
}

#[derive(Debug)]
pub struct RenderClient {
    _file: std::fs::File,
}

impl RenderClient {
    pub fn open(number: i32) -> Result<Self, ()> {
        let file = match std::fs::File::open(format!("/dev/dri/renderD{}", number)) {
            Ok(f) => f,
            Err(_) => todo!(),
        };
        Ok(Self { _file: file })
    }
}

pub struct AuthenticatedRenderNode {}

pub struct Gem {
    //handle: GemHandle,
}
