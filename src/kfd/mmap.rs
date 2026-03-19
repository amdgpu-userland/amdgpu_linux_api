use std::os::fd::AsRawFd;

use crate::kfd::{KfdFile, gpu_id::AsGpuId, ioctl::GpuId};

pub type Offset = i64;

pub const MMAP_TYPE_SHIFT: Offset = 62;
pub const MMAP_GPUID_SHIFT: Offset = 46;
pub const MMAP_GPUID_MASK: GpuId = (1 << 16) - 1;

pub const DOORBELL: Offset = 3 << MMAP_TYPE_SHIFT;
pub const EVENTS: Offset = 2 << MMAP_TYPE_SHIFT;
pub const MMIO_REMAP: Offset = 0 << MMAP_TYPE_SHIFT;

pub const fn gpu_id(x: GpuId) -> Offset {
    ((x & MMAP_GPUID_MASK) as Offset) << MMAP_GPUID_SHIFT
}

pub trait RemapMmio: KfdFile {
    fn mmio(&self, gpu_id: &impl AsGpuId) -> Mmio {
        let fd = self.as_fd().as_raw_fd();

        let id = gpu_id.gpu_id();

        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                GPU_PAGE_SIZE!(),
                libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                MMIO_REMAP | super::mmap::gpu_id(id),
            )
        };
        if ptr as i64 == -1 {
            panic!("Failed mapping mmio remap page");
        }

        Mmio(ptr.cast())
    }
}

pub struct Mmio(
    /// This pointer is valid even when kfd is closed because it point to a kmod global
    /// object
    *mut u32,
);

impl Mmio {
    pub fn flush_hdp_mem(&mut self) {
        const HDP_MEM_FLUSH_CNTL: isize = 0;
        unsafe {
            self.0
                .offset(HDP_MEM_FLUSH_CNTL)
                .write_volatile(1u32.to_le())
        };
    }

    pub fn flush_hdp_reg(&mut self) {
        const HDP_REG_FLUSH_CNTL: isize = 1;
        unsafe {
            self.0
                .offset(HDP_REG_FLUSH_CNTL)
                .write_volatile(1u32.to_le())
        };
    }
}

impl Drop for Mmio {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.0.cast(), GPU_PAGE_SIZE!()) };
    }
}
