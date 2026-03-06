const AMDKFD_IOCTL_BASE: u32 = 'K' as u32;

mod structs;
pub use structs::*;

macro_rules! define_amdkfd_ioctl {
            ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, $ioctl_direction:tt) => {
                define_ioctl!(
                    $(#[$meta])*
                    $fn_name,
                    $args_ty,
                    $num,
                    AMDKFD_IOCTL_BASE,
                    $ioctl_direction
                );
            };
        }

define_amdkfd_ioctl!(amdkfd_ioctl_get_version, KfdVersion, 0x01, R);
define_amdkfd_ioctl!(amdkfd_ioctl_create_queue, KfdIoctlCreateQueueArgs, 0x02, WR);
define_amdkfd_ioctl!(
    amdkfd_ioctl_destroy_queue,
    KfdIoctlDestroyQueueArgs,
    0x03,
    WR
);
define_amdkfd_ioctl!(
    #[deprecated(
        since = "gfx10",
        note = "It has use for gfx9. For newer asics use per allocation flags."
    )]
    amdkfd_ioctl_set_memory_policy,
    KfdIoctlSetMemoryPolicyArgs,
    0x04,
    W
);
define_amdkfd_ioctl!(
    amdkfd_ioctl_get_clock_counters,
    KfdIoctlGetClockCountersArgs,
    0x05,
    WR
);
define_amdkfd_ioctl!(
    #[deprecated(
        since = "kfd 1.2",
        note = "Use `amdkfd_ioctl_get_process_apertures_new` instead"
    )]
    amdkfd_ioctl_get_process_apertures,
    KfdIoctlGetProcessAperturesArgs,
    0x06,
    R
);
define_amdkfd_ioctl!(amdkfd_ioctl_update_queue, KfdIoctlUpdateQueueArgs, 0x07, W);
define_amdkfd_ioctl!(amdkfd_ioctl_create_event, KfdIoctlCreateEventArgs, 0x08, WR);
define_amdkfd_ioctl!(
    amdkfd_ioctl_destroy_event,
    KfdIoctlDestroyEventArgs,
    0x09,
    W
);
define_amdkfd_ioctl!(amdkfd_ioctl_set_event, KfdIoctlSetEventArgs, 0x0A, W);
define_amdkfd_ioctl!(amdkfd_ioctl_reset_event, KfdIoctlResetEventArgs, 0x0B, W);
define_amdkfd_ioctl!(
    /// It allows to query how many gpus are available, by passing 0 in num_of_nodes
    amdkfd_ioctl_get_process_apertures_new,
    KfdIoctlGetProcessAperturesNewArgs,
    0x14,
    WR
);
define_amdkfd_ioctl!(
    /// Returns:
    /// * EINVAL - if drm_fd is not a valid fd
    /// or gpu_id not found
    /// or some nested function returned it
    /// * EBUSY - if the kfd device already has an associated drm_file and it's different
    /// from the one provided
    /// * any - there might be some error deep in the callstack
    amdkfd_ioctl_acquire_vm, KfdIoctlAcquireVmArgs, 0x15, W);
define_amdkfd_ioctl!(
    /// You need to first `acquire_vm`
    ///
    /// Returns:
    /// * ENODEV - compute vm is not initialized
    /// * EINVAL - if MMIO_REMAP set and size != PAGE_SIZE also the kernel must use PAGE_SIZE =
    /// 4096
    /// or if DOORBELL set and size !=
    amdkfd_ioctl_alloc_memory_of_gpu,
    KfdIoctlAllocMemoryOfGpuArgs,
    0x16,
    WR
);
define_amdkfd_ioctl!(
    amdkfd_ioctl_free_memory_of_gpu,
    KfdIoctlFreeMemoryOfGpuArgs,
    0x17,
    W
);
define_amdkfd_ioctl!(
    amdkfd_ioctl_map_memory_to_gpu,
    KfdIoctlMapMemoryToGpuArgs,
    0x18,
    WR
);
define_amdkfd_ioctl!(
    amdkfd_ioctl_unmap_memory_from_gpu,
    KfdIoctlUnmapMemoryFromGpuArgs,
    0x19,
    WR
);
define_amdkfd_ioctl!(
    /// Get underlying BO metadata and kfd metadata
    ///
    /// Metadata size must be large enough or EINVAL
    /// Metadata has no predefined layout, you'd have to check
    /// what the source application used.
    ///
    /// Mesa3D has it's own metadata layout.
    ///
    /// Not all flags will be returned.
    amdkfd_ioctl_get_dmabuf_info,
    KfdIoctlGetDmabufInfoArgs,
    0x1C,
    WR
);
define_amdkfd_ioctl!(
    amdkfd_ioctl_import_dmabuf,
    KfdIoctlImportDmabufArgs,
    0x1D,
    WR
);
define_amdkfd_ioctl!(
    /// Returns allocatable memory in bytes or EINVAL if couldn't find gpu_id
    amdkfd_ioctl_get_available_memory,
    KfdIoctlGetAvailableMemoryArgs,
    0x23,
    WR
);
define_amdkfd_ioctl!(
    amdkfd_ioctl_export_dmabuf,
    KfdIoctlExportDmabufArgs,
    0x24,
    WR
);
define_amdkfd_ioctl!(
    amdkfd_ioctl_runtime_enable,
    KfdIoctlRuntimeEnableArgs,
    0x25,
    WR
);
