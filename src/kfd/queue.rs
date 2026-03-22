use std::marker::PhantomData;

/// A packet type which can be used in kfd queue
pub unsafe trait PktType {
    fn write_to_rb<'a>(&self, first: &'a mut [u32], second: &'a mut [u32]);
}

/// # SAFETY
/// Must be a compute packet in PM4
pub unsafe trait PM4Pkt: PktType {}

/// # SAFETY
/// Must be an aql compute packet
pub unsafe trait AqlPkt: PktType {}

/// # SAFETY
/// Must be a packet type acceptable by a particular SDMA engine
pub unsafe trait SdmaPkt: PktType {}

pub struct RingBuffer<T: PktType> {
    // base: *mut u32,
    // size_in_power_of_two: u8,
    // rptr: *const u32,
    // wptr: *mut u32,
    _marker: PhantomData<T>,
}

impl<T: PM4Pkt> RingBuffer<T> {
    pub fn compute() -> Self {
        todo!()
    }
}
impl<T: AqlPkt> RingBuffer<T> {
    pub fn compute_aql() -> Self {
        todo!()
    }
}
impl<T: SdmaPkt> RingBuffer<T> {
    pub fn sdma() -> Self {
        todo!()
    }
    pub fn sdma_by_xgi() -> Self {
        todo!()
    }
    pub fn sdma_by_engine_id() -> Self {
        todo!()
    }
}

pub struct CpuProducer<T: PktType> {
    _marker: PhantomData<T>,
}
/// An already active consumer
/// needed because cpu must use the doorbell when writing new packets to ring buff
pub struct GpuConsumer<T: PktType> {
    _marker: PhantomData<T>,
}

pub struct GpuProducer {}

impl<T: PktType> RingBuffer<T> {
    /// You can only write to it from a single cpu thread
    pub fn cpu_producer_gpu_consumer() -> Result<(CpuProducer<T>, GpuConsumer<T>), ()> {
        todo!()
    }
    /// You can only access it from a single gpu kernel
    pub fn gpu_producer_gpu_consumer() {}
}

pub fn write_producer<T>(_a: &mut [T], _b: &mut [T]) {
    todo!()
}
// impl RingBuffer {
//     pub fn write(&mut self, val: u32) {
//         let wptr = unsafe { *self.wptr };
//         let rptr = unsafe { *self.rptr };
//         let next_wptr = (wptr + 1) & (1 << self.size_in_power_of_two - 1);
//         if next_wptr == rptr {
//             panic!()
//         }
//         unsafe { self.base.add(*self.wptr as usize).write_volatile(val) };
//         unsafe { *self.wptr = next_wptr };
//     }
// }
