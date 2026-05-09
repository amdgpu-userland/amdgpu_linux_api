use std::os::fd::{OwnedFd, RawFd};

use crate::drm::GemHandle;

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
