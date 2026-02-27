const AMDKFD_IOCTL_BASE: u32 = 'K' as u32;
pub type Errno = libc::c_int;

pub type QueueId = u32;
pub type GpuId = u32;
pub type MemoryHandle = u64;
pub type AllocMemFlag = u32;

macro_rules! define_amdkfd_ioctl {
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, RW) => {
        $(#[$meta])*
        pub unsafe fn $fn_name(fd: libc::c_int, args: &mut $args_ty) -> Result<(), Errno> {
            let ptr: *mut $args_ty = args;
            let res =
                unsafe { libc::ioctl(fd, libc::_IOWR::<$args_ty>(AMDKFD_IOCTL_BASE, $num), ptr) };
            if res != 0 {
                return Err( unsafe {* libc::__errno_location()});
            }
            Ok(())
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, R) => {
        $(#[$meta])*
        pub unsafe fn $fn_name(fd: libc::c_int, args: &mut $args_ty) -> Result<(), Errno> {
            let ptr: *mut $args_ty = args;
            let res =
                unsafe { libc::ioctl(fd, libc::_IOR::<$args_ty>(AMDKFD_IOCTL_BASE, $num), ptr) };
            if res != 0 {
                return Err( unsafe {* libc::__errno_location()});
            }
            Ok(())
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, W) => {
        $(#[$meta])*
        pub unsafe fn $fn_name(fd: libc::c_int, args: &mut $args_ty) -> Result<(), Errno> {
            let ptr: *mut $args_ty = args;
            let res =
                unsafe { libc::ioctl(fd, libc::_IOW::<$args_ty>(AMDKFD_IOCTL_BASE, $num), ptr) };
            if res != 0 {
                return Err( unsafe {* libc::__errno_location()});
            }
            Ok(())
        }
    };
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Version {
    pub major_version: u32,
    pub minor_version: u32,
}

define_amdkfd_ioctl!(amdkfd_ioctl_get_version, Version, 0x01, R);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlDestroyQueueArgs {
    pub queue_id: QueueId, /* to KFD */
    pub pad: u32,
}
define_amdkfd_ioctl!(
    amdkfd_ioctl_destroy_queue,
    KfdIoctlDestroyQueueArgs,
    0x3,
    RW
);
#[repr(u32)]
pub enum QueueType {
    Compute = 0,
    Sdma = 1,
    ComputeAql = 2,
    SdmaXgmi = 3,
    SdmaByEngId = 4,
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
pub const KFD_MAX_QUEUE_PERCENTAGE: u32 = 100;
pub const KFD_MAX_QUEUE_PRIORITY: u32 = 15;
pub const KFD_MIN_QUEUE_RING_SIZE: u32 = 1024;

define_amdkfd_ioctl!(amdkfd_ioctl_create_queue, KfdIoctlCreateQueueArgs, 0x02, RW);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlAcquireVmArgs {
    pub drm_fd: u32,
    pub gpu_id: GpuId,
}
define_amdkfd_ioctl!(
    /// Returns:
    /// * EINVAL - if drm_fd is not a valid fd
    /// or gpu_id not found
    /// or some nested function returned it
    /// * EBUSY - if the kfd device already has an associated drm_file and it's different
    /// from the one provided
    /// * any - there might be some error deep in the callstack
    amdkfd_ioctl_acquire_vm, KfdIoctlAcquireVmArgs, 0x15, W);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
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

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetProcessAperturesArgs {
    pub process_apertures: [KfdProcessDeviceApertures; 7],
    pub num_of_nodes: u32,
    pub _pad: u32,
}

define_amdkfd_ioctl!(
    #[deprecated(
        since = "kfd 1.2",
        note = "Use `amdkfd_ioctl_get_process_apertures_new` instead"
    )]
    amdkfd_ioctl_get_process_apertures,
    KfdIoctlGetProcessAperturesArgs,
    0x6,
    R
);

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

define_amdkfd_ioctl!(
    /// It allows to query how many gpus are available, by passing 0 in num_of_nodes
    amdkfd_ioctl_get_process_apertures_new,
    KfdIoctlGetProcessAperturesNewArgs,
    0x14,
    RW
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetAvailableMemoryArgs {
    pub available: u64, /* from KFD */
    pub gpu_id: GpuId,  /* to KFD */
    pub _pad: u32,
}
define_amdkfd_ioctl!(
    /// Returns allocatable memory in bytes or EINVAL if couldn't find gpu_id
    amdkfd_ioctl_get_available_memory,
    KfdIoctlGetAvailableMemoryArgs,
    0x23,
    RW
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlRuntimeEnableArgs {
    r_debug: u64,
    mode_mask: u32,
    capabilities_mask: u32,
}
define_amdkfd_ioctl!(
    amdkfd_iotctl_runtime_enable,
    KfdIoctlRuntimeEnableArgs,
    0x25,
    RW
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlAllocMemoryOfGpuArgs {
    pub va_addr: u64,         /* to KFD */
    pub size: u64,            /* to KFD */
    pub handle: MemoryHandle, /* from KFD */
    pub mmap_offset: u64,     /* to KFD (userptr), from KFD (mmap offset) */
    pub gpu_id: u32,          /* to KFD */
    pub flags: u32,
}
// Allocation flags: memory types, pick only one
pub const KFD_IOC_ALLOC_MEM_FLAGS_VRAM: AllocMemFlag = 1 << 0;
pub const KFD_IOC_ALLOC_MEM_FLAGS_GTT: AllocMemFlag = 1 << 1;
pub const KFD_IOC_ALLOC_MEM_FLAGS_USERPTR: AllocMemFlag = 1 << 2;
pub const KFD_IOC_ALLOC_MEM_FLAGS_DOORBELL: AllocMemFlag = 1 << 3;
pub const KFD_IOC_ALLOC_MEM_FLAGS_MMIO_REMAP: AllocMemFlag = 1 << 4;

// Allocation flags: attributes/access options
pub const KFD_IOC_ALLOC_MEM_FLAGS_WRITABLE: AllocMemFlag = 1 << 31;
pub const KFD_IOC_ALLOC_MEM_FLAGS_EXECUTABLE: AllocMemFlag = 1 << 30;
pub const KFD_IOC_ALLOC_MEM_FLAGS_PUBLIC: AllocMemFlag = 1 << 29;
pub const KFD_IOC_ALLOC_MEM_FLAGS_NO_SUBSTITUTE: AllocMemFlag = 1 << 28;
pub const KFD_IOC_ALLOC_MEM_FLAGS_AQL_QUEUE_MEM: AllocMemFlag = 1 << 27;
pub const KFD_IOC_ALLOC_MEM_FLAGS_COHERENT: AllocMemFlag = 1 << 26;
pub const KFD_IOC_ALLOC_MEM_FLAGS_UNCACHED: AllocMemFlag = 1 << 25;
pub const KFD_IOC_ALLOC_MEM_FLAGS_EXT_COHERENT: AllocMemFlag = 1 << 24;
pub const KFD_IOC_ALLOC_MEM_FLAGS_CONTIGUOUS: AllocMemFlag = 1 << 23;
define_amdkfd_ioctl!(
    /// You need to first `acquire_vm`
    /// Returns:
    /// * ENODEV - compute vm is not initialized
    /// * EINVAL - if MMIO_REMAP set and size != PAGE_SIZE also the kernel must use PAGE_SIZE =
    /// 4096
    /// or if DOORBELL set and size !=
    amdkfd_ioctl_alloc_memory_of_gpu,
    KfdIoctlAllocMemoryOfGpuArgs,
    0x16,
    RW
);
#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlFreeMemoryOfGpuArgs {
    pub handle: MemoryHandle, /* to KFD */
}
define_amdkfd_ioctl!(
    amdkfd_ioctl_free_memory_of_gpu,
    KfdIoctlFreeMemoryOfGpuArgs,
    0x17,
    W
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlMapMemoryToGpuArgs {
    pub handle: MemoryHandle,      /* to KFD */
    pub device_ids_array_ptr: u64, /* to KFD */
    pub n_devices: u32,            /* to KFD */
    pub n_success: u32,            /* to/from KFD */
}
define_amdkfd_ioctl!(
    amdkfd_ioctl_map_memory_to_gpu,
    KfdIoctlMapMemoryToGpuArgs,
    0x18,
    RW
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlUnmapMemoryFromGpuArgs {
    pub handle: MemoryHandle,      /* to KFD */
    pub device_ids_array_ptr: u64, /* to KFD */
    pub n_devices: u32,            /* to KFD */
    pub n_success: u32,            /* to/from KFD */
}
define_amdkfd_ioctl!(
    amdkfd_ioctl_unmap_memory_from_gpu,
    KfdIoctlUnmapMemoryFromGpuArgs,
    0x19,
    RW
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlGetDmabufInfoArgs {
    pub size: u64,         /* from KFD */
    pub metadata_ptr: u64, /* to KFD */
    pub metadata_size: u32, /* to KFD (space allocated by user)
                            * from KFD (actual metadata size)
                            */
    pub gpu_id: u32,    /* from KFD */
    pub flags: u32,     /* from KFD (KFD_IOC_ALLOC_MEM_FLAGS) */
    pub dmabuf_fd: u32, /* to KFD */
}
define_amdkfd_ioctl!(
    amdkfd_ioctl_get_dmabuf_info,
    KfdIoctlGetDmabufInfoArgs,
    0x1C,
    RW
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlImportDmabufArgs {
    pub va_addr: u64,         /* to KFD */
    pub handle: MemoryHandle, /* from KFD */
    pub gpu_id: u32,          /* to KFD */
    pub dmabuf_fd: u32,       /* to KFD */
}
define_amdkfd_ioctl!(
    amdkfd_ioctl_import_dmabuf,
    KfdIoctlImportDmabufArgs,
    0x1D,
    RW
);

#[repr(C)]
#[derive(Debug, Default)]
pub struct KfdIoctlExportDmabufArgs {
    pub handle: MemoryHandle, /* to KFD */
    pub flags: u32,           /* to KFD */
    pub dmabuf_fd: u32,       /* from KFD */
}
define_amdkfd_ioctl!(
    amdkfd_ioctl_export_dmabuf,
    KfdIoctlExportDmabufArgs,
    0x24,
    RW
);
