use amdgpu_linux_api::kfd::ioctl;
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

fn main() {
    let file = std::fs::File::open("/dev/kfd").unwrap();

    let mut args = ioctl::GetProcessAperturesNewArgs {
        num_of_nodes: 0,
        ..Default::default()
    };
    // Gets num_of_nodes
    let _ = unsafe { ioctl::get_process_apertures_new(file.as_raw_fd(), &mut args) };

    let mut vec: Vec<MaybeUninit<ioctl::ProcessDeviceApertures>> =
        Vec::with_capacity(args.num_of_nodes as usize);
    unsafe { vec.set_len(args.num_of_nodes as usize) };

    args.kfd_process_device_apertures_ptr = vec.as_mut_ptr() as *mut ioctl::ProcessDeviceApertures;
    let res = unsafe { ioctl::get_process_apertures_new(file.as_raw_fd(), &mut args) };

    assert!(res.is_ok());

    let vec = unsafe {
        std::mem::transmute::<
            Vec<MaybeUninit<ioctl::ProcessDeviceApertures>>,
            Vec<ioctl::ProcessDeviceApertures>,
        >(vec)
    };

    let drm_file = std::fs::File::open("/dev/dri/renderD128").unwrap();

    let res = unsafe {
        ioctl::acquire_vm(
            file.as_raw_fd(),
            &mut ioctl::AcquireVmArgs {
                drm_fd: drm_file.as_raw_fd(), // valid fd is positive
                gpu_id: vec[0].gpu_id,
            },
        )
    };

    assert!(matches!(res, Ok(_)));
    println!("Acquire VM accepts render node");

    drop(file);
    drop(drm_file);

    println!(
        "Please check opened file descriptors for pid: {}",
        std::process::id()
    );
    let _ = std::io::stdin().read_line(&mut String::new());

    let file = std::fs::File::open("/dev/kfd").unwrap();
    // gpu_id should not have changed if the device has not been removed
    let drm_file = std::fs::File::open("/dev/dri/card1").unwrap();
    let res = unsafe {
        ioctl::acquire_vm(
            file.as_raw_fd(),
            &mut ioctl::AcquireVmArgs {
                drm_fd: drm_file.as_raw_fd(), // valid fd is positive
                gpu_id: vec[0].gpu_id,
            },
        )
    };

    assert!(matches!(res, Ok(_)));
    println!("Acquire VM accepts primary node");

    let res = unsafe {
        ioctl::acquire_vm(
            file.as_raw_fd(),
            &mut ioctl::AcquireVmArgs {
                drm_fd: drm_file.as_raw_fd(), // valid fd is positive
                gpu_id: vec[0].gpu_id,
            },
        )
    };
    assert!(matches!(res, Ok(_)));
    println!("Acquire VM accepts calling with the same drm_file again");

    let drm_file = std::fs::File::open("/dev/dri/renderD128").unwrap();
    let res = unsafe {
        ioctl::acquire_vm(
            file.as_raw_fd(),
            &mut ioctl::AcquireVmArgs {
                drm_fd: drm_file.as_raw_fd(), // valid fd is positive
                gpu_id: vec[0].gpu_id,
            },
        )
    };

    assert!(matches!(res, Err(libc::EBUSY)));
    println!(
        "Acquire VM doesn't accept calling again with a different drm_file if vm was already acquired"
    );
}
