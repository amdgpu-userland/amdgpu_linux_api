use std::os::fd::RawFd;

use crate::drm::{FlinkName, GemHandle};

/// Id of a CRTC, connector or plane
pub type ObjectId = u32;

pub type LeaseId = u32;
pub type Magic = u32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patchlevel: i32,
    pub name_len: usize,
    pub name: *mut u8,
    pub date_len: usize,
    pub date: *mut u8,
    pub desc_len: usize,
    pub desc: *mut u8,
}
assert_layout!(Version, size = 64, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Auth {
    pub magic: Magic,
}
assert_layout!(Auth, size = 4, align = 4);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Client {
    /// Set this to 0
    pub idx: i32,
    /// Is authenticated
    pub auth: i32,
    pub pid: u64,
    pub uid: u64,
    pub magic: u64,
    pub iocs: u64,
}
assert_layout!(Client, size = 40, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GemClose {
    pub handle: GemHandle,
    pub pad: u32,
}
assert_layout!(GemClose, size = 8, align = 4);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GemFlink {
    pub handle: GemHandle,
    pub name: FlinkName,
}
assert_layout!(GemFlink, size = 8, align = 4);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GemOpen {
    pub name: FlinkName,
    pub handle: GemHandle,
    pub size: usize,
}
assert_layout!(GemOpen, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GetCap {
    pub capability: u64,
    pub value: u64,
}
assert_layout!(GetCap, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct PrimeHandle {
    pub handle: GemHandle,
    /// open() flags for dmabuf fd
    pub flags: u32,
    /// Returned dmabuf file descriptor
    pub fd: RawFd,
}
assert_layout!(PrimeHandle, size = 12, align = 4);

pub mod vblank {
    pub type VblankSeqType = u32;

    pub const ABSOLUTE: VblankSeqType = 0x0;
    pub const RELATIVE: VblankSeqType = 0x1;
    pub const HIGH_CRTC_MASK: VblankSeqType = 0x0000_003e;
    pub const EVENT: VblankSeqType = 0x0400_0000;
    pub const FLIP: VblankSeqType = 0x0800_0000;
    pub const NEXTONMISS: VblankSeqType = 0x1000_0000;
    pub const SECONDARY: VblankSeqType = 0x2000_0000;
    pub const SIGNAL: VblankSeqType = 0x4000_0000;

    pub const HIGH_CRTC_SHIFT: VblankSeqType = 1;
    pub const TYPES_MASK: VblankSeqType = ABSOLUTE | RELATIVE;
    pub const FLAGS_MASK: VblankSeqType = EVENT | SIGNAL | SECONDARY | NEXTONMISS;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WaitVblankRequest {
    pub type_: vblank::VblankSeqType,
    pub sequence: u32,
    pub signal: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct WaitVblankReply {
    pub type_: vblank::VblankSeqType,
    pub sequence: u32,
    pub tval_sec: u64,
    pub tval_usec: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union WaitVblank {
    pub request: WaitVblankRequest,
    pub reply: WaitVblankReply,
}
assert_layout!(WaitVblank, size = 24, align = 8);

/// struct drm_mode_create_lease - Create lease
///
/// Lease mode resources, creating another drm_master.
///
/// The @object_ids array must reference at least one CRTC, one connector and
/// one plane if &DRM_CLIENT_CAP_UNIVERSAL_PLANES is enabled. Alternatively,
/// the lease can be completely empty.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct CreateLease {
    /** @object_ids: Pointer to array of object ids (__u32) */
    pub object_ids: *const ObjectId,
    /** @object_count: Number of object ids */
    pub object_count: u32,
    /** @flags: flags for new FD (O_CLOEXEC, etc) */
    pub flags: i32,

    /** @lessee_id: Return: unique identifier for lessee. */
    pub lessee_id: LeaseId,
    /** @fd: Return: file descriptor to new drm_master file */
    pub fd: RawFd,
}
assert_layout!(CreateLease, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ListLessees {
    /**
     * @count_lessees: Number of lessees.
     *
     * On input, provides length of the array.
     * On output, provides total number. No
     * more than the input number will be written
     * back, so two calls can be used to get
     * the size and then the data.
     */
    pub count_lessees: u32,
    /** @pad: Padding. */
    pub pad: u32,

    /**
     * @lessees_ptr: Pointer to lessees.
     *
     * Pointer to __u64 array of lessee ids
     */
    pub lessees_ptr: *mut LeaseId,
}
assert_layout!(ListLessees, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GetLease {
    /**
     * @count_objects: Number of leased objects.
     *
     * On input, provides length of the array.
     * On output, provides total number. No
     * more than the input number will be written
     * back, so two calls can be used to get
     * the size and then the data.
     */
    pub count_objects: u32,
    /** @pad: Padding. */
    pub pad: u32,

    /**
     * @objects_ptr: Pointer to objects.
     *
     * Pointer to __u32 array of object ids.
     */
    pub objects_ptr: *mut ObjectId,
}
assert_layout!(GetLease, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct RevokeLease {
    /** @lessee_id: Unique ID of lessee */
    pub lessee_id: LeaseId,
}
assert_layout!(RevokeLease, size = 4, align = 4);

pub const CLIENT_NAME_MAX_LEN: usize = 64;
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SetClientName {
    pub name_len: usize,
    pub name: *const u8,
}
assert_layout!(SetClientName, size = 16, align = 8);
