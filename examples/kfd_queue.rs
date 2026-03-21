#![feature(ptr_as_ref_unchecked)]
use amdgpu_linux_api::drm::*;
use amdgpu_linux_api::kfd::apertures::*;
use amdgpu_linux_api::kfd::ioctl::*;
use amdgpu_linux_api::kfd::mmap::RemapMmio;
use amdgpu_linux_api::kfd::*;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
use std::os::fd::RawFd;

fn assert_map_memory(fd: RawFd, handle: MemoryHandle, dev_ids: &[GpuId]) {
    let mut args = MapMemoryToGpuArgs {
        handle,
        device_ids_array_ptr: dev_ids.as_ptr(),
        n_devices: dev_ids.len() as u32,
        n_success: 0,
    };
    let res = unsafe { map_memory_to_gpu(fd, &mut args) };
    assert!(res.is_ok());
    assert!(args.n_success == args.n_devices);
}

macro_rules! rptr_idx {
    () => {
        0
    };
}
macro_rules! wptr_idx {
    () => {
        1
    };
}

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let devs = kfd.all_apertures().unwrap();
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let kfd = match kfd.acquire_vm(&devs[0], &drm) {
        AcquireVmResult::Ok(x) => x,
        _ => panic!(),
    };
    let gpu_id = devs[0].gpu_id;
    let fd = kfd.as_fd().as_raw_fd();
    let mut mmio = kfd.mmio(&devs[0]);

    let controlls_va = 0x10_000;
    let rptr_va = controlls_va + rptr_idx!() * 4;
    let wptr_va = controlls_va + wptr_idx!() * 4;
    let mut controlls_mem = [0u32; 1024];
    let controlls_size = size_of_val(&controlls_mem);
    let mut args = AllocMemoryOfGpuArgs {
        va_addr: controlls_va,
        size: controlls_size,
        handle: 0,
        mmap_offset: controlls_mem.as_ptr() as u64,
        gpu_id,
        flags: alloc_domain::USERPTR | alloc_flags::PUBLIC | alloc_flags::WRITABLE,
    };
    let res = unsafe { alloc_memory_of_gpu(fd, &mut args) };
    assert!(res.is_ok());

    let controlls_handle = args.handle;
    println!("Allocating Vram in kfd for rptr and wptr, handle: {controlls_handle}");

    let mut ring_mem = [0u32; 1024 * 4];
    ring_mem[8] = 0xCAF;
    let ring_buff_size = size_of_val(&ring_mem);
    let ring_buff_va = controlls_va + u64::try_from(controlls_size).unwrap();
    let mut args = AllocMemoryOfGpuArgs {
        va_addr: ring_buff_va,
        size: ring_buff_size,
        handle: 0,
        mmap_offset: ring_mem.as_ptr() as u64,
        gpu_id,
        flags: alloc_domain::USERPTR | alloc_flags::PUBLIC | alloc_flags::EXECUTABLE,
    };
    let res = unsafe { alloc_memory_of_gpu(fd, &mut args) };
    assert!(res.is_ok());

    let ring_buff_handle = args.handle;
    println!("Allocating Vram in kfd, handle: {ring_buff_handle}");

    let ctx_save_restore_size: u32 = 0x2_000;
    let ctx_save_restore_va = ring_buff_va + ring_buff_size as u64;
    let mut args = AllocMemoryOfGpuArgs {
        va_addr: ctx_save_restore_va,
        size: usize::try_from(ctx_save_restore_size).unwrap(),
        handle: 0,
        mmap_offset: 0,
        gpu_id,
        flags: alloc_domain::VRAM | alloc_flags::WRITABLE,
    };
    let res = unsafe { alloc_memory_of_gpu(fd, &mut args) };
    assert!(res.is_ok());
    let ctx_save_restore_handle = args.handle;

    let dev_ids = [gpu_id];
    assert_map_memory(fd, controlls_handle, &dev_ids);
    assert_map_memory(fd, ring_buff_handle, &dev_ids);
    assert_map_memory(fd, ctx_save_restore_handle, &dev_ids);

    println!("Before creating queue");
    controlls_mem[rptr_idx!()] = 0;
    controlls_mem[wptr_idx!()] = 64;
    mmio.flush_hdp_mem();

    let mut args = CreateQueueArgs {
        ring_base_address: ring_buff_va,
        write_pointer_address: wptr_va,
        read_pointer_address: rptr_va,
        doorbell_offset: 0,
        ring_size: ring_buff_size as u32,
        gpu_id,
        queue_type: queue_type::SDMA,
        queue_percentage: 0,
        queue_priority: 0xf,
        queue_id: 0,
        eop_buffer_address: 0,
        eop_buffer_size: 0,
        ctx_save_restore_address: ctx_save_restore_va,
        ctx_save_restore_size: ctx_save_restore_size,
        ctl_stack_size: 0x1_000,
        sdma_engine_id: 0,
        pad: 0,
    };
    let res = unsafe { create_queue(fd, &mut args) };
    assert!(res.is_ok());
    let doorbell_offset = args.doorbell_offset;
    let doorbell_idx = doorbell_offset & 0xFFFF;
    println!("Doorbells offset: {doorbell_offset:#x}");

    let doorbells = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            0x2_000,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            (doorbell_offset - doorbell_idx) as i64,
        )
    };
    assert!(doorbells as i64 != -1);
    let doorbell: *mut u64 = unsafe { doorbells.byte_offset(doorbell_idx as isize).cast() };

    println!("Doorbell: {}", unsafe { doorbell.read_volatile() });
    println!("Rptr: {}", controlls_mem[rptr_idx!()]);

    let _ = std::io::stdin().read_line(&mut String::new());
}
