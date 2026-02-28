use std::{ops::Deref, os::fd::AsRawFd};

pub type GemHandle = u32;

pub mod ioctl;

/// When oopening a primary client it might be already a master and therefore authenticated
/// but we need to make sure.
/// Use try_from or try_into to get MasterPrimaryClient.
#[derive(Debug)]
pub struct PrimaryClient {
    file: std::fs::File,
}

/// A primary client whose TID != current_tid
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

pub trait AuthenticatedPrimary {}

pub struct AuthenticatedPrimaryClient(PrimaryClient);

impl AuthenticatedPrimary for AuthenticatedPrimaryClient {}

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

impl AuthenticatedPrimary for MasterPrimaryClient {}

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
