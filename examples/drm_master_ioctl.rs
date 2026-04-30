use std::os::fd::{AsFd, AsRawFd};

use amdgpu_linux_api::drm::{AmdgpuDrmPrimary3_64, ioctl};

fn main() {
    let drm = AmdgpuDrmPrimary3_64::open(1).unwrap();

    println!("Before playing with master ioctls, hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { ioctl::drm::set_master(drm.as_fd().as_raw_fd()) };

    println!("After set_master, hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let new_fd = unsafe { libc::dup(drm.as_fd().as_raw_fd()) };
    println!("Dupplicated fd: {new_fd}. Hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { ioctl::drm::drop_master(drm.as_fd().as_raw_fd()) };

    println!("After drop_master, hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { ioctl::drm::set_master(drm.as_fd().as_raw_fd()) };

    println!("After set_master, hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { ioctl::drm::set_master(drm.as_fd().as_raw_fd()) };

    println!("After set_master, hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { ioctl::drm::drop_master(drm.as_fd().as_raw_fd()) };

    println!("After drop_master, hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { ioctl::drm::drop_master(drm.as_fd().as_raw_fd()) };

    println!("After drop_master, hit enter to continue");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { ioctl::drm::set_master(drm.as_fd().as_raw_fd()) };

    println!("After set_master, hit enter to exit");
    let _ = std::io::stdin().read_line(&mut String::new());
}
