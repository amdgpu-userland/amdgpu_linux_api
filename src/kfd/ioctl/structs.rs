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

pub mod hsa_mem_exception {
    pub type FailureType = u32;
    pub const NO_RAS: FailureType = 0;
    pub const ECC_SRAM: FailureType = 1;
    pub const LINK_SYNCFLOOD: FailureType = 2;
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KfdHsaHwExceptionData {
    pub reset_type: u32,
    pub reset_cause: u32,
    pub memory_lost: u32,
    pub gpu_id: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KfdHsaSignalEventData {
    pub last_event_age: u64,
}

#[repr(C)]
pub union KfdEventDataUnion {
    /// Memory exception data (from KFD)
    pub memory_exception_data: KfdHsaMemoryExceptionData,
    /// HW exception data (from KFD)
    pub hw_exception_data: KfdHsaHwExceptionData,
    /// Signal event data (to and from KFD)
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
#[repr(C)]
pub struct KfdIoctlWaitEventsArgs {
    /// Pointer to kfd_event_data array (to KFD)
    pub events_ptr: u64,
    /// Number of events (to KFD)
    pub num_events: u32,
    /// Wait for all events (to KFD)
    pub wait_for_all: u32,
    /// Timeout (to KFD)
    pub timeout: u32,
    /// Wait result (from KFD)
    pub wait_result: u32,
}
assert_layout!(KfdIoctlWaitEventsArgs, size = 24, align = 8);

#[repr(C)]
pub struct KfdIoctlResetEventArgs {
    pub event_id: EventId,
    pub _pad: u32,
}
assert_layout!(KfdIoctlResetEventArgs, size = 8, align = 4);

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
