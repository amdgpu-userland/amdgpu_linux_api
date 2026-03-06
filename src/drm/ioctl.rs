use std::ffi::c_void;

use crate::drm::GemHandle;

const DRM_IOCTL_BASE: u32 = 'd' as u32;
const DRM_COMMAND_BASE: u32 = 0x40;

macro_rules! define_drm_ioctl {
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, $ioctl_direction:tt) => {
        define_ioctl!(
            $(#[$meta])*
            $fn_name,
            $args_ty,
            $num,
            DRM_IOCTL_BASE,
            $ioctl_direction
        );
    };
    ($(#[$meta:meta])* $fn_name:ident, $ioctl_num:expr) => {
        define_ioctl!(
            $(#[$meta])*
            $fn_name,
            $ioctl_num,
            DRM_IOCTL_BASE
        );
    };
}
macro_rules! define_amddrm_ioctl {
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, $ioctl_direction:tt) => {
        define_ioctl!(
            $(#[$meta])*
            $fn_name,
            $args_ty,
            DRM_COMMAND_BASE + $num,
            DRM_IOCTL_BASE,
            $ioctl_direction
        );
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmVersion {
    /// Major version
    pub major: i32,
    /// Minor version
    pub minor: i32,
    /// Patch level
    pub patchlevel: i32,
    /// Length of name buffer
    pub name_len: usize,
    /// Name of driver
    pub name: *mut u8,
    /// Length of date buffer
    pub date_len: usize,
    /// User-space buffer to hold date
    pub date: *mut u8,
    /// Length of desc buffer
    pub desc_len: usize,
    /// User-space buffer to hold desc
    pub desc: *mut u8,
}
assert_layout!(DrmVersion, size = 64, align = 8);

define_drm_ioctl!(drm_ioctl_version, DrmVersion, 0x0, WR);

define_drm_ioctl!(drm_ioctl_set_master, 0x1e);
define_drm_ioctl!(drm_ioctl_drop_master, 0x1f);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct DrmClient {
    /// Set this to 0
    pub idx: i32,
    /// Is authenticated
    pub auth: i32,
    pub pid: u64,
    pub uid: u64,
    pub magic: u64,
    pub iocs: u64,
}
assert_layout!(DrmClient, size = 40, align = 8);
define_drm_ioctl!(
    /// Almost deprecated
    ///
    /// if idx==0 it will populate some fields
    /// which you can use to easily determine if this client is authenticated
    /// EINVAL otherwise
    drm_ioctl_get_client, DrmClient, 0x05, WR);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmAmdgpuGemCreateIn {
    /** the requested memory size */
    pub bo_size: usize,
    /** physical start_addr alignment in bytes for some HW requirements */
    pub alignment: usize,
    /** the requested memory domains */
    pub domains: u64,
    /** allocation flags */
    pub domain_flags: u64,
}

pub mod gem_domain {
    pub const CPU: u64 = 1 << 0;
    pub const GTT: u64 = 1 << 1;
    pub const VRAM: u64 = 1 << 2;
    pub const GDS: u64 = 1 << 3;
    pub const GWS: u64 = 1 << 4;
    pub const OA: u64 = 1 << 5;
    pub const DOORBELL: u64 = 1 << 6;
    pub const MMIO_REMAP: u64 = 1 << 7;
}

pub mod gem_flags {
    /// Flag that CPU access will be required for the case of VRAM domain
    pub const CPU_ACCESS_REQUIRED: u64 = 1 << 0;
    /// Flag that CPU access will not work, this VRAM domain is invisible
    pub const NO_CPU_ACCESS: u64 = 1 << 1;
    /// Flag that USWC attributes should be used for GTT
    pub const CPU_GTT_USWC: u64 = 1 << 2;
    /// Flag that the memory should be in VRAM and cleared
    pub const VRAM_CLEARED: u64 = 1 << 3;
    /// Flag that allocating the BO should use linear VRAM
    pub const VRAM_CONTIGUOUS: u64 = 1 << 5;
    /// Flag that BO is always valid in this VM
    pub const VM_ALWAYS_VALID: u64 = 1 << 6;
    /// Flag that BO sharing will be explicitly synchronized
    pub const EXPLICIT_SYNC: u64 = 1 << 7;
    /// Flag that indicates allocating MQD gart on GFX9, where the mtype
    /// for the second page onward should be set to NC. It should never
    /// be used by user space applications.
    pub const CP_MQD_GFX9: u64 = 1 << 8;
    /// Flag that BO may contain sensitive data that must be wiped before
    /// releasing the memory
    pub const VRAM_WIPE_ON_RELEASE: u64 = 1 << 9;
    /// Flag that BO will be encrypted and that the TMZ bit should be
    /// set in the PTEs when mapping this buffer via GPUVM or
    /// accessing it with various hw blocks
    pub const ENCRYPTED: u64 = 1 << 10;
    /// Flag that BO will be used only in preemptible context, which does
    /// not require GTT memory accounting
    pub const PREEMPTIBLE: u64 = 1 << 11;
    /// Flag that BO can be discarded under memory pressure without keeping the
    /// content.
    pub const DISCARDABLE: u64 = 1 << 12;
    /// Flag that BO is shared coherently between multiple devices or CPU threads.
    /// May depend on GPU instructions to flush caches to system scope explicitly.
    ///
    /// This influences the choice of MTYPE in the PTEs on GFXv9 and later GPUs and
    /// may override the MTYPE selected in AMDGPU_VA_OP_MAP.
    pub const COHERENT: u64 = 1 << 13;
    /// Flag that BO should not be cached by GPU. Coherent without having to flush
    /// GPU caches explicitly.
    ///
    /// This influences the choice of MTYPE in the PTEs on GFXv9 and later GPUs and
    /// may override the MTYPE selected in AMDGPU_VA_OP_MAP.
    pub const UNCACHED: u64 = 1 << 14;
    /// Flag that BO should be coherent across devices when using device-level
    /// atomics. May depend on GPU instructions to flush caches to device scope
    /// explicitly, promoting them to system scope automatically.
    ///
    /// This influences the choice of MTYPE in the PTEs on GFXv9 and later GPUs and
    /// may override the MTYPE selected in AMDGPU_VA_OP_MAP.
    pub const EXT_COHERENT: u64 = 1 << 15;
    /// Set PTE.D and recompress during GTT->VRAM moves according to TILING flags.
    pub const GFX12_DCC: u64 = 1 << 16;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmAmdgpuGemCreateOut {
    /** returned GEM object handle */
    pub handle: u32,
    pub _pad: u32,
}

#[repr(C)]
pub union DrmAmdgpuGemCreate {
    pub input: DrmAmdgpuGemCreateIn,
    pub output: DrmAmdgpuGemCreateOut,
}
assert_layout!(DrmAmdgpuGemCreate, size = 32, align = 8);
define_amddrm_ioctl!(
    /// Creates a new gem object
    ///
    /// The resulting Gem object doesn't have to have the parameters you set here.
    /// You need to check the gem's properties lates.
    ///
    /// For example it can move the allocation to gtt if there is not enought vram free
    amdgpu_ioctl_gem_create, DrmAmdgpuGemCreate, 0x0, WR);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AmdgpuGemMetadataData {
    /// For future use, no flags defined so far
    pub flags: u64,
    /// Family specific tiling info
    pub tiling_info: u64,
    pub data_size_bytes: u32,
    pub data: [u32; 64],
}

pub mod metadata_op {
    pub type MetadataOp = u32;
    pub const SET: MetadataOp = 1;
    pub const GET: MetadataOp = 2;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmAmdgpuGemMetadata {
    pub handle: GemHandle,
    pub op: metadata_op::MetadataOp,
    pub data: AmdgpuGemMetadataData,
}
assert_layout!(DrmAmdgpuGemMetadata, size = 288, align = 8);
define_amddrm_ioctl!(amdgpu_ioctl_gem_metadata, DrmAmdgpuGemMetadata, 0x06, WR);

pub mod va_op {
    pub type AmdgpuVaOp = u32;
    pub const MAP: AmdgpuVaOp = 1;
    pub const UNMAP: AmdgpuVaOp = 2;
    pub const CLEAR: AmdgpuVaOp = 3;
    pub const REPLACE: AmdgpuVaOp = 4;
}

pub mod map_flags {
    pub type AmdgpuVaFlags = u32;
    /// Delay the page table update till the next CS
    pub const DELAY_UPDATE: AmdgpuVaFlags = 1 << 0;

    /// Readable mapping
    pub const PAGE_READABLE: AmdgpuVaFlags = 1 << 1;
    /// Writable mapping
    pub const PAGE_WRITEABLE: AmdgpuVaFlags = 1 << 2;
    /// Executable mapping, new for VI
    pub const PAGE_EXECUTABLE: AmdgpuVaFlags = 1 << 3;
    /// Partially resident texture
    pub const PAGE_PRT: AmdgpuVaFlags = 1 << 4;
    /// MTYPE flags mask (bits 5 to 8)
    pub const MTYPE_MASK: AmdgpuVaFlags = 0xf << 5;
    /// Default MTYPE. Pre-AI must use this. Recommended for newer ASICs.
    pub const MTYPE_DEFAULT: AmdgpuVaFlags = 0 << 5;
    /// Use Non Coherent MTYPE instead of default MTYPE
    pub const MTYPE_NC: AmdgpuVaFlags = 1 << 5;
    /// Use Write Combine MTYPE instead of default MTYPE
    pub const MTYPE_WC: AmdgpuVaFlags = 2 << 5;
    /// Use Cache Coherent MTYPE instead of default MTYPE
    pub const MTYPE_CC: AmdgpuVaFlags = 3 << 5;
    /// Use UnCached MTYPE instead of default MTYPE
    pub const MTYPE_UC: AmdgpuVaFlags = 4 << 5;
    /// Use Read Write MTYPE instead of default MTYPE
    pub const MTYPE_RW: AmdgpuVaFlags = 5 << 5;
    /// Don't allocate MALL
    pub const PAGE_NOALLOC: AmdgpuVaFlags = 1 << 9;
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmAmdgpuGemVa {
    /// GEM object handle
    pub handle: GemHandle,
    pub _pad: u32,
    pub operation: va_op::AmdgpuVaOp,
    /// AMDGPU_VM_PAGE_*
    pub flags: map_flags::AmdgpuVaFlags,
    /// VA address to assign. Must be correctly aligned.
    pub va_address: usize,
    /// Specify offset inside of BO to assign. Must be correctly aligned.
    pub offset_in_bo: usize,
    /// Specify mapping size. Must be correctly aligned.
    pub map_size: usize,
    /// Sequence number used to add new timeline point.
    pub vm_timeline_point: u64,
    /// The vm page table update fence is installed in given
    /// vm_timeline_syncobj_out at vm_timeline_point.
    pub vm_timeline_syncobj_out: u32,
    /// The number of syncobj handles in input_fence_syncobj_handles
    pub num_syncobj_handles: u32,
    /// Array of sync object handles to wait for given input fences
    pub input_fence_syncobj_handles: *const c_void,
}
assert_layout!(DrmAmdgpuGemVa, size = 64, align = 8);
define_amddrm_ioctl!(amdgpu_ioctl_gem_va, DrmAmdgpuGemVa, 0x08, W);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModeCrtc {
    pub id: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QueryHwIp {
    pub r#type: u32,

    /// Index of the IP if there are more IPs of the same
    /// type. Ignored by AMDGPU_INFO_HW_IP_COUNT.
    pub ip_instance: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ReadMMReag {
    pub dword_offset: u32,
    /** number of registers to read */
    pub count: u32,
    pub instance: u32,
    /** For future use, no flags defined so far */
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QueryFw {
    /** AMDGPU_INFO_FW_* */
    pub fw_type: u32,

    /// Index of the IP if there are more IPs of
    /// the same type.
    pub ip_instance: u32,
    /// Index of the engine. Whether this is used depends
    /// on the firmware type. (e.g. MEC, SDMA)
    pub index: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VbiosInfo {
    pub r#type: u32,
    pub offset: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SensorInfo {
    pub r#type: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VideoCap {
    pub r#type: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union drm_amdgpu_info_quick_info {
    mode_crtc: ModeCrtc,
    query_hw_ip: QueryHwIp,
    read_mmr_reg: ReadMMReag,
    query_fw: QueryFw,
    vbios_info: VbiosInfo,
    sensor_info: SensorInfo,
    video_cap: VideoCap,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DrmAmdgpuInfo {
    /* Where the return value will be stored */
    return_pointer: *mut c_void,
    /* The size of the return value. Just like "size" in "snprintf",
     * it limits how many bytes the kernel can write. */
    return_size: u32,
    /* The query request id. */
    query: u32,
    quick_info: drm_amdgpu_info_quick_info,
}
define_amddrm_ioctl!(amdgpu_ioctl_info, (), 0x05, W);
