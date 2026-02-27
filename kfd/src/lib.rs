use std::{
    io,
    mem::MaybeUninit,
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
};

pub mod ioctl;
pub mod ring_buffer;

pub const KFD_FILE_PATH: &str = "/dev/kfd";

pub struct Kfd {
    file: std::fs::File,
    /// We can cache the result, because the module cannot be unloaded
    /// while the kfd file is still in use
    version: ioctl::Version,
}

pub struct AmdgpuDrm {
    pub file: std::fs::File,
}

#[derive(Debug)]
pub enum AperturesError {
    /// Internal kcalloc() failed
    NoMem,
    CopyingBackToUser,
    Unexpected(ioctl::Errno),
}

impl Kfd {
    pub fn open() -> std::io::Result<Self> {
        let file = std::fs::File::options()
            // needs write permissions for mmap
            .write(true)
            .read(true)
            .open(KFD_FILE_PATH)?;

        // Let's do version ioctl to check if we got the right file
        let mut version = ioctl::Version::default();
        if let Err(e) = unsafe { ioctl::amdkfd_ioctl_get_version(file.as_raw_fd(), &mut version) } {
            panic!("get_version {e}");
        }

        Ok(Self { file, version })
    }

    pub fn version(&self) -> &ioctl::Version {
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
    #[doc(alias = "apertures")]
    pub fn devices(&self) -> Result<Vec<ioctl::KfdProcessDeviceApertures>, AperturesError> {
        let mut args = ioctl::KfdIoctlGetProcessAperturesNewArgs {
            num_of_nodes: 0,
            ..Default::default()
        };
        // Gets num_of_nodes
        let res = unsafe {
            ioctl::amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args)
        };
        debug_assert!(
            res.is_ok(),
            "When num_of_nodes = 0, it shouldn't be able to throw"
        );

        let mut vec: Vec<MaybeUninit<ioctl::KfdProcessDeviceApertures>> =
            Vec::with_capacity(args.num_of_nodes as usize);
        unsafe { vec.set_len(args.num_of_nodes as usize) };

        args.kfd_process_device_apertures_ptr =
            vec.as_mut_ptr() as *mut ioctl::KfdProcessDeviceApertures;
        if let Err(e) = unsafe {
            ioctl::amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args)
        } {
            let er = match e {
                libc::ENOMEM => AperturesError::NoMem,
                libc::EFAULT => AperturesError::CopyingBackToUser,
                _ => AperturesError::Unexpected(e),
            };
            return Err(er);
        }

        // SAFETY: the ioctl has initialized all elements
        Ok(unsafe {
            std::mem::transmute::<
                Vec<MaybeUninit<ioctl::KfdProcessDeviceApertures>>,
                Vec<ioctl::KfdProcessDeviceApertures>,
            >(vec)
        })
    }
}
impl Kfd {
    /// Please call with relatively small array.
    /// There should be at least 1 gpu (len = 1)
    /// Old kfd limit was 7
    ///
    /// Remember that devices can be removed after this call, so
    /// return values might not be valid after some time
    #[deprecated(note = "use devices() instead")]
    pub fn apertures_with_known_size(
        &self,
        buf: &mut [ioctl::KfdProcessDeviceApertures],
    ) -> io::Result<usize> {
        let Ok(len) = u32::try_from(buf.len()) else {
            panic!("Why do you want over u32::MAX gpus?")
        };
        let mut args = ioctl::KfdIoctlGetProcessAperturesNewArgs {
            kfd_process_device_apertures_ptr: buf.as_mut_ptr(),
            num_of_nodes: len,
            _pad: 0,
        };
        unsafe { ioctl::amdkfd_ioctl_get_process_apertures_new(self.file.as_raw_fd(), &mut args) }
            .map_err(std::io::Error::from_raw_os_error)?;
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
#[derive(Debug, Clone)]
#[doc(alias = "Device")]
pub struct KfdNode<'kfd> {
    gpu_id: ioctl::GpuId,
    kfd: BorrowedFd<'kfd>,
}

impl<'kfd> KfdNode<'kfd> {
    pub fn from_aperture(kfd: &'kfd Kfd, aperture: &ioctl::KfdProcessDeviceApertures) -> Self {
        Self {
            gpu_id: aperture.gpu_id,
            kfd: kfd.as_fd(),
        }
    }

    pub unsafe fn from_raw(kfd: BorrowedFd<'kfd>, gpu_id: ioctl::GpuId) -> Self {
        Self { gpu_id, kfd }
    }
}

#[derive(Debug)]
pub enum AvailableMemoryError {
    NodeNotFound,
    Unexpected(ioctl::Errno),
}

#[derive(Debug)]
pub enum AcquireVmError {
    NodeNotFound,
    WrongDrmFile,
    Unexpected(ioctl::Errno),
}

impl<'kfd> KfdNode<'kfd> {
    pub fn create_queue(self) -> (Self,) {
        (self,)
    }

    /// Get how many bytes you should be able to allocate with [crate::KfdNode::alloc_memory_of_gpu].
    pub fn available_memory(self) -> Result<(Self, u64), AvailableMemoryError> {
        let mut args = ioctl::KfdIoctlGetAvailableMemoryArgs {
            gpu_id: self.gpu_id,
            ..Default::default()
        };
        if let Err(e) =
            unsafe { ioctl::amdkfd_ioctl_get_available_memory(self.kfd.as_raw_fd(), &mut args) }
        {
            return match e {
                libc::EINVAL => Err(AvailableMemoryError::NodeNotFound),
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
    pub unsafe fn acquire_vm(
        self,
        drm_fd: &mut AmdgpuDrm,
    ) -> Result<(KfdNodeAcquiredVm<'kfd>,), AcquireVmError> {
        let mut args = ioctl::KfdIoctlAcquireVmArgs {
            // SAFETY: AmdgpuDrm has a successfully opened file descriptor
            drm_fd: drm_fd.file.as_raw_fd() as u32,
            gpu_id: self.gpu_id,
        };
        if let Err(e) = unsafe { ioctl::amdkfd_ioctl_acquire_vm(self.kfd.as_raw_fd(), &mut args) } {
            let er = match e {
                // We can be sure that it's not because of drm_fd
                libc::EINVAL => AcquireVmError::NodeNotFound,
                libc::EBUSY => AcquireVmError::WrongDrmFile,
                _ => AcquireVmError::Unexpected(e),
            };
            return Err(er);
        }
        Ok((KfdNodeAcquiredVm(self),))
    }

    pub fn alloc_memory_of_gpu(self) -> (Self,) {
        // let mut args = KfdIoctlAllocMemoryOfGpuArgs {
        //     va_addr: todo!(),
        //     size: todo!(),
        //     handle: 0,
        //     mmap_offset: todo!(),
        //     gpu_id: self.gpu_id,
        //     flags: todo!(),
        // };
        // if let Err(e) = unsafe { amdkfd_ioctl_alloc_memory_of_gpu(self.kfd.as_raw_fd(), &mut args) }
        // {
        //     let _ = e;
        //     todo!()
        // }
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

#[derive(Debug)]
pub struct KfdNodeAcquiredVm<'kfd>(KfdNode<'kfd>);

pub enum MemCachingPolicy {
    Coherent,
    Uncached,
    ExtCoherent,
}

pub struct KfdVramMem {}

impl<'kfd> KfdMemory<'kfd> for KfdVramMem {
    fn handle(&self) -> ioctl::MemoryHandle {
        todo!()
    }

    fn kfd(&self) -> BorrowedFd<'kfd> {
        todo!()
    }
}

impl<'kfd> KfdNodeAcquiredVm<'kfd> {
    pub fn allocate_vram(
        self,
        gpu_virtual_address: u64,
        size: usize,
        flags: u32,
    ) -> Result<(Self, ioctl::MemoryHandle), ()> {
        let mut args = ioctl::KfdIoctlAllocMemoryOfGpuArgs {
            va_addr: gpu_virtual_address,
            size: u64::try_from(size).unwrap(),
            handle: 0,
            mmap_offset: 0,
            gpu_id: self.0.gpu_id,
            flags,
        };
        if let Err(e) =
            unsafe { ioctl::amdkfd_ioctl_alloc_memory_of_gpu(self.0.kfd.as_raw_fd(), &mut args) }
        {
            match e {
                _ => todo!("allocating vram: {e}"),
            }
        }
        Ok((self, args.handle))
    }

    pub fn allocate_userptr_backed_memory<'user_mem>(
        self,
        user_mem: &'user_mem mut [u8],
        gpu_virtual_address: u64,
    ) -> Result<(Self, UserptrMem<'kfd, 'user_mem>), ()> {
        let mut args = ioctl::KfdIoctlAllocMemoryOfGpuArgs {
            va_addr: gpu_virtual_address,
            size: u64::try_from(user_mem.len()).expect("size to fit u64"),
            handle: 0,
            mmap_offset: user_mem.as_ptr() as u64,
            gpu_id: self.0.gpu_id,
            flags: ioctl::KFD_IOC_ALLOC_MEM_FLAGS_USERPTR,
        };
        if let Err(e) =
            unsafe { ioctl::amdkfd_ioctl_alloc_memory_of_gpu(self.0.kfd.as_raw_fd(), &mut args) }
        {
            match e {
                _ => todo!("Allocation error: {e}"),
            }
        }
        let kfd = self.0.clone();
        Ok((
            self,
            UserptrMem {
                kfd_node: kfd,
                mem: user_mem,
                handle: args.handle,
                va: gpu_virtual_address,
            },
        ))
    }
}

pub struct UserptrMem<'kfd, 'mem> {
    pub kfd_node: KfdNode<'kfd>,
    pub mem: &'mem mut [u8],
    pub handle: ioctl::MemoryHandle,
    pub va: u64,
}

pub trait KfdMemory<'kfd> {
    fn handle(&self) -> ioctl::MemoryHandle;
    fn kfd(&self) -> BorrowedFd<'kfd>;
}

pub trait DmabufExportableMemory<'kfd>: KfdMemory<'kfd> + Sized {
    fn export_dmabuf(self, flags: u32) -> Result<(Self, OwnedFd), ()> {
        let mut args = ioctl::KfdIoctlExportDmabufArgs {
            handle: self.handle(),
            flags,
            dmabuf_fd: 0,
        };
        if let Err(e) =
            unsafe { ioctl::amdkfd_ioctl_export_dmabuf(self.kfd().as_raw_fd(), &mut args) }
        {
            match e {
                _ => todo!("exporting dmabuf: {e}"),
            }
        }
        Ok((self, unsafe {
            OwnedFd::from_raw_fd(args.dmabuf_fd.try_into().unwrap())
        }))
    }
}

impl<'kfd, 'mem> UserptrMem<'kfd, 'mem> {
    pub fn map_memory(self, device_ids: &[u32]) -> Result<(Self,), ()> {
        let n_devices = u32::try_from(device_ids.len()).map_err(|_| ())?;
        let mut args = ioctl::KfdIoctlMapMemoryToGpuArgs {
            handle: self.handle,
            device_ids_array_ptr: device_ids.as_ptr() as u64,
            n_devices: n_devices,
            n_success: 0,
        };
        while args.n_success < args.n_devices {
            if let Err(e) = unsafe {
                ioctl::amdkfd_ioctl_map_memory_to_gpu(self.kfd_node.kfd.as_raw_fd(), &mut args)
            } {
                match e {
                    _ => todo!("mapping memory to gpus: {e}"),
                }
            }
        }
        Ok((self,))
    }
}

/// New debugging api
impl KfdNode<'_> {}

/// It holds an internal ref to gpu_id
pub struct Queue<'kfd> {
    kfd: BorrowedFd<'kfd>,
    id: ioctl::QueueId,
}

impl Drop for Queue<'_> {
    fn drop(&mut self) {
        let res = unsafe {
            ioctl::amdkfd_ioctl_destroy_queue(
                self.kfd.as_raw_fd(),
                &mut ioctl::KfdIoctlDestroyQueueArgs {
                    queue_id: self.id,
                    ..Default::default()
                },
            )
        };
        debug_assert!(res.is_ok())
    }
}
