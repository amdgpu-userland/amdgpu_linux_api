use amdgpu_linux_api::kfd::ioctl;
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

fn main() {
    let file = std::fs::File::open("/dev/kfd").unwrap();

    let mut args = ioctl::KfdIoctlGetProcessAperturesNewArgs {
        num_of_nodes: 0,
        ..Default::default()
    };
    // Gets num_of_nodes
    let _ = unsafe { ioctl::amdkfd_ioctl_get_process_apertures_new(file.as_raw_fd(), &mut args) };

    let mut vec: Vec<MaybeUninit<ioctl::KfdProcessDeviceApertures>> =
        Vec::with_capacity(args.num_of_nodes as usize);
    unsafe { vec.set_len(args.num_of_nodes as usize) };

    args.kfd_process_device_apertures_ptr =
        vec.as_mut_ptr() as *mut ioctl::KfdProcessDeviceApertures;
    let res = unsafe { ioctl::amdkfd_ioctl_get_process_apertures_new(file.as_raw_fd(), &mut args) };

    assert!(res.is_ok());

    let vec = unsafe {
        std::mem::transmute::<
            Vec<MaybeUninit<ioctl::KfdProcessDeviceApertures>>,
            Vec<ioctl::KfdProcessDeviceApertures>,
        >(vec)
    };

    let drm_file = std::fs::File::open("/dev/dri/renderD128").unwrap();

    let _ = unsafe {
        ioctl::amdkfd_ioctl_acquire_vm(
            file.as_raw_fd(),
            &mut ioctl::KfdIoctlAcquireVmArgs {
                drm_fd: drm_file.as_raw_fd() as u32, // valid fd is positive
                gpu_id: vec[0].gpu_id,
            },
        )
    };

    let mut alloc_args = ioctl::KfdIoctlAllocMemoryOfGpuArgs {
        va_addr: vec[0].gpuvm_base,
        size: 5 * 4096,
        handle: 0,
        mmap_offset: 0,
        gpu_id: vec[0].gpu_id,
        flags: ioctl::KFD_IOC_ALLOC_MEM_FLAGS_VRAM,
    };
    let _ = unsafe { ioctl::amdkfd_ioctl_alloc_memory_of_gpu(file.as_raw_fd(), &mut alloc_args) };

    println!(
        "Allocated {} pages at {} VA in VRAM domain, handle: {}",
        alloc_args.size / 4096,
        alloc_args.va_addr,
        alloc_args.handle
    );
    let _ = std::io::stdin().read_line(&mut String::new());

    let device_ids = [vec[0].gpu_id];
    let _ = unsafe {
        ioctl::amdkfd_ioctl_map_memory_to_gpu(
            file.as_raw_fd(),
            &mut ioctl::KfdIoctlMapMemoryToGpuArgs {
                handle: alloc_args.handle,
                device_ids_array_ptr: &raw const device_ids as u64,
                n_devices: device_ids.len() as u32,
                n_success: 0,
            },
        )
    };
    println!("After mapping it, if you check gpu memory usage.");
    let _ = std::io::stdin().read_line(&mut String::new());

    let mut alloc_args2 = ioctl::KfdIoctlAllocMemoryOfGpuArgs {
        va_addr: vec[0].gpuvm_base + 3 * 4096,
        size: 7 * 4096,
        handle: 0,
        mmap_offset: 0,
        gpu_id: vec[0].gpu_id,
        flags: ioctl::KFD_IOC_ALLOC_MEM_FLAGS_VRAM,
    };
    let _ = unsafe { ioctl::amdkfd_ioctl_alloc_memory_of_gpu(file.as_raw_fd(), &mut alloc_args2) };

    println!(
        "Allocated {} pages at {} VA in VRAM domain, handle: {}, notice no conflict",
        alloc_args2.size / 4096,
        alloc_args2.va_addr,
        alloc_args2.handle
    );
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe {
        ioctl::amdkfd_ioctl_map_memory_to_gpu(
            file.as_raw_fd(),
            &mut ioctl::KfdIoctlMapMemoryToGpuArgs {
                handle: alloc_args2.handle,
                device_ids_array_ptr: &raw const device_ids as u64,
                n_devices: device_ids.len() as u32,
                n_success: 0,
            },
        )
    };
    println!("After seccond mapping we get conflict.");
    let _ = std::io::stdin().read_line(&mut String::new());
}
