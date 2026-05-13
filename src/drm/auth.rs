use std::marker::PhantomData;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::BorrowedFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

use crate::capabilities::ActiveCaps;
use crate::capabilities::DisabledCaps;
use crate::drm::driver_capabilities::Modeset;
use crate::drm::ioctl::drm::Magic;
use crate::drm::lease::GetLeaseError;
use crate::drm::lease::get_lease;
use crate::drm::verify_if_drm_fd_is_authenticated;
use errors::*;

use super::PrimaryClient;
use super::ioctl;

use crate::capabilities::CAP_SYS_ADMIN;

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

    pub struct ClientDoesntExistOrAlreadyAuthenticated;
}

/// Created by this thread group
pub struct Local;

/// Created outside current thread group
pub struct Foreign;

/// Sole access to underlying drm object, no dupplicates exist
pub struct Exclusive;

/// The underlying object has multiple linked file descriptors
/// so the program needs to be careful around modifying state
pub struct Shared;

/// There can be only one at a time, but it can leasse access to some objects to other primary
/// clients
#[doc = include_str!("./auth_master_warning.md")]
pub struct Master;

/// A master which created leasses
///
/// Closing it or dropping master status is dangerous because it affects the leassed clients
pub struct MasterWithLeasses {
    _leasses: Vec<()>,
}

/// Authenticated **probably** by a master via a magic value (DRM_IOCTL_AUTH_MAGIC)
///
/// If has CAP_SYS_ADMIN at creation, it is automatically set
pub struct Authenticated;

/// Authenticated definitely because this client droped master status
pub struct WasMaster;

/// A starting auth state, it can be anything
pub struct Unknown;

/// No additional permissions
pub struct Regular;

/// Master like status, but restricted to specific objects
/// Created via DRM_IOCTL_MODE_CREATE_LEASE
///
/// Leased objects can be revoked but the leased client doesn't get invalidated
///
/// But it looses master like status
/// Also closing or droping master status by the lessor revokes master like status
/// and revokes_leases
pub struct Leased<'master> {
    _permitted_objects: &'master [u32],
}

/// Once obtained, authenticated status cannot be removed
pub trait AtLeastAuthenticated {}
impl AtLeastAuthenticated for Master {}
impl AtLeastAuthenticated for MasterWithLeasses {}
impl AtLeastAuthenticated for Authenticated {}
impl AtLeastAuthenticated for Leased<'_> {}
impl AtLeastAuthenticated for WasMaster {}

pub trait ProbablyNotAuthenticated {}
impl ProbablyNotAuthenticated for Regular {}
impl ProbablyNotAuthenticated for Unknown {}

pub trait PrimaryMaster {}
impl PrimaryMaster for Master {}
impl PrimaryMaster for MasterWithLeasses {}

pub trait MasterLike {}
impl MasterLike for Master {}
impl MasterLike for MasterWithLeasses {}
impl MasterLike for Leased<'_> {}

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

pub enum RootlessLocalSetMasterError<Driver> {
    Authenticated(PrimaryClient<Authenticated, Local, Exclusive, Driver>),
    Regular(PrimaryClient<Regular, Local, Exclusive, Driver>),
    OtherMasterAlreadySet(PrimaryClient<Unknown, Local, Exclusive, Driver>),
}

impl<Driver> PrimaryClient<Unknown, Local, Exclusive, Driver> {
    pub fn set_master_rootless(
        self,
        _token: &DisabledCaps<CAP_SYS_ADMIN>,
    ) -> Result<PrimaryClient<Master, Local, Exclusive, Driver>, RootlessLocalSetMasterError<Driver>>
    {
        match set_master(self.file.as_fd()) {
            Ok(_) => Ok(PrimaryClient {
                _auth: Master,
                _origin: PhantomData,
                _access: PhantomData,
                file: self.file,
                _driver_specific: self._driver_specific,
            }),
            Err(SetMasterError::OtherMasterAlreadySet) => {
                Err(RootlessLocalSetMasterError::OtherMasterAlreadySet(self))
            }
            Err(SetMasterError::LeassedClientNotAllowed) => {
                panic!(
                    "Logic error: since this client is created locally and has unknown auth it cannot be a leasse"
                )
            }
            Err(SetMasterError::RunOutOfMemory) => panic!("Out of memory"),
            Err(SetMasterError::RequiresRootPermissions) => match self.verify_authenticated() {
                Ok(res) => Err(RootlessLocalSetMasterError::Authenticated(res)),
                Err(res) => Err(RootlessLocalSetMasterError::Regular(PrimaryClient {
                    file: res.file,
                    _auth: Regular,
                    _origin: PhantomData,
                    _access: PhantomData,
                    _driver_specific: res._driver_specific,
                })),
            },
        }
    }

    pub fn set_master(
        self,
        _token: &ActiveCaps<CAP_SYS_ADMIN>,
    ) -> Result<PrimaryClient<Master, Local, Exclusive, Driver>, (Self, OtherMasterAlreadySet)>
    {
        match set_master(self.file.as_fd()) {
            Ok(_) => Ok(PrimaryClient {
                _auth: Master,
                _origin: PhantomData,
                _access: PhantomData,
                file: self.file,
                _driver_specific: self._driver_specific,
            }),
            Err(SetMasterError::OtherMasterAlreadySet) => Err((self, OtherMasterAlreadySet)),
            Err(SetMasterError::LeassedClientNotAllowed) => {
                panic!(
                    "Logic error: since this client is created locally and has unknown auth it cannot be a leasse"
                )
            }
            Err(SetMasterError::RunOutOfMemory) => panic!("Out of memory"),
            Err(SetMasterError::RequiresRootPermissions) => {
                panic!("This thread is supposed to have CAP_SYS_ADMIN via token")
            }
        }
    }
}

pub enum ForeignSetMasterError<Driver> {
    /// Because it comes from outside this program, we cannot determine for how long the lease is
    /// going to last. In optimistic case for the whole duration of this program so 'static.
    LeassedClient(PrimaryClient<Leased<'static>, Foreign, Exclusive, Driver>),
    OtherMasterAlreadySet(PrimaryClient<Unknown, Foreign, Exclusive, Driver>),
}

impl<Driver> PrimaryClient<Unknown, Foreign, Exclusive, Driver> {
    pub fn set_master(
        self,
        _token: &ActiveCaps<CAP_SYS_ADMIN>,
    ) -> Result<PrimaryClient<Master, Local, Exclusive, Driver>, ForeignSetMasterError<Driver>>
    {
        match set_master(self.file.as_fd()) {
            Ok(_) => Ok(PrimaryClient {
                _auth: Master,
                _origin: PhantomData,
                _access: PhantomData,
                file: self.file,
                _driver_specific: self._driver_specific,
            }),
            Err(SetMasterError::OtherMasterAlreadySet) => {
                Err(ForeignSetMasterError::OtherMasterAlreadySet(self))
            }
            Err(SetMasterError::LeassedClientNotAllowed) => {
                let buffer = Box::leak(Box::new([0u32; 1024]));
                match get_lease(self.file.as_fd(), buffer.as_mut_slice()) {
                    Ok(buffer) => {
                        Err(ForeignSetMasterError::LeassedClient(PrimaryClient {
                            file: self.file,
                            _auth: Leased {
                                // We will have to remember to:
                                // - zero this memory if the lease has been revoked - Foreign + Lease<'static>
                                // - free this memory if the lease loses master status
                                _permitted_objects: buffer,
                            },
                            _origin: PhantomData,
                            _access: PhantomData,
                            _driver_specific: self._driver_specific,
                        }))
                    }
                    Err(GetLeaseError::DriverDoesntSupport) => unreachable!(
                        "Previous error told us this is a leased object, so modesetting is supported by this driver"
                    ),
                    Err(GetLeaseError::LostMasterStatus) => panic!(
                        "Lease waiting for lessor to get master status back is too complex to bother with"
                    ),
                }
            }
            Err(SetMasterError::RunOutOfMemory) => panic!("Out of memory"),
            Err(SetMasterError::RequiresRootPermissions) => {
                panic!("This thread is supposed to have CAP_SYS_ADMIN via token")
            }
        }
    }
}

impl<Driver> PrimaryClient<WasMaster, Local, Exclusive, Driver> {
    pub fn set_master(
        self,
    ) -> Result<PrimaryClient<Master, Local, Exclusive, Driver>, (Self, OtherMasterAlreadySet)>
    {
        match set_master(self.file.as_fd()) {
            Ok(_) => Ok(PrimaryClient {
                _auth: Master,
                _origin: PhantomData,
                _access: PhantomData,
                file: self.file,
                _driver_specific: self._driver_specific,
            }),
            Err(SetMasterError::OtherMasterAlreadySet) => Err((self, OtherMasterAlreadySet)),
            Err(SetMasterError::LeassedClientNotAllowed) => {
                panic!("Logic error: this drm client is not supposed to be a leasse")
            }
            Err(SetMasterError::RunOutOfMemory) => panic!("Out of memory"),
            Err(SetMasterError::RequiresRootPermissions) => panic!(
                "Logic error: this drm client is supposed to be created by the current thread and therefore not require root"
            ),
        }
    }
}

impl<Driver> PrimaryClient<Master, Local, Exclusive, Driver> {
    #[doc = include_str!("./auth_master_warning.md")]
    pub fn drop_master(mut self) -> PrimaryClient<WasMaster, Local, Exclusive, Driver> {
        match drop_master(&mut self.file) {
            Ok(_) => PrimaryClient {
                file: self.file,
                _driver_specific: self._driver_specific,
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
                    _driver_specific: self._driver_specific,
                    _auth: WasMaster,
                    _origin: PhantomData,
                    _access: PhantomData,
                }
            }
        }
    }
}

impl<Driver> PrimaryClient<Master, Foreign, Exclusive, Driver> {
    #[doc = include_str!("./auth_master_warning.md")]
    pub fn drop_master(
        mut self,
        _token: &ActiveCaps<CAP_SYS_ADMIN>,
    ) -> PrimaryClient<WasMaster, Foreign, Exclusive, Driver> {
        match drop_master(&mut self.file) {
            Ok(_) => PrimaryClient {
                file: self.file,
                _driver_specific: self._driver_specific,
                _auth: WasMaster,
                _origin: PhantomData,
                _access: PhantomData,
            },
            Err(DropMasterError::RootAccessRequired) => {
                panic!("We are supposed to have root access, via token");
            }
            Err(DropMasterError::NotCurrentMasterOrThereIsNoMasterOrItIsALeassedClient) => {
                PrimaryClient {
                    file: self.file,
                    _driver_specific: self._driver_specific,
                    _auth: WasMaster,
                    _origin: PhantomData,
                    _access: PhantomData,
                }
            }
        }
    }
}

impl<O, A, D> PrimaryClient<Unknown, O, A, D> {
    /// If you can expect this client to be already authenticated you can verify it in only one
    /// syscall, but it doesn't tell you how you got there.
    ///
    /// A primary client can be automatically master, which automatically sets authenticated status
    /// If you can expect only need authenticated status use
    /// this instead of going through master
    pub fn verify_authenticated(self) -> Result<PrimaryClient<Authenticated, O, A, D>, Self> {
        let fd = self.file.as_raw_fd();
        if !verify_if_drm_fd_is_authenticated(fd) {
            return Err(self);
        }
        Ok(PrimaryClient {
            file: self.file,
            _driver_specific: self._driver_specific,
            _auth: Authenticated,
            _origin: PhantomData,
            _access: PhantomData,
        })
    }
}

impl<Auth: ProbablyNotAuthenticated, Origin, Driver>
    PrimaryClient<Auth, Origin, Exclusive, Driver>
{
    /// After authenticating with master use `verify_authenticated` to cache state appropriately
    pub fn get_magic(&self) -> Magic {
        let mut args = ioctl::drm::Auth::default();
        match unsafe { ioctl::drm::get_magic(self.file.as_raw_fd(), &mut args) } {
            Ok(_) => args.magic,
            Err(e) => panic!("get_magic: {e}"),
        }
    }
}

impl<Auth: PrimaryMaster, Origin, Driver> PrimaryClient<Auth, Origin, Exclusive, Driver> {
    pub fn auth_magic(&self, magic: Magic) -> Result<(), ClientDoesntExistOrAlreadyAuthenticated> {
        let mut args = ioctl::drm::Auth { magic };
        match unsafe { ioctl::drm::auth_magic(self.file.as_raw_fd(), &mut args) } {
            Ok(_) => Ok(()),
            Err(libc::EINVAL) => Err(ClientDoesntExistOrAlreadyAuthenticated),
            Err(e) => panic!("auth_magic: {e}"),
        }
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

impl<Origin, Driver: Modeset + Default> PrimaryClient<Master, Origin, Exclusive, Driver> {
    /// # SAFETY
    /// When passing the leased client remember that at the end of the given closure access to
    /// leased objects will be revoked and it might loose master status
    pub fn create_lease_revoke_at_end<'lease>(
        &'lease self,
        leased_objs: &'lease [u32],
        f: impl FnOnce(PrimaryClient<Leased<'lease>, Local, Exclusive, Driver>),
    ) {
        let mut args = ioctl::drm::CreateLease {
            object_ids: leased_objs.as_ptr(),
            object_count: leased_objs.len().try_into().unwrap(),
            flags: libc::O_CLOEXEC,
            lessee_id: 0,
            fd: 0,
        };
        match unsafe { ioctl::drm::mode_create_lease(self.file.as_raw_fd(), &mut args) } {
            Ok(_) => (),
            Err(e) => panic!("create_lease: {e}"),
        }
        let lessee_id = args.lessee_id;
        let lease = PrimaryClient {
            file: unsafe { OwnedFd::from_raw_fd(args.fd) },
            _auth: Leased {
                _permitted_objects: leased_objs,
            },
            _origin: PhantomData,
            _access: PhantomData,
            _driver_specific: Driver::default(),
        };

        f(lease);

        let mut args = ioctl::drm::RevokeLease { lessee_id };
        match unsafe { ioctl::drm::mode_revoke_lease(self.file.as_raw_fd(), &mut args) } {
            Ok(_) => (),
            Err(e) => panic!("revoke_lease: {e}"),
        }
    }
}
