use std::os::fd::{AsFd, AsRawFd, BorrowedFd};

use crate::ring_buffer::RingBuffer;

pub mod ring_buffer;

pub const KFD_FILE_PATH: &str = "/dev/kfd";

pub struct Kfd {
    pub file: std::fs::File,
    /// We can cache the result, because the module cannot be unloaded
    /// while the kfd file is still in use
    version: Version,
}

impl Kfd {
    pub fn open() -> std::io::Result<Self> {
        let file = std::fs::File::open(KFD_FILE_PATH)?;

        // Let's do version ioctl to check if we got the right file
        let version = amdkfd_ioc_get_version(file.as_raw_fd());

        Ok(Self { file, version })
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn create_queue<'kfd>(&'kfd self, _builder: QueueBuilder) -> Result<Queue<'kfd>, ()> {
        let kfd = self.file.as_fd();
        todo!()

        // Ok(Queue {
        //     id: res.queue_id,
        //     kfd,
        // })
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Version {
    pub major_version: u32,
    pub minor_version: u32,
}

const AMDKFD_IOCTL_BASE: u32 = 'K' as u32;

const AMDKFD_IOC_GET_VERSION: libc::Ioctl = libc::_IOR::<Version>(AMDKFD_IOCTL_BASE, 0x1);
fn amdkfd_ioc_get_version(fd: libc::c_int) -> Version {
    let mut out = Version::default();
    let res = unsafe { libc::ioctl(fd, AMDKFD_IOC_GET_VERSION, &raw mut out) };
    if res != 0 {
        todo!("error getting version, is this file created by amdkfd driver?");
    }
    out
}

pub struct QueueBuilder {}

pub type QueueId = u32;
pub struct Queue<'kfd> {
    kfd: BorrowedFd<'kfd>,
    id: QueueId,
}

impl Drop for Queue<'_> {
    fn drop(&mut self) {
        let res = amdkfd_ioc_destroy_queue(self.kfd.as_raw_fd(), self.id);
        if res != 0 {
            todo!("destroying queue failed")
        }
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
    pub gpu_id: u32,           /* to KFD */
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

pub const AMDKFD_IOC_CREATE_QUEUE: libc::Ioctl =
    libc::_IOWR::<KfdIoctlCreateQueueArgs>(AMDKFD_IOCTL_BASE, 0x02);
fn amdkfd_ioc_create_queue(fd: libc::c_int, ring: &RingBuffer) -> KfdIoctlCreateQueueArgs {
    let mut out = KfdIoctlCreateQueueArgs {
        gpu_id: 1,
        queue_type: QueueType::Sdma as u32,
        ring_base_address: &raw const ring.memory as u64,
        // It's not really const
        read_pointer_address: &raw const ring.rptr as u64,
        write_pointer_address: &raw const ring.wptr as u64,
        ring_size: 1024,
        ..Default::default()
    };
    let res = unsafe { libc::ioctl(fd, AMDKFD_IOC_CREATE_QUEUE, &raw mut out) };
    if res != 0 {
        todo!("creating queue failed");
    }
    out
}

#[repr(C)]
#[derive(Debug, Default)]
struct KfdIoctlDestroyQueueArgs {
    pub queue_id: QueueId, /* to KFD */
    pub pad: u32,
}
const AMDKFD_IOC_DESTROY_QUEUE: libc::Ioctl =
    libc::_IOWR::<KfdIoctlDestroyQueueArgs>(AMDKFD_IOCTL_BASE, 0x03);
fn amdkfd_ioc_destroy_queue(fd: libc::c_int, queue_id: u32) -> i32 {
    let args = KfdIoctlDestroyQueueArgs { queue_id, pad: 0 };

    unsafe { libc::ioctl(fd, AMDKFD_IOC_DESTROY_QUEUE, &raw const args) }
}
