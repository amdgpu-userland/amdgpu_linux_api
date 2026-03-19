use amdgpu_linux_api::{
    SIGNAL_PAGES_SIZE,
    drm::AmdgpuDrmRender3_64,
    kfd::{
        AcquireVm, Kfd1_18,
        apertures::AperturesNew,
        ioctl::{
            AllocMemoryOfGpuArgs, CreateEventArgs, FreeMemoryOfGpuArgs, alloc_domain, alloc_flags,
            alloc_memory_of_gpu, create_event, event_type, free_memory_of_gpu,
        },
    },
};
use std::os::fd::{AsFd, AsRawFd};

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let devs = kfd.all_apertures().unwrap();
    let dev = &devs[0];

    let amdgpu_linux_api::kfd::AcquireVmResult::Ok(kfd) = kfd.acquire_vm(dev, &drm) else {
        panic!("Acquire VM")
    };
    let fd = kfd.as_fd();

    let mut args = AllocMemoryOfGpuArgs {
        va_addr: dev.gpuvm_base,
        size: SIGNAL_PAGES_SIZE!(),
        handle: 0,
        mmap_offset: 0,
        gpu_id: dev.gpu_id,
        flags: alloc_domain::GTT
            | alloc_flags::WRITABLE
            | alloc_flags::PUBLIC
            | alloc_flags::UNCACHED,
    };
    let res = unsafe { alloc_memory_of_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    let kfd_mem_handle = args.handle;

    let mut args = CreateEventArgs {
        event_page_offset: (u64::from(dev.gpu_id) << 32) | kfd_mem_handle,
        event_trigger_data: 0,
        event_type: event_type::SIGNAL,
        auto_reset: 0,
        node_id: 0,
        event_id: 0,
        event_slot_index: 0,
    };
    let res = unsafe { create_event(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    assert!(args.event_id != 0);

    let mut args = FreeMemoryOfGpuArgs {
        handle: kfd_mem_handle,
    };
    let res = unsafe { free_memory_of_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_err());
}
