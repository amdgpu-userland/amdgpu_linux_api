use std::os::fd::RawFd;
pub type QueueId = u32;

pub type Errno = libc::c_int;
pub type GpuId = u32;
pub type MemoryHandle = u64;
pub type AllocMemFlag = u32;

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdVersion {
    pub major: u32,
    pub minor: u32,
}
assert_layout!(KfdVersion, size = 8, align = 4);

#[repr(u32)]
pub enum QueueType {
    Compute = 0,
    Sdma = 1,
    ComputeAql = 2,
    SdmaXgmi = 3,
    SdmaByEngId = 4,
}
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
    pub queue_type: u32,       /* to KFD */
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
    pub va_addr: u64,         /* to KFD */
    pub size: u64,            /* to KFD */
    pub handle: MemoryHandle, /* from KFD */
    pub mmap_offset: u64,     /* to KFD (userptr), from KFD (mmap offset) */
    pub gpu_id: u32,          /* to KFD */
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
    pub va_addr: u64,         /* to KFD */
    pub handle: MemoryHandle, /* from KFD */
    pub gpu_id: GpuId,        /* to KFD */
    pub dmabuf_fd: RawFd,     /* to KFD */
}
assert_layout!(KfdIoctlImportDmabufArgs, size = 24, align = 8);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetAvailableMemoryArgs {
    pub available: u64, /* from KFD */
    pub gpu_id: GpuId,  /* to KFD */
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
