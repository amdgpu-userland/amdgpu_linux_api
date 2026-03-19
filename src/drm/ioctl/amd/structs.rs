use crate::{
    drm::{
        self, GemHandle, SyncobjHandle,
        ioctl::amd::{BoListHandle, CsFence, CtxId, FenceHandle, IpInstance, IpRing, SyncobjSeqNo},
    },
    kfd::ioctl::VirtualAddress,
};

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
pub struct GemMmapIn {
    /// the GEM object handle
    pub handle: GemHandle,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemMmapOut {
    /// mmap offset from the vma offset manager
    pub addr_ptr: u64,
}

#[repr(C)]
pub union GemMmap {
    pub in_: GemMmapIn,
    pub out: GemMmapOut,
}
assert_layout!(GemMmap, size = 8, align = 8);

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CtxOp {
    AllocCtx = 1,
    FreeCtx = 2,
    QueryState = 3,
    QueryState2 = 4,
    GetStablePstate = 5,
    SetStablePstate = 6,
}

/* GPU reset status */
pub const AMDGPU_CTX_NO_RESET: u32 = 0;
pub const AMDGPU_CTX_GUILTY_RESET: u32 = 1;
pub const AMDGPU_CTX_INNOCENT_RESET: u32 = 2;
pub const AMDGPU_CTX_UNKNOWN_RESET: u32 = 3;

/* QUERY2 flags */
pub const AMDGPU_CTX_QUERY2_FLAGS_RESET: u32 = 1 << 0;
pub const AMDGPU_CTX_QUERY2_FLAGS_VRAMLOST: u32 = 1 << 1;
pub const AMDGPU_CTX_QUERY2_FLAGS_GUILTY: u32 = 1 << 2;
pub const AMDGPU_CTX_QUERY2_FLAGS_RAS_CE: u32 = 1 << 3;
pub const AMDGPU_CTX_QUERY2_FLAGS_RAS_UE: u32 = 1 << 4;
pub const AMDGPU_CTX_QUERY2_FLAGS_RESET_IN_PROGRESS: u32 = 1 << 5;

/// Context priority levels
///
/// Any other value is treated as Unset, which defaults to Normal
/// Priority > Normal requires CAP_SYS_NICE or drm master
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum CtxPriority {
    Unset = -2048,
    VeryLow = -1023,
    Low = -512,
    Normal = 0,
    High = 512,
    VeryHigh = 1023,
}

/* Stable pstate */
pub const AMDGPU_CTX_STABLE_PSTATE_FLAGS_MASK: u32 = 0xf;
pub const AMDGPU_CTX_STABLE_PSTATE_NONE: u32 = 0;
pub const AMDGPU_CTX_STABLE_PSTATE_STANDARD: u32 = 1;
pub const AMDGPU_CTX_STABLE_PSTATE_MIN_SCLK: u32 = 2;
pub const AMDGPU_CTX_STABLE_PSTATE_MIN_MCLK: u32 = 3;
pub const AMDGPU_CTX_STABLE_PSTATE_PEAK: u32 = 4;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CtxIn {
    pub op: CtxOp,
    /// Set to 0 except for SET_PSTATE op
    pub flags: u32,
    pub ctx_id: CtxId,
    /// Only used for ALLOC_CTX
    pub priority: CtxPriority,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CtxOutAlloc {
    pub ctx_id: CtxId,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CtxOutState {
    /// For future use, no flags defined so far
    pub flags: u64,
    /// Number of resets caused by this context so far
    pub hangs: u32,
    /// Reset status since the last call of the ioctl
    pub reset_status: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CtxOutPstate {
    pub flags: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union CtxOut {
    pub alloc: CtxOutAlloc,
    pub state: CtxOutState,
    pub pstate: CtxOutPstate,
}

#[repr(C)]
pub union Ctx {
    pub in_: CtxIn,
    pub out: CtxOut,
}
assert_layout!(Ctx, size = 16, align = 8);

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum BoListOp {
    Create,
    Destroy,
    Update,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BoListIn {
    /// Type of operation
    pub operation: BoListOp,
    /// Handle of list or 0 if we want to create one
    pub list_handle: BoListHandle,
    /// Number of BOs in bo_info_ptr
    pub bo_number: u32,
    /// Size of each element describing BO
    /// size_of::<BoListEntry>()
    pub bo_info_size: u32,
    /// Pointer to array describing BOs
    ///
    /// It teoretically accept any type which is no larger than BoListEntry
    /// but internally it allocates space as if for BoListEntry and simply casts provided data
    ///
    /// It allows for extending BoListEntry in the future
    pub bo_info_ptr: *const BoListEntry,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BoListEntry {
    /// Handle of BO
    pub bo_handle: u32,
    /// New (if specified) BO priority to be used during migration
    pub bo_priority: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BoListOut {
    /// Handle of resource list
    pub list_handle: BoListHandle,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union BoList {
    pub in_: BoListIn,
    pub out: BoListOut,
}
assert_layout!(BoList, size = 24, align = 8);

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ChunkId {
    /// CsChunkIb
    /// You can have multiple of these in a submission
    ///
    /// There is a limit of at most 4 IBs (jobs) in a gang (submission)
    ///
    /// For some rings the content might be validated (parse_cs) or changed (patch_cs_in_place)
    /// by ring driver
    IB = 0x01,
    /// CsChunkFence
    /// You only want one of these in a submission
    ///
    /// Some rings may not support user fences
    FENCE = 0x02,
    /// CsChunkDep
    /// You can have multiple of these in a submission
    /// Accepts an array, just set length_in_dw to array len * sizeof(ChunkDep) / 4
    DEPENDENCIES = 0x03,
    /// CsChunkSem
    /// You can have multiple of these in a submission
    /// Accepts an array, just set length_in_dw to array len * sizeof(ChunkSem) / 4
    SYNCOBJ_IN = 0x04,
    /// CsChunkSem
    /// You can have only one of these in a submission and you have to choose between
    /// this and SYNCOBJ_TIMELINE_SIGNAL
    /// Accepts an array, just set length_in_dw to array len * sizeof(ChunkSem) / 4
    SYNCOBJ_OUT = 0x05,
    /// BoListIn
    /// You can have only one of these in a submission
    BO_HANDLES = 0x06,
    /// CsChunkDep
    /// You can have multiple of these in a submission
    /// Accepts an array, just set length_in_dw to array len * sizeof(ChunkDep) / 4
    SCHEDULED_DEPENDENCIES = 0x07,
    /// CsChunkSyncobj
    /// You can have multiple of these in a submission
    /// Accepts an array, just set length_in_dw to array len * sizeof(ChunkSyncobj) / 4
    SYNCOBJ_TIMELINE_WAIT = 0x08,
    /// CsChunkSyncobj
    /// You only want one of these in a submission and you have to choose between
    /// this and SYNCOBJ_OUT
    /// Accepts an array, just set length_in_dw to array len * sizeof(ChunkSyncobj) / 4
    SYNCOBJ_TIMELINE_SIGNAL = 0x09,
    /// CsChunkCpGfxShadow
    /// You only want one of these in a submission
    /// Used by gfx11
    CP_GFX_SHADOW = 0x0a,
}

pub mod amdgpu_ib_flag {
    pub type Type = u32;
    /// This IB should be submitted to Constant Engine
    /// otherwise to drawing engine
    pub const CE: Type = 1 << 0;
    /// Preamble flag, IB could be dropped if no context switch
    pub const PREAMBLE: Type = 1 << 1;
    /// IB should set Pre_enb bit if PREEMPT flag detected
    ///
    /// For ID_GFX only one IB can have this flag
    pub const PREEMPT: Type = 1 << 2;
    /// IB fence should do L2 writeback but not invalidate any shader caches
    pub const TC_WB_NOT_INVALIDATE: Type = 1 << 3;
    /// Set GDS_COMPUTE_MAX_WAVE_ID = DEFAULT before PACKET3_INDIRECT_BUFFER,
    /// resetting wave ID counters for the IB
    pub const RESET_GDS_MAX_WAVE_ID: Type = 1 << 4;
    /// Flag the IB as secure (TMZ)
    pub const SECURE: Type = 1 << 5;
    /// Tell KMD to flush and invalidate caches
    pub const EMIT_MEM_SYNC: Type = 1 << 6;
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum HwIp {
    GFX = 0,
    COMPUTE = 1,
    DMA = 2,
    UVD = 3,
    VCE = 4,
    UVD_ENC = 5,
    VCN_DEC = 6,
    /// From VCN4, AMDGPU_HW_IP_VCN_ENC is re-used to support
    /// both encoding and decoding jobs.
    VCN_ENC = 7,
    VCN_JPEG = 8,
    VPE = 9,
    NUM = 10,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsChunkIb {
    pub _pad: u32,
    pub flags: amdgpu_ib_flag::Type,
    /// Virtual address to memory with hw_ip specific instructions to begin execution
    /// It must be already mapped
    pub va_start: u64,
    /// Size of submission in bytes, must be 4 bytes aligned
    pub ib_bytes: u32,
    /// There is a limit of using at most 4 different rings (entities) in a submission
    pub ip_type: HwIp,
    pub ip_instance: IpInstance,
    pub ring: IpRing,
}
assert_layout!(CsChunkIb, size = 32, align = 8);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsChunkDep {
    pub ip_type: HwIp,
    pub ip_instance: IpInstance,
    pub ring: IpRing,
    pub ctx_id: CtxId,
    pub handle: FenceHandle,
}
assert_layout!(CsChunkDep, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsChunkFence {
    /// GEM must have PAGE_SIZE size
    /// It cannot be a userptr GEM
    pub handle: GemHandle,
    /// Must be a valid offset (in bytes) to a u64 in provided GEM
    ///
    /// It best if you align it too, as it's hw_ip and generation
    /// dependant if you get undefined behaviour
    pub offset: u32,
}
assert_layout!(CsChunkFence, size = 8, align = 4);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsChunkSem {
    pub handle: SyncobjHandle,
}
assert_layout!(CsChunkSem, size = 4, align = 4);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsChunkSyncobj {
    pub handle: SyncobjHandle,
    /// DRM_SYNCOBJ_WAIT_FLAGS_ for SyncoobjWait
    /// Unused for SyncobjSignal
    pub flags: u32,
    pub point: SyncobjSeqNo,
}
assert_layout!(CsChunkSyncobj, size = 16, align = 8);

pub mod cs_chunk_cp_gfx_shadow_flags {
    pub type Type = u64;
    pub const INIT_SHADOW: Type = 0x1;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsChunkCpGfxShadow {
    pub shadow_va: VirtualAddress,
    pub csa_va: VirtualAddress,
    pub gds_va: VirtualAddress,
    pub flags: cs_chunk_cp_gfx_shadow_flags::Type,
}
assert_layout!(CsChunkCpGfxShadow, size = 32, align = 8);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsChunk {
    pub chunk_id: ChunkId,
    /// Size of the struct chunk_data points to
    /// Unit: dword (4bytes)
    pub length_dw: u32,
    /// Ptr to struct depending on chunk_id
    pub chunk_data: u64,
}
assert_layout!(CsChunk, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsIn {
    pub ctx_id: CtxId,
    pub bo_list_handle: BoListHandle,
    pub num_chunks: u32,
    pub flags: u32,
    /// Array of pointers to CsChunk
    /// The order of most chunks matters
    pub chunks: *const *const CsChunk,
}
assert_layout!(CsIn, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CsOut {
    pub handle: CsFence,
}
assert_layout!(CsOut, size = 8, align = 8);

#[repr(C)]
pub union Cs {
    pub in_: CsIn,
    pub out: CsOut,
}
assert_layout!(Cs, size = 24, align = 8);

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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CsWaitIn {
    /// Command submission handle.
    /// - `0` means none to wait for
    /// - `!0u64` means wait for the latest sequence number
    pub handle: CsFence,
    /// Absolute timeout in nanoseconds to wait
    pub timeout: u64,
    pub ip_type: HwIp,
    pub ip_instance: IpInstance,
    pub ring: IpRing,
    pub ctx_id: CtxId,
}
assert_layout!(CsWaitIn, size = 32, align = 8);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CsWaitOut {
    /// TODO: To be verified yet
    /// CS status: `0` = CS completed, `1` = timeout
    pub status: u64,
}
assert_layout!(CsWaitOut, size = 8, align = 8);

#[repr(C)]
pub union CsWait {
    pub in_: CsWaitIn,
    pub out: CsWaitOut,
}
assert_layout!(CsWait, size = 32, align = 8);
