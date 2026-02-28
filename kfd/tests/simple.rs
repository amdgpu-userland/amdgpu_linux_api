#[test]
fn doorbell() {
    let kfd = Kfd::open().unwrap();
    let apertures = kfd.devices().unwrap();
    let gpu = apertures.first().unwrap();
    let node = KfdNode::from_aperture(&kfd, gpu);
    let drm_file = std::fs::File::open("/dev/dri/renderD128").unwrap();
    let res = unsafe { node.acquire_vm(&mut AmdgpuDrm { file: drm_file }) };
    println!("acquire_vm: {res:?}");
    let mut _line = String::new();
    let _ = std::io::stdin().read_line(&mut _line);

    let mut args = KfdIoctlAllocMemoryOfGpuArgs {
        va_addr: 0,
        size: 8192,
        handle: 0,
        mmap_offset: 0,
        gpu_id: gpu.gpu_id,
        flags: KFD_IOC_ALLOC_MEM_FLAGS_DOORBELL | KFD_IOC_ALLOC_MEM_FLAGS_WRITABLE,
    };
    let res = unsafe { amdkfd_ioctl_alloc_memory_of_gpu(kfd.as_fd().as_raw_fd(), &mut args) };
    println!("allocation: {res:?}");

    let mut _line = String::new();
    let _ = std::io::stdin().read_line(&mut _line);

    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            8192,
            libc::PROT_WRITE,
            libc::MAP_SHARED,
            kfd.as_fd().as_raw_fd(),
            ((3 << 62) | ((gpu.gpu_id as u64) << 46)) as i64,
        )
    };
    if ptr.addr() == usize::MAX {
        let res = unsafe { *libc::__errno_location() };
        println!("Mapping error: {res}")
    } else {
        println!("Got a doorbells mapping {ptr:?}");
        let mut _line = String::new();
        let _ = std::io::stdin().read_line(&mut _line);

        unsafe {
            std::ptr::write_unaligned::<u64>(ptr as *mut u64, 12);
        }
        println!("Wrote a value into the mapping");
        let mut _line = String::new();
        let _ = std::io::stdin().read_line(&mut _line);

        unsafe {
            std::ptr::write_unaligned::<u64>(ptr.byte_offset(4) as *mut u64, u64::MAX);
        }
        println!("Wrote a max 64bit value into the mapping at offset 1 * size_of<u32>");
        let mut _line = String::new();
        let _ = std::io::stdin().read_line(&mut _line);
    }

    let res = unsafe {
        amdkfd_ioctl_free_memory_of_gpu(
            kfd.as_fd().as_raw_fd(),
            &mut KfdIoctlFreeMemoryOfGpuArgs {
                handle: args.handle,
            },
        )
    };
    println!("free: {res:?}");
    let mut _line = String::new();
    let _ = std::io::stdin().read_line(&mut _line);
}
