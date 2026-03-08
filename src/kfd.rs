use std::{
    ops::{Deref, DerefMut},
    os::fd::{AsFd, AsRawFd},
};

use crate::{drm::AmdgpuDrmFile, kfd::gpu_id::AsGpuId};

pub mod apertures;
pub mod ioctl;

pub const KFD_FILE_PATH: &str = "/dev/kfd";

/// Simple unifying abstraction for opened KFD chardev file.
///
/// Opening it creates a process global kfd_process.
/// Closing/dropping a kfd_file does not release any resources untill program exits.
///
/// Therefore all operations on kfd file don't require exclusive access.
///
/// Except for ptrace tracing process, all use of kfd file is restricted to the process, which
/// opened it.
///
pub unsafe trait KfdFile: AsFd {}

#[derive(Debug)]
pub enum OpenError {
    OpeningKfdFile(std::io::Error),
    GettingVersion(libc::c_int),
    VersionBelowRequested,
}

/// Convenience macro to define versioned Kfd structs
///
/// We can assume version is not going to change,
/// because the module cannot be unloaded
/// while the kfd file is still in use
macro_rules! define_kfd_version {
    ($name:ident, $major:literal, $minor:literal $(, $auto_trait:ty)*) => {
        #[doc = concat!("Kfd at least ", $major, ".", $minor)]
        pub struct $name {
            fd: std::os::fd::OwnedFd,
        }

        impl $name {
            pub fn open() -> Result<Self, OpenError> {
                let file = std::fs::File::options()
                    // needs write permissions for mmap
                    .write(true)
                    .read(true)
                    .open(KFD_FILE_PATH)
                    .map_err(OpenError::OpeningKfdFile)?;

                let mut version = ioctl::GetVersionArgs::default();
                unsafe { ioctl::get_version(std::os::fd::AsRawFd::as_raw_fd(&file), &mut version) }
                    .map_err(OpenError::GettingVersion)?;

                if version.major < $major && version.minor < $minor {
                    return Err(OpenError::VersionBelowRequested);
                }

                Ok(Self { fd: file.into() })
            }
        }

        impl AsFd for $name {
            fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
                self.fd.as_fd()
            }
        }

        unsafe impl KfdFile for $name {}

        /// It is send but only within the same linux process
        unsafe impl Send for $name {}

        unsafe impl Sync for $name {}

        $(
        impl $auto_trait for $name {}
        )*
    };
}

define_kfd_version!(Kfd1_1, 1, 1);
define_kfd_version!(Kfd1_2, 1, 2);
define_kfd_version!(Kfd1_3, 1, 3);
define_kfd_version!(Kfd1_4, 1, 4);
define_kfd_version!(Kfd1_5, 1, 5);
define_kfd_version!(Kfd1_6, 1, 6);
define_kfd_version!(Kfd1_7, 1, 7);
define_kfd_version!(Kfd1_8, 1, 8);
define_kfd_version!(Kfd1_9, 1, 9);
define_kfd_version!(Kfd1_10, 1, 10);
define_kfd_version!(Kfd1_11, 1, 11);
define_kfd_version!(Kfd1_12, 1, 12);
define_kfd_version!(Kfd1_13, 1, 13);
define_kfd_version!(Kfd1_14, 1, 14);
define_kfd_version!(Kfd1_15, 1, 15);
define_kfd_version!(Kfd1_16, 1, 16);
define_kfd_version!(Kfd1_17, 1, 17);
define_kfd_version!(
    Kfd1_18,
    1,
    18,
    apertures::Apertures,
    apertures::AperturesNew,
    AvailableMemory,
    AcquireVm
);

/// In KFD commands use gpu_id to signal which device they should impact or use
///
/// But gpu_id doesn't hold any lock on devices, which means they can be removed
/// or changed.
pub mod gpu_id {
    pub trait AsGpuId {
        /// This gpu_id is hopefully still valid
        fn gpu_id(&self) -> super::ioctl::GpuId;
    }

    /// A shortcut to provide a hopefully valid gpu_id by yourself
    pub struct ManualGpuId(super::ioctl::GpuId);
    impl ManualGpuId {
        pub fn from(gpu_id: super::ioctl::GpuId) -> Self {
            Self(gpu_id)
        }
    }
    impl AsGpuId for ManualGpuId {
        fn gpu_id(&self) -> super::ioctl::GpuId {
            self.0
        }
    }

    impl AsGpuId for super::ioctl::ProcessDeviceApertures {
        fn gpu_id(&self) -> super::ioctl::GpuId {
            self.gpu_id
        }
    }
}

#[derive(Debug)]
pub enum AvailableMemoryError {
    GpuNotFound,
    Unexpected(ioctl::Errno),
}
pub trait AvailableMemory: KfdFile {
    /// Get how many bytes you should be able to allocate in VRAM.
    ///
    /// The value can change dynamically as you might not be the only user of the device.
    ///
    /// Because in case of error it's unsafe to use the provided gpu_id
    /// it's passed by moving ownership
    fn available_memory<T: AsGpuId>(&self, gpu: T) -> Result<(T, usize), AvailableMemoryError> {
        let fd = self.as_fd().as_raw_fd();
        let gpu_id = gpu.gpu_id();

        let mut args = ioctl::GetAvailableMemoryArgs {
            gpu_id,
            ..Default::default()
        };
        if let Err(e) = unsafe { ioctl::get_available_memory(fd, &mut args) } {
            return match e {
                libc::EINVAL => Err(AvailableMemoryError::GpuNotFound),
                _ => Err(AvailableMemoryError::Unexpected(e)),
            };
        }
        Ok((gpu, args.available))
    }
}

#[derive(Debug)]
pub enum AcquireVmResult<T: AcquireVm> {
    /// Successfuly acquired vm from provided drm_file
    /// **or** it has been already acquired before this call.
    Ok(AcquiredVm<T>),
    /// You should probably check if gpu has been removed
    GpuNotFound(T),
    /// The kfd object already acquired a VM from a different drm_file
    ///
    /// Because droping kfd fd doesn't release resources, it very much possible a VM has already
    /// been acquired and it could have even been done by outside code if it used exec into us.
    MemoryAlreadyAcquiredWithDifferentDrmFile(AcquiredVm<T>),
    Unexpected(ioctl::Errno),
}

pub trait AcquireVm: KfdFile + Sized {
    /// This could take a &self instead, since all opened kfd files point to the same object in the
    /// kernel and closing them don't release resources / state.
    ///
    /// But to signal that after acquiring VM the kfd object will keep it for
    /// the rest of the duration of the program. The acquired VM cannot be chagned and closing the file doesn't matter.
    fn acquire_vm(
        self,
        gpu_id: &impl AsGpuId,
        drm_file: &impl AmdgpuDrmFile,
    ) -> AcquireVmResult<Self> {
        let drm_fd = drm_file.as_fd().as_raw_fd();
        let gpu_id = gpu_id.gpu_id();
        let kfd_fd = self.as_fd().as_raw_fd();

        let mut args = ioctl::AcquireVmArgs { drm_fd, gpu_id };
        if let Err(e) = unsafe { ioctl::acquire_vm(kfd_fd, &mut args) } {
            return match e {
                // We can be sure that it's not because of drm_fd
                libc::EINVAL => AcquireVmResult::GpuNotFound(self),
                libc::EBUSY => {
                    AcquireVmResult::MemoryAlreadyAcquiredWithDifferentDrmFile(AcquiredVm(self))
                }
                _ => AcquireVmResult::Unexpected(e),
            };
        }
        AcquireVmResult::Ok(AcquiredVm(self))
    }
}

/// A kfd file, where it had successfully acquired the VM
///
/// Remember droping this doesn't change that a VM has been acquired already for this process.
#[derive(Debug)]
pub struct AcquiredVm<T: KfdFile>(T);

impl<T: KfdFile> Deref for AcquiredVm<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: KfdFile> DerefMut for AcquiredVm<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// #[derive(Debug)]
// pub enum AcquireVmError {
//     NodeNotFound,
//     WrongDrmFile,
//     Unexpected(ioctl::Errno),
// }
//
// impl<'kfd> KfdNode<'kfd> {
//     pub fn create_queue(self) -> (Self,) {
//         (self,)
//     }
//
//     pub fn clock_counters(self) -> (Self,) {
//         (self,)
//     }
//
//     pub fn set_scratch_backing_va(self) -> (Self,) {
//         (self,)
//     }
//
//     pub fn tile_config(self) -> (Self,) {
//         (self,)
//     }
//
//     /// Signature not finished yet. There is an ownership transfer for drm_file internal VM.
//     pub unsafe fn acquire_vm(
//         self,
//         drm_fd: &mut AmdgpuDrm,
//     ) -> Result<(KfdNodeAcquiredVm<'kfd>,), AcquireVmError> {
//         let mut args = ioctl::KfdIoctlAcquireVmArgs {
//             // SAFETY: AmdgpuDrm has a successfully opened file descriptor
//             drm_fd: drm_fd.file.as_raw_fd(),
//             gpu_id: self.gpu_id,
//         };
//         if let Err(e) = unsafe { ioctl::amdkfd_ioctl_acquire_vm(self.kfd.as_raw_fd(), &mut args) } {
//             let er = match e {
//                 // We can be sure that it's not because of drm_fd
//                 libc::EINVAL => AcquireVmError::NodeNotFound,
//                 libc::EBUSY => AcquireVmError::WrongDrmFile,
//                 _ => AcquireVmError::Unexpected(e),
//             };
//             return Err(er);
//         }
//         Ok((KfdNodeAcquiredVm(self),))
//     }
//
//     pub fn alloc_memory_of_gpu(self) -> (Self,) {
//         // let mut args = KfdIoctlAllocMemoryOfGpuArgs {
//         //     va_addr: todo!(),
//         //     size: todo!(),
//         //     handle: 0,
//         //     mmap_offset: todo!(),
//         //     gpu_id: self.gpu_id,
//         //     flags: todo!(),
//         // };
//         // if let Err(e) = unsafe { amdkfd_ioctl_alloc_memory_of_gpu(self.kfd.as_raw_fd(), &mut args) }
//         // {
//         //     let _ = e;
//         //     todo!()
//         // }
//         (self,)
//     }
//
//     pub fn import_dmabuf(self) -> (Self,) {
//         (self,)
//     }
//
//     #[deprecated(
//         since = "gfx9",
//         note = "It's still available on newer gpus but does nothing"
//     )]
//     pub fn set_memory_policy(self, policy: ()) -> (Self,) {
//         let _ = policy;
//         (self,)
//     }
// }
//
// /// Deprecated debugging api
// impl KfdNode<'_> {
//     pub fn debug_register(self) -> (Self,) {
//         (self,)
//     }
//
//     pub fn debug_unregister(self) -> (Self,) {
//         (self,)
//     }
//
//     pub fn debug_address_watch(self) -> (Self,) {
//         (self,)
//     }
//
//     pub fn debug_wave_control(self) -> (Self,) {
//         (self,)
//     }
// }
//
// #[derive(Debug)]
// pub struct KfdNodeAcquiredVm<'kfd>(KfdNode<'kfd>);
//
// pub enum MemCachingPolicy {
//     Coherent,
//     Uncached,
//     ExtCoherent,
// }
//
// pub struct KfdVramMem {}
//
// impl<'kfd> KfdMemory<'kfd> for KfdVramMem {
//     fn handle(&self) -> ioctl::MemoryHandle {
//         todo!()
//     }
//
//     fn kfd(&self) -> BorrowedFd<'kfd> {
//         todo!()
//     }
// }
//
// impl<'kfd> KfdNodeAcquiredVm<'kfd> {
//     pub fn allocate_vram(
//         self,
//         gpu_virtual_address: u64,
//         size: usize,
//         flags: u32,
//     ) -> Result<(Self, ioctl::MemoryHandle), ()> {
//         let mut args = ioctl::KfdIoctlAllocMemoryOfGpuArgs {
//             va_addr: gpu_virtual_address,
//             size: u64::try_from(size).unwrap(),
//             handle: 0,
//             mmap_offset: 0,
//             gpu_id: self.0.gpu_id,
//             flags,
//         };
//         if let Err(e) =
//             unsafe { ioctl::amdkfd_ioctl_alloc_memory_of_gpu(self.0.kfd.as_raw_fd(), &mut args) }
//         {
//             match e {
//                 _ => todo!("allocating vram: {e}"),
//             }
//         }
//         Ok((self, args.handle))
//     }
//
//     pub fn allocate_userptr_backed_memory<'user_mem>(
//         self,
//         user_mem: &'user_mem mut [u8],
//         gpu_virtual_address: u64,
//     ) -> Result<(Self, UserptrMem<'kfd, 'user_mem>), ()> {
//         let mut args = ioctl::KfdIoctlAllocMemoryOfGpuArgs {
//             va_addr: gpu_virtual_address,
//             size: u64::try_from(user_mem.len()).expect("size to fit u64"),
//             handle: 0,
//             mmap_offset: user_mem.as_ptr() as u64,
//             gpu_id: self.0.gpu_id,
//             flags: ioctl::KFD_IOC_ALLOC_MEM_FLAGS_USERPTR,
//         };
//         if let Err(e) =
//             unsafe { ioctl::amdkfd_ioctl_alloc_memory_of_gpu(self.0.kfd.as_raw_fd(), &mut args) }
//         {
//             match e {
//                 _ => todo!("Allocation error: {e}"),
//             }
//         }
//         let kfd = self.0.clone();
//         Ok((
//             self,
//             UserptrMem {
//                 kfd_node: kfd,
//                 mem: user_mem,
//                 handle: args.handle,
//                 va: gpu_virtual_address,
//             },
//         ))
//     }
// }
//
// pub struct UserptrMem<'kfd, 'mem> {
//     pub kfd_node: KfdNode<'kfd>,
//     pub mem: &'mem mut [u8],
//     pub handle: ioctl::MemoryHandle,
//     pub va: u64,
// }
//
// pub trait KfdMemory<'kfd> {
//     fn handle(&self) -> ioctl::MemoryHandle;
//     fn kfd(&self) -> BorrowedFd<'kfd>;
// }
//
// pub trait DmabufExportableMemory<'kfd>: KfdMemory<'kfd> + Sized {
//     fn export_dmabuf(self, flags: u32) -> Result<(Self, OwnedFd), ()> {
//         let mut args = ioctl::KfdIoctlExportDmabufArgs {
//             handle: self.handle(),
//             flags,
//             dmabuf_fd: 0,
//         };
//         if let Err(e) =
//             unsafe { ioctl::amdkfd_ioctl_export_dmabuf(self.kfd().as_raw_fd(), &mut args) }
//         {
//             match e {
//                 _ => todo!("exporting dmabuf: {e}"),
//             }
//         }
//         Ok((self, unsafe {
//             OwnedFd::from_raw_fd(args.dmabuf_fd.try_into().unwrap())
//         }))
//     }
// }
//
// impl<'kfd, 'mem> UserptrMem<'kfd, 'mem> {
//     pub fn map_memory(self, device_ids: &[u32]) -> Result<(Self,), ()> {
//         let n_devices = u32::try_from(device_ids.len()).map_err(|_| ())?;
//         let mut args = ioctl::KfdIoctlMapMemoryToGpuArgs {
//             handle: self.handle,
//             device_ids_array_ptr: device_ids.as_ptr() as u64,
//             n_devices: n_devices,
//             n_success: 0,
//         };
//         while args.n_success < args.n_devices {
//             if let Err(e) = unsafe {
//                 ioctl::amdkfd_ioctl_map_memory_to_gpu(self.kfd_node.kfd.as_raw_fd(), &mut args)
//             } {
//                 match e {
//                     _ => todo!("mapping memory to gpus: {e}"),
//                 }
//             }
//         }
//         Ok((self,))
//     }
// }
//
// /// New debugging api
// impl KfdNode<'_> {}
//
// /// It holds an internal ref to gpu_id
// pub struct Queue<'kfd> {
//     kfd: BorrowedFd<'kfd>,
//     id: ioctl::QueueId,
// }
//
// impl Drop for Queue<'_> {
//     fn drop(&mut self) {
//         let res = unsafe {
//             ioctl::amdkfd_ioctl_destroy_queue(
//                 self.kfd.as_raw_fd(),
//                 &mut ioctl::KfdIoctlDestroyQueueArgs {
//                     queue_id: self.id,
//                     ..Default::default()
//                 },
//             )
//         };
//         debug_assert!(res.is_ok())
//     }
// }
