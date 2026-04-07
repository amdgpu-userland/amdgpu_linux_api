use amdgpu_linux_api::drm::*;
use amdgpu_linux_api::kfd::apertures::*;
use amdgpu_linux_api::kfd::ioctl::*;
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

    let controlls_va = 0x10_000;
    let mut controlls_mem = [0u64; 512];
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

    let (rptr, wptr) = controlls_mem.split_at_mut(4);
    let rptr = &mut rptr[0];
    let wptr = &mut wptr[0];
    *rptr = 0;
    *wptr = 64;
    let rptr: *mut u64 = rptr;
    let wptr: *mut u64 = wptr;

    let controlls_handle = args.handle;
    println!("Allocating controlls in kfd for rptr and wptr, handle: {controlls_handle}");

    let mut ring_mem = [0u32; 1024];
    ring_mem[0] = 0xCAF;
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
    let mut args = CreateQueueArgs {
        ring_base_address: ring_mem.as_ptr(),
        write_pointer_address: wptr.cast(),
        read_pointer_address: rptr.cast(),
        doorbell_offset: 0,
        ring_size: ring_buff_size as u32,
        gpu_id,
        queue_type: queue_type::SDMA,
        queue_percentage: 100,
        queue_priority: 0xf,
        queue_id: 0,
        eop_buffer_address: std::ptr::null_mut(),
        eop_buffer_size: 0,
        ctx_save_restore_address: std::ptr::null_mut(),
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
    println!("Rptr: {}", unsafe { rptr.read_volatile() });

    //let _ = std::io::stdin().read_line(&mut String::new());
}
