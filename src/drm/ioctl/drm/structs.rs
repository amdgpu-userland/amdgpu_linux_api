use std::os::fd::RawFd;

use crate::drm::{FlinkName, GemHandle, SyncobjHandle as SyncobjHandleId};

/// Id of a CRTC, connector or plane
pub type ObjectId = u32;

pub type LeaseId = u32;
pub type Magic = u32;

/// Legacy unique bus/device string buffer.
///
/// For GET_UNIQUE, `unique_len` is the caller-provided byte capacity of
/// `unique`. On return it is the kernel unique string length. The kernel only
/// copies bytes when the provided capacity is large enough.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Unique {
    pub unique_len: usize,
    pub unique: *mut u8,
}
assert_layout!(Unique, size = 16, align = 8);

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

/// Requested DRM interface and driver versions.
///
/// Set either major field to `-1` to leave that side unchanged. On return,
/// all fields are overwritten with the supported DRM interface and driver
/// versions.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SetVersion {
    pub drm_major: i32,
    pub drm_minor: i32,
    pub device_major: i32,
    pub device_minor: i32,
}
assert_layout!(SetVersion, size = 16, align = 4);

/// Request to enable or disable a DRM client capability for this file.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SetClientCap {
    pub capability: u64,
    pub value: u64,
}
assert_layout!(SetClientCap, size = 16, align = 8);

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

/// Move a GEM object to a different handle number.
///
/// `new_handle` must be unused. On success `handle` is closed, and future
/// ioctls must use `new_handle`.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct GemChangeHandle {
    pub handle: GemHandle,
    pub new_handle: GemHandle,
}
assert_layout!(GemChangeHandle, size = 8, align = 4);

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

/// Query the current scanout sequence number for a CRTC.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct CrtcGetSequence {
    pub crtc_id: ObjectId,
    /// Non-zero if the CRTC output is active.
    pub active: u32,
    /// Returned most recent vblank sequence.
    pub sequence: u64,
    /// Returned timestamp of the most recent first-pixel-out event, in ns.
    pub sequence_ns: i64,
}
assert_layout!(CrtcGetSequence, size = 24, align = 8);

pub mod crtc_sequence {
    pub type CrtcSequenceFlags = u32;
    pub const RELATIVE: CrtcSequenceFlags = 0x0000_0001;
    pub const NEXT_ON_MISS: CrtcSequenceFlags = 0x0000_0002;
}

/// Queue a CRTC sequence event.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct CrtcQueueSequence {
    pub crtc_id: ObjectId,
    pub flags: crtc_sequence::CrtcSequenceFlags,
    /// Input target sequence; output actual queued sequence.
    pub sequence: u64,
    /// User data copied into the delivered event.
    pub user_data: u64,
}
assert_layout!(CrtcQueueSequence, size = 24, align = 8);

pub mod syncobj_create {
    pub const SIGNALED: u32 = 1 << 0;
}

/// Create a DRM sync object.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjCreate {
    /// Returned syncobj handle.
    pub handle: SyncobjHandleId,
    pub flags: u32,
}
assert_layout!(SyncobjCreate, size = 8, align = 4);

/// Destroy a DRM sync object handle.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjDestroy {
    pub handle: SyncobjHandleId,
    pub pad: u32,
}
assert_layout!(SyncobjDestroy, size = 8, align = 4);

pub mod syncobj_handle_flags {
    pub const IMPORT_SYNC_FILE: u32 = 1 << 0;
    pub const EXPORT_SYNC_FILE: u32 = 1 << 0;
    pub const TIMELINE: u32 = 1 << 1;
}

/// Import or export a syncobj as an fd.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjHandle {
    pub handle: SyncobjHandleId,
    pub flags: u32,
    pub fd: RawFd,
    pub pad: u32,
    pub point: u64,
}
assert_layout!(SyncobjHandle, size = 24, align = 8);

/// Transfer a fence from one sync object point to another.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjTransfer {
    pub src_handle: SyncobjHandleId,
    pub dst_handle: SyncobjHandleId,
    pub src_point: u64,
    pub dst_point: u64,
    pub flags: u32,
    pub pad: u32,
}
assert_layout!(SyncobjTransfer, size = 32, align = 8);

pub mod syncobj_wait {
    pub const WAIT_ALL: u32 = 1 << 0;
    pub const WAIT_FOR_SUBMIT: u32 = 1 << 1;
    pub const WAIT_AVAILABLE: u32 = 1 << 2;
    pub const WAIT_DEADLINE: u32 = 1 << 3;
}

/// Wait for one or more binary sync objects.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjWait {
    /// Pointer to an array of syncobj handles.
    pub handles: *mut SyncobjHandleId,
    /// Absolute timeout in ns.
    pub timeout_nsec: i64,
    pub count_handles: u32,
    pub flags: u32,
    /// First signaled index; only valid without WAIT_ALL.
    pub first_signaled: u32,
    pub pad: u32,
    /// Absolute CLOCK_MONOTONIC deadline hint when WAIT_DEADLINE is set.
    pub deadline_nsec: u64,
}
assert_layout!(SyncobjWait, size = 40, align = 8);

/// Wait for one or more timeline sync object points.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjTimelineWait {
    /// Pointer to an array of syncobj handles.
    pub handles: *mut SyncobjHandleId,
    /// Pointer to an array of timeline points.
    pub points: *mut u64,
    /// Absolute timeout in ns.
    pub timeout_nsec: i64,
    pub count_handles: u32,
    pub flags: u32,
    /// First signaled index; only valid without WAIT_ALL.
    pub first_signaled: u32,
    pub pad: u32,
    /// Absolute CLOCK_MONOTONIC deadline hint when WAIT_DEADLINE is set.
    pub deadline_nsec: u64,
}
assert_layout!(SyncobjTimelineWait, size = 48, align = 8);

/// Register an eventfd to be signalled by a syncobj.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjEventfd {
    pub handle: SyncobjHandleId,
    pub flags: u32,
    pub point: u64,
    pub fd: RawFd,
    /// Must be zero.
    pub pad: u32,
}
assert_layout!(SyncobjEventfd, size = 24, align = 8);

/// Array of binary sync object handles.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjArray {
    /// Pointer to an array of syncobj handles.
    pub handles: *mut SyncobjHandleId,
    pub count_handles: u32,
    pub pad: u32,
}
assert_layout!(SyncobjArray, size = 16, align = 8);

pub mod syncobj_query {
    pub const LAST_SUBMITTED: u32 = 1 << 0;
}

/// Array of timeline sync object handles and points.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct SyncobjTimelineArray {
    /// Pointer to an array of syncobj handles.
    pub handles: *mut SyncobjHandleId,
    /// Pointer to an array of timeline points. For query this is output.
    pub points: *mut u64,
    pub count_handles: u32,
    pub flags: u32,
}
assert_layout!(SyncobjTimelineArray, size = 24, align = 8);

pub const DISPLAY_MODE_LEN: usize = 32;
pub const PROP_NAME_LEN: usize = 32;

/// Display mode information used by KMS mode ioctls.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeInfo {
    pub clock: u32,
    pub hdisplay: u16,
    pub hsync_start: u16,
    pub hsync_end: u16,
    pub htotal: u16,
    pub hskew: u16,
    pub vdisplay: u16,
    pub vsync_start: u16,
    pub vsync_end: u16,
    pub vtotal: u16,
    pub vscan: u16,
    pub vrefresh: u32,
    pub flags: u32,
    pub type_: u32,
    pub name: [u8; DISPLAY_MODE_LEN],
}
assert_layout!(ModeInfo, size = 68, align = 4);

/// KMS object ID lists and global min/max framebuffer dimensions.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCardRes {
    pub fb_id_ptr: *mut ObjectId,
    pub crtc_id_ptr: *mut ObjectId,
    pub connector_id_ptr: *mut ObjectId,
    pub encoder_id_ptr: *mut ObjectId,
    pub count_fbs: u32,
    pub count_crtcs: u32,
    pub count_connectors: u32,
    pub count_encoders: u32,
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
}
assert_layout!(ModeCardRes, size = 64, align = 8);

/// Get or set CRTC state.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCrtc {
    pub set_connectors_ptr: *mut ObjectId,
    pub count_connectors: u32,
    pub crtc_id: ObjectId,
    pub fb_id: ObjectId,
    pub x: u32,
    pub y: u32,
    pub gamma_size: u32,
    pub mode_valid: u32,
    pub mode: ModeInfo,
}
assert_layout!(ModeCrtc, size = 104, align = 8);

/// Legacy cursor update.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCursor {
    pub flags: u32,
    pub crtc_id: ObjectId,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub handle: GemHandle,
}
assert_layout!(ModeCursor, size = 28, align = 4);

/// CRTC gamma lookup table pointers.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCrtcLut {
    pub crtc_id: ObjectId,
    pub gamma_size: u32,
    pub red: *mut u16,
    pub green: *mut u16,
    pub blue: *mut u16,
}
assert_layout!(ModeCrtcLut, size = 32, align = 8);

/// KMS encoder metadata.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeGetEncoder {
    pub encoder_id: ObjectId,
    pub encoder_type: u32,
    pub crtc_id: ObjectId,
    pub possible_crtcs: u32,
    pub possible_clones: u32,
}
assert_layout!(ModeGetEncoder, size = 20, align = 4);

/// KMS connector metadata and array query.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeGetConnector {
    pub encoders_ptr: *mut ObjectId,
    pub modes_ptr: *mut ModeInfo,
    pub props_ptr: *mut ObjectId,
    pub prop_values_ptr: *mut u64,
    pub count_modes: u32,
    pub count_props: u32,
    pub count_encoders: u32,
    pub encoder_id: ObjectId,
    pub connector_id: ObjectId,
    pub connector_type: u32,
    pub connector_type_id: u32,
    pub connection: u32,
    pub mm_width: u32,
    pub mm_height: u32,
    pub subpixel: u32,
    /// Must be zero.
    pub pad: u32,
}
assert_layout!(ModeGetConnector, size = 80, align = 8);

/// One enum or bitmask entry for a property.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModePropertyEnum {
    pub value: u64,
    pub name: [u8; PROP_NAME_LEN],
}
assert_layout!(ModePropertyEnum, size = 40, align = 8);

/// KMS property metadata and array query.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeGetProperty {
    pub values_ptr: *mut u64,
    pub enum_blob_ptr: *mut ModePropertyEnum,
    pub prop_id: ObjectId,
    pub flags: u32,
    pub name: [u8; PROP_NAME_LEN],
    pub count_values: u32,
    pub count_enum_blobs: u32,
}
assert_layout!(ModeGetProperty, size = 64, align = 8);

/// Legacy connector property set request.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeConnectorSetProperty {
    pub value: u64,
    pub prop_id: ObjectId,
    pub connector_id: ObjectId,
}
assert_layout!(ModeConnectorSetProperty, size = 16, align = 8);

/// Get properties attached to any KMS object.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeObjGetProperties {
    pub props_ptr: *mut ObjectId,
    pub prop_values_ptr: *mut u64,
    pub count_props: u32,
    pub obj_id: ObjectId,
    pub obj_type: u32,
}
assert_layout!(ModeObjGetProperties, size = 32, align = 8);

/// Set a property on any KMS object.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeObjSetProperty {
    pub value: u64,
    pub prop_id: ObjectId,
    pub obj_id: ObjectId,
    pub obj_type: u32,
}
assert_layout!(ModeObjSetProperty, size = 24, align = 8);

/// Read a KMS property blob.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeGetBlob {
    pub blob_id: ObjectId,
    pub length: u32,
    pub data: *mut u8,
}
assert_layout!(ModeGetBlob, size = 16, align = 8);

/// Legacy single-plane framebuffer command.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeFbCmd {
    pub fb_id: ObjectId,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u32,
    pub depth: u32,
    pub handle: GemHandle,
}
assert_layout!(ModeFbCmd, size = 28, align = 4);

/// Framebuffer object ID argument used by RMFB.
pub type ModeRmfb = u32;

/// Legacy page flip request.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCrtcPageFlip {
    pub crtc_id: ObjectId,
    pub fb_id: ObjectId,
    pub flags: u32,
    /// Must be zero unless using page-flip target flags on newer kernels.
    pub reserved: u32,
    pub user_data: u64,
}
assert_layout!(ModeCrtcPageFlip, size = 24, align = 8);

/// Dirty framebuffer update region request.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeFbDirtyCmd {
    pub fb_id: ObjectId,
    pub flags: u32,
    pub color: u32,
    pub num_clips: u32,
    pub clips_ptr: *mut u8,
}
assert_layout!(ModeFbDirtyCmd, size = 24, align = 8);

/// Create a KMS dumb buffer object.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCreateDumb {
    pub height: u32,
    pub width: u32,
    pub bpp: u32,
    /// Must be zero.
    pub flags: u32,
    /// Returned GEM handle.
    pub handle: GemHandle,
    /// Returned pitch in bytes.
    pub pitch: u32,
    /// Returned allocation size in bytes.
    pub size: u64,
}
assert_layout!(ModeCreateDumb, size = 32, align = 8);

/// Get the mmap offset for a dumb buffer.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeMapDumb {
    pub handle: GemHandle,
    pub pad: u32,
    /// Returned fake mmap offset.
    pub offset: u64,
}
assert_layout!(ModeMapDumb, size = 16, align = 8);

/// Destroy a dumb buffer.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeDestroyDumb {
    pub handle: GemHandle,
}
assert_layout!(ModeDestroyDumb, size = 4, align = 4);

/// Plane object ID list query.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeGetPlaneRes {
    pub plane_id_ptr: *mut ObjectId,
    pub count_planes: u32,
}
assert_layout!(ModeGetPlaneRes, size = 16, align = 8);

/// KMS plane metadata and format list query.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeGetPlane {
    pub plane_id: ObjectId,
    pub crtc_id: ObjectId,
    pub fb_id: ObjectId,
    pub possible_crtcs: u32,
    pub gamma_size: u32,
    pub count_format_types: u32,
    pub format_type_ptr: *mut u32,
}
assert_layout!(ModeGetPlane, size = 32, align = 8);

/// Set a plane's framebuffer and source/destination rectangles.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeSetPlane {
    pub plane_id: ObjectId,
    pub crtc_id: ObjectId,
    pub fb_id: ObjectId,
    pub flags: u32,
    pub crtc_x: i32,
    pub crtc_y: i32,
    pub crtc_w: u32,
    pub crtc_h: u32,
    pub src_x: u32,
    pub src_y: u32,
    pub src_h: u32,
    pub src_w: u32,
}
assert_layout!(ModeSetPlane, size = 48, align = 4);

/// Multi-plane framebuffer command used by ADDFB2 and GETFB2.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeFbCmd2 {
    pub fb_id: ObjectId,
    pub width: u32,
    pub height: u32,
    pub pixel_format: u32,
    pub flags: u32,
    pub handles: [GemHandle; 4],
    pub pitches: [u32; 4],
    pub offsets: [u32; 4],
    pub modifier: [u64; 4],
}
assert_layout!(ModeFbCmd2, size = 104, align = 8);

/// Cursor update with hotspot coordinates.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCursor2 {
    pub flags: u32,
    pub crtc_id: ObjectId,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub handle: GemHandle,
    pub hot_x: i32,
    pub hot_y: i32,
}
assert_layout!(ModeCursor2, size = 36, align = 4);

/// Atomic KMS commit request.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeAtomic {
    pub flags: u32,
    pub count_objs: u32,
    pub objs_ptr: *mut ObjectId,
    pub count_props_ptr: *mut u32,
    pub props_ptr: *mut ObjectId,
    pub prop_values_ptr: *mut u64,
    /// Must be zero.
    pub reserved: u64,
    pub user_data: u64,
}
assert_layout!(ModeAtomic, size = 56, align = 8);

/// Create a user property blob.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCreateBlob {
    pub data: *const u8,
    pub length: u32,
    /// Returned blob object ID.
    pub blob_id: ObjectId,
}
assert_layout!(ModeCreateBlob, size = 16, align = 8);

/// Destroy a user property blob.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeDestroyBlob {
    pub blob_id: ObjectId,
}
assert_layout!(ModeDestroyBlob, size = 4, align = 4);

/// Close a framebuffer object without disabling active planes.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeCloseFb {
    pub fb_id: ObjectId,
    pub pad: u32,
}
assert_layout!(ModeCloseFb, size = 8, align = 4);

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
