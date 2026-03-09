use crate::drm::{GemHandle, SyncobjHandle};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemCreateIn {
    /** the requested memory size */
    pub bo_size: usize,
    /** physical start_addr alignment in bytes for some HW requirements */
    pub alignment: usize,
    /** the requested memory domains */
    pub domains: gem_domain::Type,
    /** allocation flags */
    pub domain_flags: gem_flags::Type,
}

pub mod gem_domain {
    pub type Type = u64;
    pub const CPU: Type = 1 << 0;
    pub const GTT: Type = 1 << 1;
    pub const VRAM: Type = 1 << 2;
    pub const GDS: Type = 1 << 3;
    pub const GWS: Type = 1 << 4;
    pub const OA: Type = 1 << 5;
    pub const DOORBELL: Type = 1 << 6;
    pub const MMIO_REMAP: Type = 1 << 7;
}

pub mod gem_flags {
    pub type Type = u64;
    /// Flag that CPU access will be required for the case of VRAM domain
    pub const CPU_ACCESS_REQUIRED: Type = 1 << 0;
    /// Flag that CPU access will not work, this VRAM domain is invisible
    pub const NO_CPU_ACCESS: Type = 1 << 1;
    /// Flag that USWC attributes should be used for GTT
    pub const CPU_GTT_USWC: Type = 1 << 2;
    /// Flag that the memory should be in VRAM and cleared
    pub const VRAM_CLEARED: Type = 1 << 3;
    /// Flag that allocating the BO should use linear VRAM
    pub const VRAM_CONTIGUOUS: Type = 1 << 5;
    /// Flag that BO is always valid in this VM
    pub const VM_ALWAYS_VALID: Type = 1 << 6;
    /// Flag that BO sharing will be explicitly synchronized
    pub const EXPLICIT_SYNC: Type = 1 << 7;
    /// Flag that indicates allocating MQD gart on GFX9, where the mtype
    /// for the second page onward should be set to NC. It should never
    /// be used by user space applications.
    pub const CP_MQD_GFX9: Type = 1 << 8;
    /// Flag that BO may contain sensitive data that must be wiped before
    /// releasing the memory
    pub const VRAM_WIPE_ON_RELEASE: Type = 1 << 9;
    /// Flag that BO will be encrypted and that the TMZ bit should be
    /// set in the PTEs when mapping this buffer via GPUVM or
    /// accessing it with various hw blocks
    pub const ENCRYPTED: Type = 1 << 10;
    /// Flag that BO will be used only in preemptible context, which does
    /// not require GTT memory accounting
    pub const PREEMPTIBLE: Type = 1 << 11;
    /// Flag that BO can be discarded under memory pressure without keeping the
    /// content.
    pub const DISCARDABLE: Type = 1 << 12;
    /// Flag that BO is shared coherently between multiple devices or CPU threads.
    /// May depend on GPU instructions to flush caches to system scope explicitly.
    ///
    /// This influences the choice of MTYPE in the PTEs on GFXv9 and later GPUs and
    /// may override the MTYPE selected in AMDGPU_VA_OP_MAP.
    pub const COHERENT: Type = 1 << 13;
    /// Flag that BO should not be cached by GPU. Coherent without having to flush
    /// GPU caches explicitly.
    ///
    /// This influences the choice of MTYPE in the PTEs on GFXv9 and later GPUs and
    /// may override the MTYPE selected in AMDGPU_VA_OP_MAP.
    pub const UNCACHED: Type = 1 << 14;
    /// Flag that BO should be coherent across devices when using device-level
    /// atomics. May depend on GPU instructions to flush caches to device scope
    /// explicitly, promoting them to system scope automatically.
    ///
    /// This influences the choice of MTYPE in the PTEs on GFXv9 and later GPUs and
    /// may override the MTYPE selected in AMDGPU_VA_OP_MAP.
    pub const EXT_COHERENT: Type = 1 << 15;
    /// Set PTE.D and recompress during GTT->VRAM moves according to TILING flags.
    pub const GFX12_DCC: Type = 1 << 16;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemCreateOut {
    /** returned GEM object handle */
    pub handle: u32,
    pub _pad: u32,
}

#[repr(C)]
pub union GemCreate {
    pub input: GemCreateIn,
    pub output: GemCreateOut,
}
assert_layout!(GemCreate, size = 32, align = 8);

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
pub union InfoUnion {
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
pub struct Info {
    /* Where the return value will be stored */
    return_pointer: *mut (),
    /* The size of the return value. Just like "size" in "snprintf",
     * it limits how many bytes the kernel can write. */
    return_size: u32,
    /* The query request id. */
    query: u32,
    quick_info: InfoUnion,
}
assert_layout!(Info, size = 32, align = 8);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemMetadataData {
    /// For future use, no flags defined so far
    pub flags: u64,
    /// Family specific tiling info
    pub tiling_info: u64,
    pub data_size_bytes: u32,
    pub data: [u32; 64],
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum MetadataOp {
    Set = 1,
    Get = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemMetadata {
    pub handle: GemHandle,
    pub op: MetadataOp,
    pub data: GemMetadataData,
}
assert_layout!(GemMetadata, size = 288, align = 8);

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum VaOp {
    Map = 1,
    Unmap = 2,
    Clear = 3,
    Replace = 4,
}

pub mod map_flags {
    pub type VaFlags = u32;
    /// Delay the page table update till the next CS
    pub const DELAY_UPDATE: VaFlags = 1 << 0;

    /// Readable mapping
    pub const PAGE_READABLE: VaFlags = 1 << 1;
    /// Writable mapping
    pub const PAGE_WRITEABLE: VaFlags = 1 << 2;
    /// Executable mapping, new for VI
    pub const PAGE_EXECUTABLE: VaFlags = 1 << 3;
    /// Partially resident texture
    pub const PAGE_PRT: VaFlags = 1 << 4;
    /// MTYPE flags mask (bits 5 to 8)
    pub const MTYPE_MASK: VaFlags = 0xf << 5;
    /// Default MTYPE. Pre-AI must use this. Recommended for newer ASICs.
    pub const MTYPE_DEFAULT: VaFlags = 0 << 5;
    /// Use Non Coherent MTYPE instead of default MTYPE
    pub const MTYPE_NC: VaFlags = 1 << 5;
    /// Use Write Combine MTYPE instead of default MTYPE
    pub const MTYPE_WC: VaFlags = 2 << 5;
    /// Use Cache Coherent MTYPE instead of default MTYPE
    pub const MTYPE_CC: VaFlags = 3 << 5;
    /// Use UnCached MTYPE instead of default MTYPE
    pub const MTYPE_UC: VaFlags = 4 << 5;
    /// Use Read Write MTYPE instead of default MTYPE
    pub const MTYPE_RW: VaFlags = 5 << 5;
    /// Don't allocate MALL
    pub const PAGE_NOALLOC: VaFlags = 1 << 9;
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemVa {
    /// GEM object handle
    pub handle: GemHandle,
    pub _pad: u32,
    pub operation: VaOp,
    /// AMDGPU_VM_PAGE_*
    pub flags: map_flags::VaFlags,
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
    pub input_fence_syncobj_handles: *const SyncobjHandle,
}
assert_layout!(GemVa, size = 64, align = 8);
