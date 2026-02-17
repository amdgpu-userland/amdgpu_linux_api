/// Pinned memory, that is written to by the userspace and read by the kernel.
/// The kernel always reads (shared ref) the range from rptr to wptr
/// The user can write (mut ref) from wptr to rptr wrapping around size.
/// The size is always a power of 2.
pub struct RingBuffer {
    pub memory: [u32; 1024],
    pub rptr: u32,
    pub wptr: u32,
}
