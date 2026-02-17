use std::{
    fs::File,
    io::{self},
    os::fd::{AsFd, AsRawFd},
};

use amdkfd::*;

#[test]
fn opening_kfd() {
    let kfd = amdkfd::Kfd::open().expect("Please run this on linux with a modern AMD gpu");
    let version = kfd.version();
    println!("{version:?}");
    drop(kfd);
}

// #[test]
// fn creating_small_sdma_queue() {
//     let mut ring = [0u32; 1024];
//     let mut rptr = 0;
//     let mut wptr = 0;
//     let kfd = Kfd::open().expect("Hello");
//     let mut out = KfdIoctlCreateQueueArgs {
//         gpu_id: 34961,
//         queue_type: QueueType::Sdma as u32,
//         ring_base_address: &raw const ring as u64,
//         // It's not really const
//         read_pointer_address: &raw mut rptr as u64,
//         write_pointer_address: &raw const wptr as u64,
//         ring_size: 1024,
//         ..Default::default()
//     };
//     let fd = kfd.as_fd();
//     let res = unsafe { libc::ioctl(fd.as_raw_fd(), AMDKFD_IOC_CREATE_QUEUE, &raw mut out) };
//     println!("{res}, errno: {}", std::io::Error::last_os_error());
//     let mut _line = String::new();
//     let _ = stdin().read_line(&mut _line);
// }

/// Acquire vm modifies state for the whole process which makes it hard to test
/// Tests the ioctl is invoked, what it returns,
/// how it reacts to multiple calls
#[test]
fn acquire_vm() -> io::Result<()> {
    let kfd = Kfd::open().unwrap();
    let apertures = kfd.apertures()?;
    let fd = kfd.as_fd();
    let drm_file = File::open("/dev/dri/renderD128").unwrap();
    let drm_fd = drm_file.as_fd();
    let gpu_id = apertures
        .first()
        .expect("at least one kfd device required")
        .gpu_id;
    // println!("before first call: {apertures:#?}");
    // let mut _line = String::new();
    // let _ = stdin().read_line(&mut _line);
    let sth = unsafe {
        amdkfd_ioctl_acquire_vm(
            fd.as_raw_fd(),
            &mut KfdIoctlAcquireVmArgs {
                drm_fd: drm_fd.as_raw_fd() as u32, // valid fd is positive
                gpu_id,
            },
        )
    };
    //println!("first time: {sth:?}");
    // let mut _line = String::new();
    // let _ = stdin().read_line(&mut _line);
    assert!(sth.is_ok());
    let sth = unsafe {
        amdkfd_ioctl_acquire_vm(
            fd.as_raw_fd(),
            &mut KfdIoctlAcquireVmArgs {
                drm_fd: drm_fd.as_raw_fd() as u32, // valid fd is positive
                gpu_id,
            },
        )
    };
    assert!(sth.is_ok());
    // println!("second time: {sth:?}");
    // let mut _line = String::new();
    // let _ = stdin().read_line(&mut _line);
    // println!("releasing resources");
    let drm_file = File::open("/dev/dri/renderD128").unwrap();
    let drm_fd = drm_file.as_fd();
    let sth = unsafe {
        amdkfd_ioctl_acquire_vm(
            fd.as_raw_fd(),
            &mut KfdIoctlAcquireVmArgs {
                drm_fd: drm_fd.as_raw_fd() as u32, // valid fd is positive
                gpu_id,
            },
        )
    };
    assert!(sth.is_err());
    drop(drm_file);
    drop(kfd);
    Ok(())
}
