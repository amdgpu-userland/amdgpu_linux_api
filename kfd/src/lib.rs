use std::{
    io,
    mem::MaybeUninit,
    os::fd::{AsFd, AsRawFd, BorrowedFd},
};

pub mod ring_buffer;

pub const KFD_FILE_PATH: &str = "/dev/kfd";

pub type GpuId = u32;
pub type QueueId = u32;

pub struct Kfd {
    file: std::fs::File,
    /// We can cache the result, because the module cannot be unloaded
    /// while the kfd file is still in use
    version: Version,
}

pub struct AmdgpuDrm {
    file: std::fs::File,
}

impl Kfd {
    pub fn open() -> std::io::Result<Self> {
        let file = std::fs::File::open(KFD_FILE_PATH)?;

        // Let's do version ioctl to check if we got the right file
        let mut version = Version::default();
        if let Err(e) = unsafe { amdkfd_ioctl_get_version(file.as_raw_fd(), &mut version) } {
            panic!("get_version {e}");
        }

        Ok(Self { file, version })
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    // fn apertures(&self) -> io::Result<Vec<KfdProcessDeviceApertures>> {
    //     let mut args = KfdIoctlGetProcessAperturesNewArgs {
    //         num_of_nodes: 0,
    //         ..Default::default()
    //     };
    //     unsafe { amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args) }?;
    //     let mut vec: Vec<KfdProcessDeviceApertures> =
    //         vec![KfdProcessDeviceApertures::default(); args.num_of_nodes as usize];
    //     args.kfd_process_device_apertures_ptr = vec.as_mut_ptr();
    //     unsafe { amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args) }?;
    //
    //     Ok(vec)
    // }

    /// Remember that devices can be removed after this call, so
    /// return values might not be valid after some time
    pub fn apertures(&self) -> io::Result<Vec<KfdProcessDeviceApertures>> {
        let mut args = KfdIoctlGetProcessAperturesNewArgs {
            num_of_nodes: 0,
            ..Default::default()
        };
        // Gets num_of_nodes
        unsafe { amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args) }?;

        let mut vec: Vec<MaybeUninit<KfdProcessDeviceApertures>> =
            Vec::with_capacity(args.num_of_nodes as usize);
        unsafe { vec.set_len(args.num_of_nodes as usize) };

        args.kfd_process_device_apertures_ptr = vec.as_mut_ptr() as *mut KfdProcessDeviceApertures;
        unsafe { amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args) }?;

        // SAFETY: the ioctl has initialized all elements
        Ok(unsafe {
            std::mem::transmute::<
                Vec<MaybeUninit<KfdProcessDeviceApertures>>,
                Vec<KfdProcessDeviceApertures>,
            >(vec)
        })
    }

    /// Please call with relatively small array.
    /// There should be at least 1 gpu (len = 1)
    /// Old kfd limit was 7
    ///
    /// Remember that devices can be removed after this call, so
    /// return values might not be valid after some time
    pub fn apertures_with_known_size(
        &self,
        buf: &mut [KfdProcessDeviceApertures],
    ) -> io::Result<usize> {
        let Ok(len) = u32::try_from(buf.len()) else {
            panic!("Why do you want over u32::MAX gpus?")
        };
        let mut args = KfdIoctlGetProcessAperturesNewArgs {
            kfd_process_device_apertures_ptr: buf.as_mut_ptr(),
            num_of_nodes: len,
            _pad: 0,
        };
        unsafe { amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args) }?;
        Ok(args.num_of_nodes as usize)
    }
}

impl Kfd {
    pub fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

/// Kfd devices can be removed at runtime
/// therefore all methods take ownership of the data
/// and will not return it back on certain errors
pub struct KfdNode<'kfd> {
    gpu_id: GpuId,
    kfd: BorrowedFd<'kfd>,
}

impl<'kfd> KfdNode<'kfd> {
    pub fn from_aperture(kfd: &'kfd Kfd, aperture: &KfdProcessDeviceApertures) -> Self {
        Self {
            gpu_id: aperture.gpu_id,
            kfd: kfd.as_fd(),
        }
    }

    pub unsafe fn from_raw(kfd: BorrowedFd<'kfd>, gpu_id: GpuId) -> Self {
        Self { gpu_id, kfd }
    }
}

pub enum AvailableMemoryError {
    NodeNotFound,
    Unexpected(std::io::Error),
}

pub enum AcquireVmError {
    NodeNotFound,
    WrongDrmFile,
    Unexpected(std::io::Error),
}

impl KfdNode<'_> {
    pub fn create_queue(self) -> (Self,) {
        (self,)
    }

    /// Get how many bytes you should be able to allocate with [`alloc_memory_of_gpu`].
    pub fn available_memory(self) -> Result<(Self, u64), AvailableMemoryError> {
        let mut args = KfdIoctlGetAvailableMemoryArgs {
            gpu_id: self.gpu_id,
            ..Default::default()
        };
        if let Err(e) =
            unsafe { amdkfd_ioctl_get_available_memory(self.kfd.as_raw_fd(), &mut args) }
        {
            return match e.kind() {
                io::ErrorKind::InvalidInput => Err(AvailableMemoryError::NodeNotFound),
                _ => Err(AvailableMemoryError::Unexpected(e)),
            };
        }
        Ok((self, args.available))
    }

    pub fn clock_counters(self) -> (Self,) {
        (self,)
    }

    pub fn set_scratch_backing_va(self) -> (Self,) {
        (self,)
    }

    pub fn tile_config(self) -> (Self,) {
        (self,)
    }

    /// Signature not finished yet. There is an ownership transfer for drm_file internal VM.
    pub unsafe fn acquire_vm<'drm>(
        self,
        drm_fd: &'drm mut AmdgpuDrm,
    ) -> Result<(Self,), AcquireVmError> {
        let mut args = KfdIoctlAcquireVmArgs {
            // SAFETY: AmdgpuDrm has a successfully opened file descriptor
            drm_fd: drm_fd.file.as_raw_fd() as u32,
            gpu_id: self.gpu_id,
        };
        if let Err(e) = unsafe { amdkfd_ioctl_acquire_vm(self.kfd.as_raw_fd(), &mut args) } {
            let er = match e.kind() {
                // We can be sure that it's not because of drm_fd
                io::ErrorKind::InvalidInput => AcquireVmError::NodeNotFound,
                io::ErrorKind::ResourceBusy => AcquireVmError::WrongDrmFile,
                _ => AcquireVmError::Unexpected(e),
            };
            return Err(er);
        }
        Ok((self,))
    }

    pub fn alloc_memory_of_gpu(self) -> (Self,) {
        (self,)
    }

    pub fn import_dmabuf(self) -> (Self,) {
        (self,)
    }

    #[deprecated(
        since = "gfx9",
        note = "It's still available on newer gpus but does nothing"
    )]
    pub fn set_memory_policy(self, policy: ()) -> (Self,) {
        let _ = policy;
        (self,)
    }
}

/// Deprecated debugging api
impl KfdNode<'_> {
    pub fn debug_register(self) -> (Self,) {
        (self,)
    }

    pub fn debug_unregister(self) -> (Self,) {
        (self,)
    }

    pub fn debug_address_watch(self) -> (Self,) {
        (self,)
    }

    pub fn debug_wave_control(self) -> (Self,) {
        (self,)
    }
}

/// New debugging api
impl KfdNode<'_> {}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Version {
    pub major_version: u32,
    pub minor_version: u32,
}

const AMDKFD_IOCTL_BASE: u32 = 'K' as u32;

macro_rules! define_amdkfd_ioctl {
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, RW) => {
        $(#[$meta])*
        pub unsafe fn $fn_name(fd: libc::c_int, args: &mut $args_ty) -> io::Result<()> {
            let ptr: *mut $args_ty = args;
            let res =
                unsafe { libc::ioctl(fd, libc::_IOWR::<$args_ty>(AMDKFD_IOCTL_BASE, $num), ptr) };
            if res != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, R) => {
        $(#[$meta])*
        pub unsafe fn $fn_name(fd: libc::c_int, args: &mut $args_ty) -> io::Result<()> {
            let ptr: *mut $args_ty = args;
            let res =
                unsafe { libc::ioctl(fd, libc::_IOR::<$args_ty>(AMDKFD_IOCTL_BASE, $num), ptr) };
            if res != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
    };
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, W) => {
        $(#[$meta])*
        pub unsafe fn $fn_name(fd: libc::c_int, args: &mut $args_ty) -> io::Result<()> {
            let ptr: *mut $args_ty = args;
            let res =
                unsafe { libc::ioctl(fd, libc::_IOW::<$args_ty>(AMDKFD_IOCTL_BASE, $num), ptr) };
            if res != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
    };
}

define_amdkfd_ioctl!(amdkfd_ioctl_get_version, Version, 0x01, R);

/// It holds an internal ref to gpu_id
pub struct Queue<'kfd> {
    kfd: BorrowedFd<'kfd>,
    id: QueueId,
}

impl Drop for Queue<'_> {
    fn drop(&mut self) {
        let res = unsafe {
            amdkfd_ioctl_destroy_queue(
                self.kfd.as_raw_fd(),
                &mut KfdIoctlDestroyQueueArgs {
                    queue_id: self.id,
                    ..Default::default()
                },
            )
        };
        debug_assert!(res.is_ok())
    }
}
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
