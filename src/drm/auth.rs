use std::marker::PhantomData;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;
use std::os::fd::OwnedFd;

use crate::drm::verify_if_drm_fd_is_authenticated;
use errors::*;

use super::PrimaryClient;
use super::ioctl;

pub mod errors {
    #[derive(Debug)]
    #[non_exhaustive]
    pub enum SetMasterError {
        /// This client was never a master or is foreign so it requires root permissions
        RequiresRootPermissions,
        OtherMasterAlreadySet,
        /// Modyfing master state not allowed for leassed clients
        ///
        /// This only makes sense to come from attempting to overwrite a
        /// drm_lease, becaues the other check is not possible because this ioctl is not
        /// available to Render clients
        LeassedClientNotAllowed,
        RunOutOfMemory,
    }

    #[non_exhaustive]
    pub enum RegularSetMasterError {
        /// Either:
        /// - this client was never a master therefore it would require root permissions
        ///   so it is either a regular client or authenticated via magic value
        /// - or it is a foreign client and would require master regardless
        RootPermissionsRequired,
        OtherMasterAlreadySet,
    }

    pub enum DropMasterError {
        /// This client was never a master or is foreign so it requires root permissions
        RootAccessRequired,
        NotCurrentMasterOrThereIsNoMasterOrItIsALeassedClient,
    }

    pub struct OtherMasterAlreadySet;
    pub struct RootAccessRequired;
}

/// Created by this thread group
pub struct Local;

/// Created outside current thread group, but we have sole ownership of it now
pub struct Foreign;

/// Sole access to underlying drm object, no dupplicates exist
pub struct Exclusive;

/// The underlying object has multiple linked file descriptors
pub struct Shared;

/// There can be only one at a time, but it can leasse access to some objects to other primary
/// clients
#[doc = include_str!("./auth_master_warning.md")]
pub struct Master;

/// A master which created leasses
pub struct MasterWithLeasses {
    _leasses: Vec<()>,
}

/// Authenticated probably by a master via a magic value (DRM_IOCTL_AUTH_MAGIC)
pub struct Authenticated;

/// Authenticated definitely because this client droped master status
pub struct WasMaster;

/// A starting auth state, it can be anything
pub struct Unknown;

/// No additional permissions
pub struct Regular;

/// Master like status, but restricted to specific objects
/// Created via DRM_IOCTL_MODE_CREATE_LEASE
pub struct Leased {
    _permitted_objects: (),
}

/// Once obtained, authenticated status cannot be removed
pub trait AtLeastAuthenticated {}
impl AtLeastAuthenticated for Master {}
impl AtLeastAuthenticated for MasterWithLeasses {}
impl AtLeastAuthenticated for Authenticated {}
impl AtLeastAuthenticated for Leased {}
impl AtLeastAuthenticated for WasMaster {}

/// Only applicable to primary clients because of ioctl's flags
fn set_master(fd: BorrowedFd<'_>) -> Result<(), SetMasterError> {
    if let Err(e) = unsafe { ioctl::drm::set_master(fd.as_raw_fd()) } {
        let err = match e {
            libc::EACCES => SetMasterError::RequiresRootPermissions,
            libc::EBUSY => SetMasterError::OtherMasterAlreadySet,
            libc::EINVAL => SetMasterError::LeassedClientNotAllowed,
            libc::ENOMEM => SetMasterError::RunOutOfMemory,
            _ => todo!("set_master: {e}"),
        };
        return Err(err);
    }
    Ok(())
}

impl<Origin> PrimaryClient<Unknown, Origin, Exclusive> {
    pub fn set_master(
        self,
    ) -> Result<PrimaryClient<Master, Origin, Exclusive>, (Self, RegularSetMasterError)> {
        match set_master(self.file.as_fd()) {
            Ok(_) => Ok(PrimaryClient {
                _auth: Master,
                _origin: PhantomData,
                _access: PhantomData,
                file: self.file,
            }),
            Err(SetMasterError::OtherMasterAlreadySet) => {
                Err((self, RegularSetMasterError::OtherMasterAlreadySet))
            }
            Err(SetMasterError::LeassedClientNotAllowed) => {
                panic!("Logic error: this drm client is not supposed to be a leasse")
            }
            Err(SetMasterError::RunOutOfMemory) => panic!("Out of memory"),
            Err(SetMasterError::RequiresRootPermissions) => {
                Err((self, RegularSetMasterError::RootPermissionsRequired))
            }
        }
    }
}

impl PrimaryClient<WasMaster, Local, Exclusive> {
    pub fn set_master(
        self,
    ) -> Result<PrimaryClient<Master, Local, Exclusive>, (Self, OtherMasterAlreadySet)> {
        match set_master(self.file.as_fd()) {
            Ok(_) => Ok(PrimaryClient {
                _auth: Master,
                _origin: PhantomData,
                _access: PhantomData,
                file: self.file,
            }),
            Err(SetMasterError::OtherMasterAlreadySet) => Err((self, OtherMasterAlreadySet)),
            Err(SetMasterError::LeassedClientNotAllowed) => {
                panic!("Logic error: this drm client is not supposed to be a leasse")
            }
            Err(SetMasterError::RunOutOfMemory) => panic!("Out of memory"),
            Err(SetMasterError::RequiresRootPermissions) => panic!(
                "Logic error: this drm client is supposed to be created by the current thread"
            ),
        }
    }
}

impl<O, A> PrimaryClient<Unknown, O, A> {
    /// If you can expect this client to be already authenticated you can verify it in only one
    /// syscall
    ///
    /// A primary client can be automatically master, which automatically sets authenticated status
    /// If you can expect only need authenticated status use
    /// this instead of going through master
    pub fn verify_authenticated(self) -> Result<PrimaryClient<Authenticated, O, A>, Self> {
        let fd = self.file.as_raw_fd();
        if !verify_if_drm_fd_is_authenticated(fd) {
            return Err(self);
        }
        Ok(PrimaryClient {
            file: self.file,
            _auth: Authenticated,
            _origin: PhantomData,
            _access: PhantomData,
        })
    }
}

fn drop_master(fd: &mut OwnedFd) -> Result<(), DropMasterError> {
    match unsafe { ioctl::drm::drop_master(fd.as_raw_fd()) } {
        Ok(r) => Ok(r),
        Err(libc::EACCES) => Err(DropMasterError::RootAccessRequired),
        Err(libc::EINVAL) => {
            Err(DropMasterError::NotCurrentMasterOrThereIsNoMasterOrItIsALeassedClient)
        }
        Err(e) => panic!("Unexpected drop_master: {e}"),
    }
}

impl PrimaryClient<Master, Local, Exclusive> {
    #[doc = include_str!("./auth_master_warning.md")]
    pub fn drop_master(mut self) -> PrimaryClient<WasMaster, Local, Exclusive> {
        match drop_master(&mut self.file) {
            Ok(_) => PrimaryClient {
                file: self.file,
                _auth: WasMaster,
                _origin: PhantomData,
                _access: PhantomData,
            },
            Err(DropMasterError::RootAccessRequired) => {
                panic!(
                    "Logic error: this client should have been master at some point and be from current thread group"
                )
            }
            Err(DropMasterError::NotCurrentMasterOrThereIsNoMasterOrItIsALeassedClient) => {
                PrimaryClient {
                    file: self.file,
                    _auth: WasMaster,
                    _origin: PhantomData,
                    _access: PhantomData,
                }
            }
        }
    }
}

impl PrimaryClient<Master, Foreign, Exclusive> {
    #[doc = include_str!("./auth_master_warning.md")]
    pub fn drop_master(
        mut self,
    ) -> Result<PrimaryClient<WasMaster, Foreign, Exclusive>, (Self, RootAccessRequired)> {
        match drop_master(&mut self.file) {
            Ok(_) => Ok(PrimaryClient {
                file: self.file,
                _auth: WasMaster,
                _origin: PhantomData,
                _access: PhantomData,
            }),
            Err(DropMasterError::RootAccessRequired) => Err((self, RootAccessRequired)),
            Err(DropMasterError::NotCurrentMasterOrThereIsNoMasterOrItIsALeassedClient) => {
                Ok(PrimaryClient {
                    file: self.file,
                    _auth: WasMaster,
                    _origin: PhantomData,
                    _access: PhantomData,
                })
            }
        }
    }
}
