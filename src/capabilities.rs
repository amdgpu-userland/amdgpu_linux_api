//! Thread-local Linux capability management.
//!
//! Linux capabilities split privileged operations into individual bits such as
//! [`CAP_SYS_ADMIN`]. Each thread has effective, permitted, and inheritable
//! capability sets:
//!
//! - **effective** capabilities are active right now and are checked by the
//!   kernel for privileged operations;
//! - **permitted** capabilities are available to be raised into the effective
//!   set;
//! - **inheritable** capabilities control what can survive `execve`.
//!
//! Linux also has **ambient** and **bounding** capability sets. This module
//! intentionally leaves them out and focuses only on the capability state
//! exposed through `capget(2)` and `capset(2)`: effective, permitted, and
//! inheritable capabilities.
//!
//! This module gives you a scoped way to temporarily change the current
//! thread's effective capability set. It is designed around proof tokens:
//! [`ActiveCaps`] proves that a capability is effective inside a closure, and
//! [`DisabledCaps`] proves that it is not effective inside a closure. Other
//! APIs in this crate can require those tokens instead of silently depending on
//! ambient privileges.
//!
//! # Basic flow
//!
//! Acquire a [`ThreadCapabilities`] handle for the current thread, optionally
//! clear the effective set, and then raise only the capabilities needed for a
//! critical section.
//!
//! ```no_run
//! use amdgpu_linux_api::capabilities::{
//!     ActiveCaps, CAP_SYS_ADMIN, ThreadCapabilities,
//! };
//!
//! fn operation_requiring_sys_admin(_token: &ActiveCaps<CAP_SYS_ADMIN>) {
//!     // Call an API that requires CAP_SYS_ADMIN here.
//! }
//!
//! # fn main() -> Result<(), amdgpu_linux_api::capabilities::CapsetError> {
//! let mut caps = ThreadCapabilities::acquire()
//!     .expect("capabilities handle already acquired on this thread");
//!
//! // Keep ambient privilege low. CAP_SYS_ADMIN can still be raised later only
//! // if it remains in the permitted set.
//! caps.clear_effective();
//!
//! caps.with_effective::<CAP_SYS_ADMIN, _, _, _>(
//!     |sys_admin, ()| operation_requiring_sys_admin(sys_admin),
//!     (),
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! `with_effective` only raises capabilities that are already in the permitted
//! set. If the capability is missing from the permitted set, it returns
//! [`CapsetError::PermissionDenied`].
//!
//! # Proving a capability is disabled
//!
//! Some DRM operations behave differently for privileged and unprivileged
//! callers. Use [`ThreadCapabilities::without_effective`] when you need to run a
//! section with a capability temporarily removed.
//!
//! ```no_run
//! use amdgpu_linux_api::capabilities::{
//!     CAP_SYS_ADMIN, DisabledCaps, ThreadCapabilities,
//! };
//!
//! fn rootless_path(_token: &DisabledCaps<CAP_SYS_ADMIN>) {
//!     // Call an API that must observe the thread without CAP_SYS_ADMIN.
//! }
//!
//! # fn main() -> Result<(), amdgpu_linux_api::capabilities::CapsetError> {
//! let mut caps = ThreadCapabilities::acquire()
//!     .expect("capabilities handle already acquired on this thread");
//!
//! caps.without_effective::<CAP_SYS_ADMIN, _, _, _>(
//!     |no_sys_admin, ()| rootless_path(no_sys_admin),
//!     (),
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! # Multiple capabilities and subsets
//!
//! Capability constants are bitmasks and can be ORed together. A wider token can
//! be reborrowed as a narrower token with [`ActiveCaps::subset`] or
//! [`DisabledCaps::subset`].
//!
//! ```no_run
//! use amdgpu_linux_api::capabilities::{
//!     ActiveCaps, CAP_NET_ADMIN, CAP_SYS_ADMIN, CapSet,
//! };
//!
//! const ADMIN_CAPS: CapSet = CAP_SYS_ADMIN | CAP_NET_ADMIN;
//!
//! fn needs_sys_admin(_token: &ActiveCaps<CAP_SYS_ADMIN>) {}
//!
//! fn use_wide_token(token: &ActiveCaps<ADMIN_CAPS>) {
//!     needs_sys_admin(token.subset::<CAP_SYS_ADMIN>());
//! }
//! ```
//!
//! # Ownership and concurrency
//!
//! Only one [`ThreadCapabilities`] handle can be acquired per thread at a time.
//! This prevents two pieces of code from restoring stale capability snapshots
//! over each other. Drop the handle before acquiring another one on the same
//! thread.
//!
//! The handle and proof tokens are intentionally thread-local. Do not use a
//! token as proof for another thread's capability state.
//!
//! # External capability changes
//!
//! Linux permits capability state to be changed outside this handle's control,
//! for example by another thread in the same process or by a sufficiently
//! privileged external process. [`ThreadCapabilities`] keeps a snapshot taken
//! at acquisition time and updates that snapshot only after changes made
//! through this module.
//!
//! Scoped methods such as [`ThreadCapabilities::with_effective`] and
//! [`ThreadCapabilities::without_effective`] assume that nothing else mutates
//! the current thread's capability sets while the closure is running. If another
//! actor changes permitted capabilities behind this module's back, restoration
//! can fail and the method may panic rather than silently continue with a stale
//! model of the thread's privileges.
//!
//! # Irreversible operations
//!
//! Clearing the effective set is reversible as long as the capability remains in
//! the permitted set. Clearing the permitted set with
//! [`ThreadCapabilities::clear_permitted`] is normally irreversible for the
//! current process, so do that only after all privileged work is complete.

use std::{cell::Cell, marker::PhantomData, mem::MaybeUninit, rc::Rc};

/// Bitmask of Linux capabilities.
///
/// Each `CAP_*` constant in this module occupies one bit. Combine them with
/// bitwise OR when an operation needs more than one capability.
pub type CapSet = u64;

#[derive(Debug)]
#[repr(C)]
struct CapUserHeader {
    // Linux capabilities version (runtime kernel support)
    version: u32,
    // Process ID (thread)
    pid: i32,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct CapUserData {
    effective_s0: u32,
    permitted_s0: u32,
    inheritable_s0: u32,
    effective_s1: u32,
    permitted_s1: u32,
    inheritable_s1: u32,
}

/// Current thread capability snapshot and scoped mutation handle.
///
/// The handle is acquired from the kernel with `capget(2)` and keeps an
/// internal copy of the current thread's capability sets. Mutating methods call
/// `capset(2)` and update that copy on success.
///
/// Only one handle may be active per thread. This avoids stale restore problems
/// when nested or unrelated code tries to change the same thread's capability
/// state.
#[derive(Debug)]
pub struct ThreadCapabilities {
    data: CapUserData,
    _thread_local: PhantomData<Rc<()>>,
}

#[allow(clippy::unreadable_literal)]
const CAPS_V3: u32 = 0x20080522;

thread_local! {
    static THREAD_CAPABILITIES_ACQUIRED: Cell<bool> = const { Cell::new(false) };
}

/// Error returned when a [`ThreadCapabilities`] handle is already active for
/// the current thread.
#[derive(Debug)]
pub struct ThreadCapabilitiesAlreadyAcquired;

fn raw_capget() -> CapUserData {
    let hdr = CapUserHeader {
        version: CAPS_V3,
        pid: 0,
    };
    let mut data: MaybeUninit<CapUserData> = MaybeUninit::uninit();
    let r = unsafe { libc::syscall(libc::SYS_capget, &raw const hdr, &raw mut data) };
    match r {
        0 => unsafe { data.assume_init() },
        _ => panic!("capget: {}", unsafe { *libc::__errno_location() }),
    }
}

/// Acquires a [`ThreadCapabilities`] handle or panics if this thread already has
/// one.
///
/// Prefer [`ThreadCapabilities::acquire`] in library code so callers can decide
/// how to handle acquisition conflicts.
pub fn capget_or_panic() -> ThreadCapabilities {
    ThreadCapabilities::acquire().expect("ThreadCapabilities already acquired for this thread")
}

/// Error returned by operations that modify the current thread's capability
/// sets.
#[derive(Debug)]
pub enum CapsetError {
    /// The kernel rejected the supplied capability data.
    InvalidArguments,
    /// A requested capability cannot be raised because it is not in the
    /// permitted set, or the kernel rejected the change for permissions reasons.
    PermissionDenied,
}

fn raw_capset(data: &CapUserData) -> Result<(), CapsetError> {
    let hdr = CapUserHeader {
        version: CAPS_V3,
        pid: 0,
    };
    let r = unsafe { libc::syscall(libc::SYS_capset, hdr, data) };
    if r == 0 {
        return Ok(());
    }
    match unsafe { *libc::__errno_location() } {
        libc::EINVAL => Err(CapsetError::InvalidArguments),
        libc::EPERM => Err(CapsetError::PermissionDenied),
        e => panic!("unexpected error from capset: {e}"),
    }
}

impl Drop for ThreadCapabilities {
    fn drop(&mut self) {
        THREAD_CAPABILITIES_ACQUIRED.set(false);
    }
}

fn two_u32_to_u64_little_endian(hi: u32, lo: u32) -> u64 {
    u64::from(hi) << 32 | u64::from(lo)
}

/// Returns true when `current_caps` contains every bit from `desired_caps`.
pub const fn has_all_caps(current_caps: CapSet, desired_caps: CapSet) -> bool {
    desired_caps == (desired_caps & current_caps)
}

/// Token proving the **current** thread has `CAPS` in its effective set.
///
/// Values of this type are created by [`ThreadCapabilities::with_effective`].
/// APIs can accept `&ActiveCaps<CAP_SYS_ADMIN>` or another const capability
/// mask to make their privilege requirement explicit.
pub struct ActiveCaps<const CAPS: CapSet>(std::marker::PhantomData<*mut ()>);

/// Token proving the **current** thread does not have `CAPS` in its effective
/// set.
///
/// Values of this type are created by [`ThreadCapabilities::without_effective`].
/// This is useful for paths that need to observe unprivileged kernel behavior.
pub struct DisabledCaps<const CAPS: CapSet>(std::marker::PhantomData<*mut ()>);

impl<const CAPS: CapSet> ActiveCaps<CAPS> {
    /// Reborrows this token as proof for a subset of the active capabilities.
    ///
    /// This lets callers obtain several narrower capability tokens from one
    /// wider token without issuing another `capget`/`capset` sequence.
    ///
    /// # Panics
    ///
    /// Panics when `SUBSET` contains a capability not present in `CAPS`.
    pub const fn subset<const SUBSET: CapSet>(&self) -> &ActiveCaps<SUBSET> {
        assert!(has_all_caps(CAPS, SUBSET));

        // ActiveCaps is a zero-sized proof token. Reborrow with the same
        // lifetime while changing only the const parameter after proving that
        // the requested token is narrower than the original one.
        unsafe { &*(self as *const Self).cast::<ActiveCaps<SUBSET>>() }
    }
}

impl<const CAPS: CapSet> DisabledCaps<CAPS> {
    /// Reborrows this token as proof for a subset of the disabled capabilities.
    ///
    /// This lets callers obtain several narrower capability tokens from one
    /// wider token without issuing another `capget`/`capset` sequence.
    ///
    /// # Panics
    ///
    /// Panics when `SUBSET` contains a capability not present in `CAPS`.
    pub const fn subset<const SUBSET: CapSet>(&self) -> &DisabledCaps<SUBSET> {
        assert!(has_all_caps(CAPS, SUBSET));

        // DisabledCaps is a zero-sized proof token. Reborrow with the same
        // lifetime while changing only the const parameter after proving that
        // the requested token is narrower than the original one.
        unsafe { &*(self as *const Self).cast::<DisabledCaps<SUBSET>>() }
    }
}

impl ThreadCapabilities {
    /// Acquires a capability handle for the current thread.
    ///
    /// Returns [`ThreadCapabilitiesAlreadyAcquired`] if a handle is already
    /// alive on this thread. Drop the existing handle before acquiring another.
    pub fn acquire() -> Result<Self, ThreadCapabilitiesAlreadyAcquired> {
        THREAD_CAPABILITIES_ACQUIRED.with(|acquired| {
            if acquired.get() {
                return Err(ThreadCapabilitiesAlreadyAcquired);
            }

            let data = raw_capget();
            acquired.set(true);
            Ok(Self {
                data,
                _thread_local: PhantomData,
            })
        })
    }

    /// Returns the current snapshot of the effective capability set.
    ///
    /// The value is updated by this handle's mutating methods. It will be stale
    /// if other code changes the current thread's capabilities behind this
    /// handle.
    pub fn effective(&self) -> CapSet {
        two_u32_to_u64_little_endian(self.data.effective_s1, self.data.effective_s0)
    }

    /// Returns the current snapshot of the permitted capability set.
    pub fn permitted(&self) -> CapSet {
        two_u32_to_u64_little_endian(self.data.permitted_s1, self.data.permitted_s0)
    }

    /// Returns the current snapshot of the inheritable capability set.
    pub fn inheritable(&self) -> CapSet {
        two_u32_to_u64_little_endian(self.data.inheritable_s1, self.data.inheritable_s0)
    }

    /// Returns true when every bit in `caps` is present in the effective set.
    pub fn has_all_effective(&self, caps: CapSet) -> bool {
        has_all_caps(self.effective(), caps)
    }

    /// Returns true when every bit in `caps` is present in the permitted set.
    pub fn has_all_permitted(&self, caps: CapSet) -> bool {
        has_all_caps(self.permitted(), caps)
    }

    /// Returns true when every bit in `caps` is present in the inheritable set.
    pub fn has_all_inherited(&self, caps: CapSet) -> bool {
        has_all_caps(self.inheritable(), caps)
    }

    /// Executes `f` with all `CAPS` present in the effective set.
    ///
    /// Capabilities that were already effective are left unchanged. Missing
    /// effective capabilities are raised from the permitted set before `f` is
    /// called, then lowered again afterward.
    ///
    /// The closure receives an [`ActiveCaps<CAPS>`] proof token tied to the
    /// call. Pass that token to APIs that require the capability.
    ///
    /// # Errors
    ///
    /// Returns [`CapsetError::PermissionDenied`] when any requested capability
    /// is not in the permitted set.
    ///
    /// # Panics
    ///
    /// Panics if the closure, or other code running on the same thread, mutates
    /// the permitted set in a way that prevents this method from restoring the
    /// previous effective set.
    pub fn with_effective<const CAPS: CapSet, Args, Func, Ret>(
        &mut self,
        f: Func,
        args: Args,
    ) -> Result<Ret, CapsetError>
    where
        Func: FnOnce(&ActiveCaps<CAPS>, Args) -> Ret,
    {
        let current_ef: u64 = self.effective();
        let current_pm: u64 = self.permitted();
        let missing_ef = !current_ef & CAPS;
        let missing_pm = !current_pm & missing_ef;
        if missing_ef != 0 {
            if missing_pm != 0 {
                return Err(CapsetError::PermissionDenied);
            }
            let mut raised = self.data;
            raised.effective_s0 |= missing_ef as u32;
            raised.effective_s1 |= (missing_ef >> 32) as u32;
            match raw_capset(&raised) {
                Ok(_) => self.data = raised,
                Err(CapsetError::PermissionDenied) => {
                    panic!(
                        "Somebody modified current thread's permitted capabilities behind my back"
                    )
                }
                Err(e) => return Err(e),
            }
        }
        let token = ActiveCaps(std::marker::PhantomData);
        let res = f(&token, args);
        if missing_ef != 0 {
            let mut restored = self.data;
            restored.effective_s0 &= !missing_ef as u32;
            restored.effective_s1 &= (!missing_ef >> 32) as u32;
            raw_capset(&restored).expect("Provided function or somebody else is supposed not to modify current thread's permitted capabilites which might make this operation not valid");
            self.data = restored;
        }
        Ok(res)
    }

    /// Executes `f` with all `CAPS` removed from the effective set.
    ///
    /// Capabilities that are not currently effective are left unchanged.
    /// Capabilities that are currently effective are lowered before `f` is
    /// called, then restored afterward.
    ///
    /// The closure receives a [`DisabledCaps<CAPS>`] proof token tied to the
    /// call. Pass that token to APIs that require the capability to be absent.
    ///
    /// # Panics
    ///
    /// Panics if the closure, or other code running on the same thread, mutates
    /// the permitted set in a way that prevents this method from restoring the
    /// previous effective set.
    pub fn without_effective<const CAPS: CapSet, Args, Func, Ret>(
        &mut self,
        f: Func,
        args: Args,
    ) -> Result<Ret, CapsetError>
    where
        Func: FnOnce(&DisabledCaps<CAPS>, Args) -> Ret,
    {
        let current_ef: u64 = self.effective();
        let present_ef = current_ef & CAPS;
        if present_ef != 0 {
            let mut lowered = self.data;
            lowered.effective_s0 &= !(present_ef as u32);
            lowered.effective_s1 &= !((present_ef >> 32) as u32);
            raw_capset(&lowered).expect(Self::LOWERING_CAPS_EXPECT);
            self.data = lowered;
        }
        let token = DisabledCaps(std::marker::PhantomData);
        let res = f(&token, args);
        if present_ef != 0 {
            let mut restored = self.data;
            restored.effective_s0 |= present_ef as u32;
            restored.effective_s1 |= (present_ef >> 32) as u32;
            raw_capset(&restored).expect("Provided function or somebody else is supposed not to modify current thread's permitted capabilites which might make this operation not valid");
            self.data = restored;
        }
        Ok(res)
    }

    const LOWERING_CAPS_EXPECT: &str = "Lowering your own capabilities should always work";

    /// Clears all effective capabilities for the current thread.
    ///
    /// This lowers ambient privilege while keeping permitted capabilities
    /// available for later [`with_effective`](Self::with_effective) calls.
    pub fn clear_effective(&mut self) {
        if self.data.effective_s1 == 0 && self.data.effective_s0 == 0 {
            return;
        }
        let mut cleared = self.data;
        cleared.effective_s0 = 0;
        cleared.effective_s1 = 0;
        raw_capset(&cleared).expect(Self::LOWERING_CAPS_EXPECT);
        self.data = cleared;
    }

    /// Clears all permitted capabilities for the current thread.
    ///
    /// Be careful: once a capability is removed from the permitted set, it
    /// normally cannot be raised again in the current process.
    pub fn clear_permitted(&mut self) {
        if self.data.permitted_s1 == 0 && self.data.permitted_s0 == 0 {
            return;
        }
        let mut cleared = self.data;
        cleared.permitted_s1 = 0;
        cleared.permitted_s0 = 0;
        raw_capset(&cleared).expect(Self::LOWERING_CAPS_EXPECT);
        self.data = cleared;
    }

    /// Clears all inheritable capabilities for the current thread.
    pub fn clear_inheritable(&mut self) {
        if self.data.inheritable_s1 == 0 && self.data.inheritable_s0 == 0 {
            return;
        }
        let mut cleared = self.data;
        cleared.inheritable_s1 = 0;
        cleared.inheritable_s0 = 0;
        raw_capset(&cleared).expect(Self::LOWERING_CAPS_EXPECT);
        self.data = cleared;
    }
}

macro_rules! define_cap {
    ($name:ident, $index:expr, $doc:expr) => {
        #[doc = $doc]
        pub const $name: CapSet = 1 << $index;
    };
}

define_cap!(
    CAP_CHOWN,
    0,
    "Make arbitrary changes to file UIDs and GIDs."
);
define_cap!(
    CAP_DAC_OVERRIDE,
    1,
    "Bypass file read, write, and execute permission checks."
);
define_cap!(
    CAP_DAC_READ_SEARCH,
    2,
    "Bypass file read permission checks and directory read/execute checks."
);
define_cap!(
    CAP_FOWNER,
    3,
    "Bypass permission checks on operations that normally require the file UID to match."
);
define_cap!(
    CAP_FSETID,
    4,
    "Don't clear set-user-ID and set-group-ID permission bits when a file is modified."
);
define_cap!(CAP_KILL, 5, "Bypass permission checks for sending signals.");
define_cap!(
    CAP_SETGID,
    6,
    "Make arbitrary manipulations of process GIDs and supplementary GID list."
);
define_cap!(
    CAP_SETUID,
    7,
    "Make arbitrary manipulations of process UIDs."
);
define_cap!(CAP_SETPCAP, 8, "Modify process capabilities.");
define_cap!(
    CAP_LINUX_IMMUTABLE,
    9,
    "Modify file attributes on ext2/ext3/ext4 filesystems."
);
define_cap!(
    CAP_NET_BIND_SERVICE,
    10,
    "Bind a socket to Internet domain privileged ports (port numbers < 1024)."
);
define_cap!(
    CAP_NET_BROADCAST,
    11,
    "Make network broadcasting and listen to multicasts."
);
define_cap!(
    CAP_NET_ADMIN,
    12,
    "Perform various network-related operations."
);
define_cap!(CAP_NET_RAW, 13, "Use RAW and PACKET sockets.");
define_cap!(
    CAP_IPC_LOCK,
    14,
    "Lock memory (mlock, mlockall, mmap, shmctl)."
);
define_cap!(
    CAP_IPC_OWNER,
    15,
    "Bypass permission checks for operations on System V IPC objects."
);
define_cap!(CAP_SYS_MODULE, 16, "Load and unload kernel modules.");
define_cap!(
    CAP_SYS_RAWIO,
    17,
    "Perform I/O port operations (iopl and ioperm)."
);
define_cap!(CAP_SYS_CHROOT, 18, "Use chroot.");
define_cap!(
    CAP_SYS_PTRACE,
    19,
    "Trace arbitrary processes using ptrace."
);
define_cap!(CAP_SYS_PACCT, 20, "Configure process accounting.");
define_cap!(
    CAP_SYS_ADMIN,
    21,
    "Perform a range of system administration operations."
);
define_cap!(CAP_SYS_BOOT, 22, "Use reboot and kexec_load.");
define_cap!(
    CAP_SYS_NICE,
    23,
    "Raise process nice value and change scheduling priority."
);
define_cap!(
    CAP_SYS_RESOURCE,
    24,
    "Override various system resource limits."
);
define_cap!(CAP_SYS_TIME, 25, "Set system clock and real-time clock.");
define_cap!(CAP_SYS_TTY_CONFIG, 26, "Configure TTY devices.");
define_cap!(CAP_MKNOD, 27, "Create special files using mknod.");
define_cap!(CAP_LEASE, 28, "Establish leases on arbitrary files.");
define_cap!(CAP_AUDIT_WRITE, 29, "Write records to kernel auditing log.");
define_cap!(CAP_AUDIT_CONTROL, 30, "Enable and disable kernel auditing.");
define_cap!(CAP_SETFCAP, 31, "Set file capabilities.");
define_cap!(
    CAP_MAC_OVERRIDE,
    32,
    "Allow MAC configuration or state changes."
);
define_cap!(
    CAP_MAC_ADMIN,
    33,
    "Override Mandatory Access Control (MAC)."
);
define_cap!(CAP_SYSLOG, 34, "Perform privileged syslog operations.");
define_cap!(
    CAP_WAKE_ALARM,
    35,
    "Trigger something that will wake up the system."
);
define_cap!(
    CAP_BLOCK_SUSPEND,
    36,
    "Employ features that can block system suspend."
);
define_cap!(
    CAP_AUDIT_READ,
    37,
    "Allow reading the audit log via multicast netlink socket."
);
define_cap!(CAP_PERFMON, 38, "Allow system performance monitoring.");
define_cap!(CAP_BPF, 39, "Allow extended BPF operations.");
define_cap!(
    CAP_CHECKPOINT_RESTORE,
    40,
    "Allow checkpoint/restore operations."
);

#[cfg(test)]
mod tests {
    use super::*;

    const TWO_CAPS: CapSet = CAP_SYS_ADMIN | CAP_NET_ADMIN;

    fn needs_active_sys_admin(_: &ActiveCaps<CAP_SYS_ADMIN>) {}
    fn needs_disabled_sys_admin(_: &DisabledCaps<CAP_SYS_ADMIN>) {}

    #[test]
    fn active_caps_can_be_reborrowed_as_subset() {
        let caps = ActiveCaps::<TWO_CAPS>(PhantomData);

        needs_active_sys_admin(caps.subset::<CAP_SYS_ADMIN>());
    }

    #[test]
    fn disabled_caps_can_be_reborrowed_as_subset() {
        let caps = DisabledCaps::<TWO_CAPS>(PhantomData);

        needs_disabled_sys_admin(caps.subset::<CAP_SYS_ADMIN>());
    }

    #[test]
    #[should_panic]
    fn active_caps_reject_non_subset() {
        let caps = ActiveCaps::<CAP_SYS_ADMIN>(PhantomData);

        let _ = caps.subset::<TWO_CAPS>();
    }

    #[test]
    #[should_panic]
    fn disabled_caps_reject_non_subset() {
        let caps = DisabledCaps::<CAP_SYS_ADMIN>(PhantomData);

        let _ = caps.subset::<TWO_CAPS>();
    }
}
