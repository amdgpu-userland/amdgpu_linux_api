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

define_amdkfd_ioctl!(
    /// See which version the KFD driver is
    get_version, GetVersionArgs, 0x01, R);
define_amdkfd_ioctl!(
    /// Creates a command queue
    create_queue, CreateQueueArgs, 0x02, WR);
define_amdkfd_ioctl!(
    /// Removes a command queue
    destroy_queue, DestroyQueueArgs, 0x03, WR);
define_amdkfd_ioctl!(
    #[deprecated]
    /// Set caching policy for all memory types for gfx9
    ///
    /// Deprecated since gfx10.
    /// It has use for gfx9. For newer asics use per allocation flags.
    set_memory_policy,
    SetMemoryPolicyArgs,
    0x04,
    W
);
define_amdkfd_ioctl!(
    /// Get simple counters, useful for profiling
    get_clock_counters, GetClockCountersArgs, 0x05, WR);
define_amdkfd_ioctl!(
    #[deprecated]
    /// Get available devices and their virtual address space ranges
    ///
    /// Deprecated since kfd 1.2
    /// Use `get_process_apertures_new` instead
    get_process_apertures,
    GetProcessAperturesArgs,
    0x06,
    R
);
define_amdkfd_ioctl!(
    /// Change queue parameters, for example resizing the ring buffer
    update_queue, UpdateQueueArgs, 0x07, W);
define_amdkfd_ioctl!(
    /// Create an event listener, to be used signaled by the user or the gpu and waited on with
    /// [wait_events]
    ///
    /// There is a limit of 4096 events
    create_event, CreateEventArgs, 0x08, WR);
define_amdkfd_ioctl!(
    /// Remove an event listener
    destroy_event, DestroyEventArgs, 0x09, W);
define_amdkfd_ioctl!(
    /// CPU signals an event
    set_event, SetEventArgs, 0x0A, W);
define_amdkfd_ioctl!(
    /// Reset an event listener to unsignaled state
    reset_event, ResetEventArgs, 0x0B, W);
define_amdkfd_ioctl!(
    /// Wait for at least one or all events to be signaled
    wait_events, WaitEventsArgs, 0x0C, WR);
define_amdkfd_ioctl!(
    #[deprecated]
    /// Returns EPERM
    dbg_register,
    DbgRegisterArgs,
    0x0D,
    W
);
define_amdkfd_ioctl!(
    #[deprecated]
    /// Returns EPERM
    dbg_unregister,
    DbgUnregisterArgs,
    0x0E,
    W
);
define_amdkfd_ioctl!(
    #[deprecated]
    /// Returns EPERM
    dbg_address_watch,
    DbgAddressWatchArgs,
    0x0F,
    W
);
define_amdkfd_ioctl!(
    #[deprecated]
    /// Returns EPERM
    dbg_wave_control,
    DbgWaveControlArgs,
    0x10,
    W
);
define_amdkfd_ioctl!(
    /// Set the VA to be used as scratch memory base address
    ///
    /// Not that usefull since gfx9, because this address in no longer passed to shaders via gpu
    /// register mmSH_HIDDEN_PRIVATE_BASE_VMID.
    set_scratch_backing_va,
    SetScratchBackingVaArgs,
    0x11,
    WR
);
define_amdkfd_ioctl!(get_tile_config, GetTileConfigArgs, 0x12, WR);
define_amdkfd_ioctl!(
    /// Register a custom trap handler to be called from the primary handler
    set_trap_handler, SetTrapHandlerArgs, 0x13, W);
define_amdkfd_ioctl!(
    /// Modernized way to get all available devices.
    ///
    /// It allows to query how many gpus are available, by passing 0 in num_of_nodes
    get_process_apertures_new,
    GetProcessAperturesNewArgs,
    0x14,
    WR
);
define_amdkfd_ioctl!(
    /// Take a reference on amdgpu VM and prepares it for compute workload
    ///
    /// Returns:
    /// * EINVAL - if drm_fd is not a valid fd
    /// or gpu_id not found
    /// or some nested function returned it
    /// * EBUSY - if the kfd device already has an associated drm_file and it's different
    /// from the one provided
    /// * any - there might be some error deep in the callstack
    acquire_vm, AcquireVmArgs, 0x15, W);
define_amdkfd_ioctl!(
    /// Allocate memory to be used by devices
    ///
    /// You need to first `acquire_vm`
    ///
    /// Returns:
    /// * ENODEV - compute vm is not initialized
    /// * EINVAL - if MMIO_REMAP set and size != PAGE_SIZE also the kernel must use PAGE_SIZE =
    /// 4096
    /// or if DOORBELL set and size !=
    alloc_memory_of_gpu,
    AllocMemoryOfGpuArgs,
    0x16,
    WR
);
define_amdkfd_ioctl!(
    /// Free memory allocated with [alloc_memory_of_gpu]
    free_memory_of_gpu, FreeMemoryOfGpuArgs, 0x17, W);
define_amdkfd_ioctl!(
    /// Try to map provided memory to the same virtual address for all provided gpus
    map_memory_to_gpu, MapMemoryToGpuArgs, 0x18, WR);
define_amdkfd_ioctl!(
    /// Remove a virual address mapping from provided gpus
    unmap_memory_from_gpu, UnmapMemoryFromGpuArgs, 0x19, WR);
define_amdkfd_ioctl!(
    /// Set which CUs should be enabled/disabled for given queue
    set_cu_mask, SetCuMaskArgs, 0x1A, W);
define_amdkfd_ioctl!(
    /// Dump wave state
    get_queue_wave_state, GetQueueWaveStateArgs, 0x1B, WR);
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
    get_dmabuf_info,
    GetDmabufInfoArgs,
    0x1C,
    WR
);
define_amdkfd_ioctl!(
    /// Import memory allocated elsewere and exported as dmabuf
    import_dmabuf, ImportDmabufArgs, 0x1D, WR);
define_amdkfd_ioctl!(alloc_queue_gws, AllocQueueGwsArgs, 0x1E, WR);
define_amdkfd_ioctl!(
    /// Create a new file descriptor for system monitoring purposes
    ///
    /// Returns an OwnedFd for a special file
    /// which you can use to receive process scoped events
    /// or system scope events if you have enough permissions
    /// and say so in the filter mask.
    smi_events, SmiEventsArgs, 0x1F, WR);
define_amdkfd_ioctl!(
    /// Query and set attributes for ranges of virtual addresses
    ///
    /// EPERM if not supported
    svm, SvmArgs, 0x20, WR);
define_amdkfd_ioctl!(
    /// Query or try setting an xnack mode
    ///
    /// EPERM if not supported
    set_xnack_mode,
    SetXnackModeArgs,
    0x21,
    WR
);
define_amdkfd_ioctl!(
    /// Create checkpoints you can use to restore the kfd state from
    criu, CriuArgs, 0x22, WR);
define_amdkfd_ioctl!(
    /// Return allocatable memory in bytes
    ///
    /// or EINVAL if couldn't find gpu_id
    get_available_memory,
    GetAvailableMemoryArgs,
    0x23,
    WR
);
define_amdkfd_ioctl!(
    /// Create a new dmabuf file descriptor for given memory, so you can send it to another process to import
    export_dmabuf, ExportDmabufArgs, 0x24, WR);
define_amdkfd_ioctl!(
    /// Tell Kfd driver you are ready for being debugged
    runtime_enable, RuntimeEnableArgs, 0x25, WR);
define_amdkfd_ioctl!(
    /// Controlls for a debugger to trace other processes with
    dbg_trap, DbgTrapArgs, 0x26, WR);
