use crate::kfd::ioctl::GpuId;

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
