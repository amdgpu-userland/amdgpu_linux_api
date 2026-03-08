use std::{ffi::c_void, os::fd::RawFd};
pub type QueueId = u32;

pub type Errno = libc::c_int;
pub type GpuId = u32;
pub type MemoryHandle = u64;
pub type AllocMemFlag = u32;
pub type VirtualAddress = u64;

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdVersion {
    pub major: u32,
    pub minor: u32,
}
assert_layout!(KfdVersion, size = 8, align = 4);

pub type QueueType = u32;
pub const COMPUTE: QueueType = 0;
pub const SDMA: QueueType = 1;
pub const COMPUTE_AQL: QueueType = 2;
pub const SDMA_XGMI: QueueType = 3;
pub const SDMA_BY_ENG_ID: QueueType = 4;

pub mod queue_limits {
    pub const KFD_MAX_QUEUE_PERCENTAGE: u32 = 100;
    pub const KFD_MAX_QUEUE_PRIORITY: u32 = 15;
    pub const KFD_MIN_QUEUE_RING_SIZE: u32 = 1024;
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlCreateQueueArgs {
    pub ring_base_address: u64,     /* to KFD */
    pub write_pointer_address: u64, /* to KFD */
    pub read_pointer_address: u64,  /* to KFD */
    pub doorbell_offset: u64,       /* from KFD */

    pub ring_size: u32,        /* to KFD */
    pub gpu_id: GpuId,         /* to KFD */
    pub queue_type: QueueType, /* to KFD */
    pub queue_percentage: u32, /* to KFD */
    pub queue_priority: u32,   /* to KFD */
    pub queue_id: QueueId,     /* from KFD */

    pub eop_buffer_address: u64,       /* to KFD */
    pub eop_buffer_size: u64,          /* to KFD */
    pub ctx_save_restore_address: u64, /* to KFD */
    pub ctx_save_restore_size: u32,    /* to KFD */
    pub ctl_stack_size: u32,           /* to KFD */
    pub sdma_engine_id: u32,           /* to KFD */
    pub pad: u32,
}
assert_layout!(KfdIoctlCreateQueueArgs, size = 96, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlDestroyQueueArgs {
    pub queue_id: QueueId, /* to KFD */
    pub pad: u32,
}
assert_layout!(KfdIoctlDestroyQueueArgs, size = 8, align = 4);

pub mod cache_policy {
    pub type CachePolicy = u32;
    pub const COHERENT: CachePolicy = 0;
    pub const NON_COHERENT: CachePolicy = 1;
}
pub type MiscProcessFlags = u32;
pub const KFD_PROC_FLAG_MFMA_HIGH_PRECISION: MiscProcessFlags = 1 << 0;

#[repr(C)]
pub struct KfdIoctlSetMemoryPolicyArgs {
    pub alternate_aperture_base: usize,
    pub alternate_aperture_size: usize,
    pub gpu_id: GpuId,
    pub default_policy: cache_policy::CachePolicy,
    pub alternate_policy: cache_policy::CachePolicy,
    pub misc_process_flag: MiscProcessFlags,
}
assert_layout!(KfdIoctlSetMemoryPolicyArgs, size = 32, align = 8);

#[repr(C)]
pub struct KfdIoctlGetClockCountersArgs {
    pub gpu_clock_counter: u64,
    pub cpu_clock_counter: u64,
    pub system_clock_counter: u64,
    pub system_clock_freq: u64,
    pub gpu_id: GpuId,
    pub _pad: u32,
}
assert_layout!(KfdIoctlGetClockCountersArgs, size = 40, align = 8);

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct KfdProcessDeviceApertures {
    pub lds_base: u64,      /* from KFD */
    pub lds_limit: u64,     /* from KFD */
    pub scratch_base: u64,  /* from KFD */
    pub scratch_limit: u64, /* from KFD */
    pub gpuvm_base: u64,    /* from KFD */
    pub gpuvm_limit: u64,   /* from KFD */
    pub gpu_id: GpuId,      /* from KFD */
    pub _pad: u32,
}
assert_layout!(KfdProcessDeviceApertures, size = 56, align = 8);

impl std::fmt::Debug for KfdProcessDeviceApertures {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KfdProcessDeviceApertures")
            .field("lds_base", &format_args!("{:#018x}", self.lds_base))
            .field("lds_limit", &format_args!("{:#018x}", self.lds_limit))
            .field("scratch_base", &format_args!("{:#018x}", self.scratch_base))
            .field(
                "scratch_limit",
                &format_args!("{:#018x}", self.scratch_limit),
            )
            .field("gpuvm_base", &format_args!("{:#018x}", self.gpuvm_base))
            .field("gpuvm_limit", &format_args!("{:#018x}", self.gpuvm_limit))
            .field("gpu_id", &self.gpu_id)
            .field("_pad", &self._pad)
            .finish()
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetProcessAperturesArgs {
    pub process_apertures: [KfdProcessDeviceApertures; 7],
    pub num_of_nodes: u32,
    pub _pad: u32,
}
assert_layout!(KfdIoctlGetProcessAperturesArgs, size = 400, align = 8);

#[repr(C)]
pub struct KfdIoctlUpdateQueueArgs {
    /// Ring base address (to KFD)
    pub ring_base_address: u64,
    /// Queue ID (to KFD)
    pub queue_id: QueueId,
    /// Ring size (to KFD)
    pub ring_size: u32,
    /// Queue percentage (to KFD)
    pub queue_percentage: u32,
    /// Queue priority (to KFD)
    pub queue_priority: u32,
}
assert_layout!(KfdIoctlUpdateQueueArgs, size = 24, align = 8);

pub mod event_type {
    pub type EventType = u32;
    pub const SIGNAL: EventType = 0;
    pub const NODE_CHANGE: EventType = 1;
    pub const DEVICE_STATE_CHANGE: EventType = 2;
    pub const HW_EXCEPTION: EventType = 3;
    pub const SYSTEM_EVENT: EventType = 4;
    pub const DEBUG_EVENT: EventType = 5;
    pub const PROFILE_EVENT: EventType = 6;
    pub const QUEUE_EVENT: EventType = 7;
    pub const MEMORY: EventType = 8;
}
pub type EventId = u32;
#[repr(C)]
pub struct KfdIoctlCreateEventArgs {
    pub event_page_offset: u64,
    pub event_trigger_data: u32,
    pub event_type: event_type::EventType,
    pub auto_reset: u32,
    pub node_id: u32,
    pub event_id: EventId,
    pub event_slot_index: u32,
}
assert_layout!(KfdIoctlCreateEventArgs, size = 32, align = 8);

#[repr(C)]
pub struct KfdIoctlDestroyEventArgs {
    pub event_id: EventId,
    pub _pad: u32,
}
assert_layout!(KfdIoctlDestroyEventArgs, size = 8, align = 4);

#[repr(C)]
pub struct KfdIoctlSetEventArgs {
    pub event_id: EventId,
    pub _pad: u32,
}
assert_layout!(KfdIoctlSetEventArgs, size = 8, align = 4);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KfdMemoryExceptionFailure {
    /// Page not present or supervisor privilege
    pub not_present: u32,
    /// Write access to a read-only page
    pub read_only: u32,
    /// Execute access to a page marked NX
    pub no_execute: u32,
    /// Can't determine the exact fault address
    pub imprecise: u32,
}
assert_layout!(KfdMemoryExceptionFailure, size = 16, align = 4);

pub mod hsa_mem_exception {
    pub type FailureType = u32;
    pub const NO_RAS: FailureType = 0;
    pub const ECC_SRAM: FailureType = 1;
    pub const POISON_CONSUMED: FailureType = 2;
    pub const GPU_HANG: FailureType = 3;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KfdHsaMemoryExceptionData {
    pub failure: KfdMemoryExceptionFailure,
    pub va: VirtualAddress,
    pub gpu_id: GpuId,
    /// 0 = no RAS error, 1 = ECC_SRAM, 2 = Link_SYNFLOOD (poison),
    /// 3 = GPU hang (not attributable to a specific cause), other values reserved
    pub error_type: hsa_mem_exception::FailureType,
}
assert_layout!(KfdHsaMemoryExceptionData, size = 32, align = 8);

pub mod hw_reset_type {
    pub type ResetType = u32;
    pub const WHOLE_GPU_RESET: ResetType = 0;
    pub const PER_ENGINE_RESET: ResetType = 1;
}

pub mod hw_reset_cause {
    pub type ResetCause = u32;
    pub const GPU_HANG: ResetCause = 0;
    pub const ECC: ResetCause = 1;
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KfdHsaHwExceptionData {
    pub reset_type: hw_reset_type::ResetType,
    pub reset_cause: hw_reset_cause::ResetCause,
    pub memory_lost: u32,
    pub gpu_id: GpuId,
}
assert_layout!(KfdHsaHwExceptionData, size = 16, align = 4);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KfdHsaSignalEventData {
    pub last_event_age: u64,
}
assert_layout!(KfdHsaSignalEventData, size = 8, align = 8);

#[repr(C)]
pub union KfdEventDataUnion {
    pub memory_exception_data: KfdHsaMemoryExceptionData,
    pub hw_exception_data: KfdHsaHwExceptionData,
    pub signal_event_data: KfdHsaSignalEventData,
}

#[repr(C)]
pub struct KfdEventData {
    pub data: KfdEventDataUnion,
    /// Pointer to an extension structure for future exception types
    pub kfd_event_data_ext: *mut c_void,
    pub event_id: EventId,
    pub _pad: u32,
}
assert_layout!(KfdEventData, size = 48, align = 8);

pub mod wait_result {
    pub type WaitResult = u32;
    pub const COMPLETE: WaitResult = 0;
    pub const TIMEOUT: WaitResult = 1;
    pub const FAIL: WaitResult = 2;
}
#[repr(C)]
pub struct KfdIoctlWaitEventsArgs {
    pub events_ptr: *mut KfdEventData,
    pub num_events: u32,
    pub wait_for_all: u32,
    /// In milicesonds
    pub timeout: u32,
    pub wait_result: wait_result::WaitResult,
}
assert_layout!(KfdIoctlWaitEventsArgs, size = 24, align = 8);

#[repr(C)]
pub struct KfdIoctlResetEventArgs {
    pub event_id: EventId,
    pub _pad: u32,
}
assert_layout!(KfdIoctlResetEventArgs, size = 8, align = 4);

#[repr(C)]
pub struct KfdIoctlDbgRegisterArgs {
    pub gpu_id: GpuId,
    pub _pad: u32,
}
assert_layout!(KfdIoctlDbgRegisterArgs, size = 8, align = 4);

#[repr(C)]
pub struct KfdIoctlDbgUnregisterArgs {
    pub gpu_id: GpuId,
    pub _pad: u32,
}
assert_layout!(KfdIoctlDbgUnregisterArgs, size = 8, align = 4);

#[repr(C)]
pub struct KfdIoctlDbgAddressWatchArgs {
    pub content_ptr: *const c_void, // a pointer to the actual content
    pub gpu_id: GpuId,
    pub buf_size_in_bytes: u32,
}
assert_layout!(KfdIoctlDbgAddressWatchArgs, size = 16, align = 8);

#[repr(C)]
pub struct KfdIoctlDbgWaveControlArgs {
    pub content_ptr: *const c_void,
    pub gpu_id: GpuId,
    pub buf_size_in_bytes: u32,
}
assert_layout!(KfdIoctlDbgWaveControlArgs, size = 16, align = 8);

#[repr(C)]
pub struct KfdIoctlSetScratchBackingVaArgs {
    pub va_addr: VirtualAddress, // to KFD
    pub gpu_id: GpuId,           // to KFD
    pub _pad: u32,
}
assert_layout!(KfdIoctlSetScratchBackingVaArgs, size = 16, align = 8);

#[repr(C)]
pub struct KfdIoctlGetTileConfigArgs {
    /// to KFD: pointer to tile array
    pub tile_config_ptr: *const c_void,
    /// to KFD: pointer to macro tile array
    pub macro_tile_config_ptr: *const c_void,
    /// to KFD: array size allocated by user mode
    /// from KFD: array size filled by kernel
    pub num_tile_configs: u32,
    /// to KFD: array size allocated by user mode
    /// from KFD: array size filled by kernel
    pub num_macro_tile_configs: u32,
    pub gpu_id: GpuId,       // to KFD
    pub gb_addr_config: u32, // from KFD
    pub num_banks: u32,      // from KFD
    pub num_ranks: u32,      // from KFD
}
assert_layout!(KfdIoctlGetTileConfigArgs, size = 40, align = 8);

#[repr(C)]
pub struct KfdIoctlSetTrapHandlerArgs {
    pub tba_addr: VirtualAddress, // to KFD
    pub tma_addr: VirtualAddress, // to KFD
    pub gpu_id: GpuId,            // to KFD
    pub _pad: u32,
}
assert_layout!(KfdIoctlSetTrapHandlerArgs, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetProcessAperturesNewArgs {
    /* User allocated. Pointer to struct kfd_process_device_apertures
     * filled in by Kernel
     */
    pub kfd_process_device_apertures_ptr: *mut KfdProcessDeviceApertures,
    /* to KFD - indicates amount of memory present in
     *  kfd_process_device_apertures_ptr
     * from KFD - Number of entries filled by KFD.
     */
    pub num_of_nodes: u32,
    pub _pad: u32,
}
assert_layout!(KfdIoctlGetProcessAperturesNewArgs, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlAcquireVmArgs {
    pub drm_fd: RawFd,
    pub gpu_id: GpuId,
}
assert_layout!(KfdIoctlAcquireVmArgs, size = 8, align = 4);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlAllocMemoryOfGpuArgs {
    pub va_addr: VirtualAddress, /* to KFD */
    pub size: usize,             /* to KFD */
    pub handle: MemoryHandle,    /* from KFD */
    pub mmap_offset: u64,        /* to KFD (userptr), from KFD (mmap offset) */
    pub gpu_id: u32,             /* to KFD */
    pub flags: AllocMemFlag,
}
assert_layout!(KfdIoctlAllocMemoryOfGpuArgs, size = 40, align = 8);
/// Allocation flags: memory types, pick only one
pub mod alloc_domain {
    use super::AllocMemFlag;

    pub const VRAM: AllocMemFlag = 1 << 0;

    pub const GTT: AllocMemFlag = 1 << 1;

    pub const USERPTR: AllocMemFlag = 1 << 2;

    pub const DOORBELL: AllocMemFlag = 1 << 3;

    pub const MMIO_REMAP: AllocMemFlag = 1 << 4;
}

/// Allocation flags: attributes/access options
pub mod alloc_flags {
    use super::AllocMemFlag;

    pub const WRITABLE: AllocMemFlag = 1 << 31;

    pub const EXECUTABLE: AllocMemFlag = 1 << 30;

    pub const PUBLIC: AllocMemFlag = 1 << 29;

    pub const NO_SUBSTITUTE: AllocMemFlag = 1 << 28;

    pub const AQL_QUEUE_MEM: AllocMemFlag = 1 << 27;

    pub const COHERENT: AllocMemFlag = 1 << 26;

    pub const UNCACHED: AllocMemFlag = 1 << 25;

    pub const EXT_COHERENT: AllocMemFlag = 1 << 24;

    pub const CONTIGUOUS: AllocMemFlag = 1 << 23;
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlFreeMemoryOfGpuArgs {
    pub handle: MemoryHandle, /* to KFD */
}
assert_layout!(KfdIoctlFreeMemoryOfGpuArgs, size = 8, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlMapMemoryToGpuArgs {
    pub handle: MemoryHandle,      /* to KFD */
    pub device_ids_array_ptr: u64, /* to KFD */
    pub n_devices: u32,            /* to KFD */
    pub n_success: u32,            /* to/from KFD */
}
assert_layout!(KfdIoctlMapMemoryToGpuArgs, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlUnmapMemoryFromGpuArgs {
    pub handle: MemoryHandle,      /* to KFD */
    pub device_ids_array_ptr: u64, /* to KFD */
    pub n_devices: u32,            /* to KFD */
    pub n_success: u32,            /* to/from KFD */
}
assert_layout!(KfdIoctlUnmapMemoryFromGpuArgs, size = 24, align = 8);

#[repr(C)]
pub struct KfdIoctlSetCuMaskArgs {
    pub queue_id: QueueId, // to KFD
    /// Bit count (len() * 32)
    pub num_cu_mask: u32, // to KFD
    pub cu_mask_ptr: *const u32, // to KFD
}
assert_layout!(KfdIoctlSetCuMaskArgs, size = 16, align = 8);

#[repr(C)]
pub struct KfdIoctlGetQueueWaveStateArgs {
    pub ctl_stack_address: u64,   // to KFD
    pub ctl_stack_used_size: u32, // from KFD
    pub save_area_used_size: u32, // from KFD
    pub queue_id: QueueId,        // to KFD
    pub _pad: u32,
}
assert_layout!(KfdIoctlGetQueueWaveStateArgs, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetDmabufInfoArgs {
    /// Underlying buffer size in bytes
    pub size: u64, /* from KFD */
    /// Ptr to user allocated memory, where the currntly set metadata
    /// will be copied to,
    pub metadata_ptr: u64, /* to KFD */
    pub metadata_size: u32, /* to KFD (space allocated by user)
                             * from KFD (actual metadata size)
                             */
    pub gpu_id: GpuId,    /* from KFD */
    pub flags: u32,       /* from KFD (KFD_IOC_ALLOC_MEM_FLAGS) */
    pub dmabuf_fd: RawFd, /* to KFD */
}
assert_layout!(KfdIoctlGetDmabufInfoArgs, size = 32, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlImportDmabufArgs {
    pub va_addr: VirtualAddress, /* to KFD */
    pub handle: MemoryHandle,    /* from KFD */
    pub gpu_id: GpuId,           /* to KFD */
    pub dmabuf_fd: RawFd,        /* to KFD */
}
assert_layout!(KfdIoctlImportDmabufArgs, size = 24, align = 8);

#[repr(C)]
pub struct KfdIoctlAllocQueueGwsArgs {
    pub queue_id: QueueId, // to KFD
    pub num_gws: u32,      // to KFD
    pub first_gws: u32,    // from KFD
    pub _pad: u32,
}
assert_layout!(KfdIoctlAllocQueueGwsArgs, size = 16, align = 4);

pub mod kfd_smi_event {
    pub type Type = u64;
    pub const NONE: Type = 0; /* not used */
    pub const VMFAULT: Type = 1; /* event start counting at 1 */
    pub const THERMAL_THROTTLE: Type = 2;
    pub const GPU_PRE_RESET: Type = 3;
    pub const GPU_POST_RESET: Type = 4;
    pub const MIGRATE_START: Type = 5;
    pub const MIGRATE_END: Type = 6;
    pub const PAGE_FAULT_START: Type = 7;
    pub const PAGE_FAULT_END: Type = 8;
    pub const QUEUE_EVICTION: Type = 9;
    pub const QUEUE_RESTORE: Type = 10;
    pub const UNMAP_FROM_GPU: Type = 11;
    pub const PROCESS_START: Type = 12;
    pub const PROCESS_END: Type = 13;
    /// Max event number, as filteing mask is 64bits wide.
    ///
    /// Requires super user permission, otherwise will not be able to
    /// receive event from any process.
    ///
    /// Without this flag, receives events from same process only.
    pub const ALL_PROCESS: Type = 64;

    /// As event mask
    pub const fn msk(ev: Type) -> Type {
        1 << (ev - 1)
    }
}

/// The reason of the page migration event
pub mod kfd_migrate_triggers {
    pub type Type = u32;
    pub const PREFETCH: Type = 0; /* Prefetch to GPU VRAM or system memory */
    pub const PAGEFAULT_GPU: Type = 1; /* GPU page fault recover */
    pub const PAGEFAULT_CPU: Type = 2; /* CPU page fault recover */
    pub const TTM_EVICTION: Type = 3; /* TTM eviction */
}

/// The reason of user queue eviction event
pub mod kfd_queue_eviction_triggers {
    pub type Type = u32;
    pub const SVM: Type = 0; /* SVM buffer migration */
    pub const USERPTR: Type = 1; /* userptr movement */
    pub const TTM: Type = 2; /* TTM move buffer */
    pub const SUSPEND: Type = 3; /* GPU suspend */
    pub const CRIU_CHECKPOINT: Type = 4; /* CRIU checkpoint */
    pub const CRIU_RESTORE: Type = 5; /* CRIU restore */
}

/// The reason of unmap buffer from GPU event
pub mod kfd_svm_unmap_triggers {
    pub type Type = u32;
    pub const MMU_NOTIFY: Type = 0; /* MMU notifier CPU buffer movement */
    pub const MMU_NOTIFY_MIGRATE: Type = 1; /* MMU notifier page migration */
    pub const UNMAP_FROM_CPU: Type = 2; /* Unmap to free the buffer */
}

pub const KFD_SMI_EVENT_MSG_SIZE: usize = 96;

#[repr(C)]
pub struct KfdIoctlSmiEventsArgs {
    pub gpuid: GpuId,   /* to KFD */
    pub anon_fd: RawFd, /* from KFD */
}
assert_layout!(KfdIoctlSmiEventsArgs, size = 8, align = 4);

pub mod kfd_ioctl_svm_flag {
    pub type Type = u32;
    pub const HOST_ACCESS: Type = 0x00000001;
    pub const FLAG_COHERENT: Type = 0x00000002;
    pub const HIVE_LOCAL: Type = 0x00000004;
    pub const GPU_RO: Type = 0x00000008;
    pub const GPU_EXEC: Type = 0x00000010;
    pub const GPU_READ_MOSTLY: Type = 0x00000020;
    pub const GPU_ALWAYS_MAPPED: Type = 0x00000040;
    pub const EXT_COHERENT: Type = 0x00000080;
}

/// Must be used **only** for input to amdgpu
#[repr(u32)]
pub enum KfdIoctlSvmOp {
    Set = 0,
    Get = 1,
}

pub mod kfd_ioctl_svm_location {
    pub type Type = u32;
    pub const SYSMEM: Type = 0;
    pub const UNDEFINED: Type = 0xffffffff;
}

pub mod kfd_ioctl_svm_attr_type {
    pub type Type = u32;
    pub const PREFERRED_LOC: Type = 0;
    pub const PREFETCH_LOC: Type = 1;
    pub const ACCESS: Type = 2;
    pub const ACCESS_IN_PLACE: Type = 3;
    pub const NO_ACCESS: Type = 4;
    pub const SET_FLAGS: Type = 5;
    pub const CLR_FLAGS: Type = 6;
    pub const GRANULARITY: Type = 7;
}

#[repr(C)]
pub struct KfdIoctlSvmAttribute {
    pub type_: kfd_ioctl_svm_attr_type::Type,
    pub value: u32,
}
assert_layout!(KfdIoctlSvmAttribute, size = 8, align = 4);

#[repr(C)]
pub struct KfdIoctlSvmArgs {
    pub start_addr: u64,
    pub size: usize,
    pub op: KfdIoctlSvmOp,
    pub nattr: u32,
    pub attrs: [KfdIoctlSvmAttribute; 0],
}
assert_layout!(KfdIoctlSvmArgs, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetAvailableMemoryArgs {
    pub available: usize, /* from KFD */
    pub gpu_id: GpuId,    /* to KFD */
    pub _pad: u32,
}
assert_layout!(KfdIoctlGetAvailableMemoryArgs, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlExportDmabufArgs {
    pub handle: MemoryHandle, /* to KFD */
    pub flags: u32,           /* to KFD */
    pub dmabuf_fd: RawFd,     /* from KFD */
}
assert_layout!(KfdIoctlExportDmabufArgs, size = 16, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlRuntimeEnableArgs {
    pub r_debug: u64,
    pub mode_mask: u32,
    pub capabilities_mask: u32,
}
assert_layout!(KfdIoctlRuntimeEnableArgs, size = 16, align = 8);
