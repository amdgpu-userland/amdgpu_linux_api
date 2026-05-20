use std::{cell::Cell, marker::PhantomData, mem::MaybeUninit, rc::Rc};

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

pub fn capget_or_panic() -> ThreadCapabilities {
    ThreadCapabilities::acquire().expect("ThreadCapabilities already acquired for this thread")
}

#[derive(Debug)]
pub enum CapsetError {
    InvalidArguments,
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

pub const fn has_all_caps(current_caps: CapSet, desired_caps: CapSet) -> bool {
    desired_caps == (desired_caps & current_caps)
}

/// Token "proving" the **current** thread has specified capabilities in effective set
pub struct ActiveCaps<const CAPS: CapSet>(std::marker::PhantomData<*mut ()>);

/// Token "proving" the **current** thread does not have specified capabilities in effective set
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

    pub fn effective(&self) -> CapSet {
        two_u32_to_u64_little_endian(self.data.effective_s1, self.data.effective_s0)
    }
    pub fn permitted(&self) -> CapSet {
        two_u32_to_u64_little_endian(self.data.permitted_s1, self.data.permitted_s0)
    }
    pub fn inheritable(&self) -> CapSet {
        two_u32_to_u64_little_endian(self.data.inheritable_s1, self.data.inheritable_s0)
    }

    pub fn has_all_effective(&self, caps: CapSet) -> bool {
        has_all_caps(self.effective(), caps)
    }

    pub fn has_all_permitted(&self, caps: CapSet) -> bool {
        has_all_caps(self.permitted(), caps)
    }

    pub fn has_all_inherited(&self, caps: CapSet) -> bool {
        has_all_caps(self.inheritable(), caps)
    }

    /// Executes a provided function with provided arguments with all
    /// specified capabilities in effective set.
    ///
    /// Lowers down capabilities it had to raise after the function.
    ///
    /// Provided function needs to be extra careful with further modyfing
    /// current thread's capability set as it may result in unexpected error
    /// during restoring previous capabilities.
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

    /// Executes a provided function with provided arguments with all
    /// specified capabilities removed from effective set.
    ///
    /// Restores capabilities it had to lower after the function.
    ///
    /// Provided function needs to be extra careful with further modyfing
    /// current thread's capability set as it may result in unexpected error
    /// during restoring previous capabilities.
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

    /// It's better to keep effective set clear and raise capabilities for critical sections
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

    /// Be careful once removed from permitted set they can no longer return without special
    /// circumstances.
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
