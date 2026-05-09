use amdgpu_linux_api::drm::ioctl;
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;

fn main() {
    let file1 = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/dri/card1")
        .expect("Failed to open /dev/dri/card1");
    let fd1 = file1.as_raw_fd();

    unsafe { ioctl::drm::set_master(fd1).expect("Failed to set master") };

    let file2 = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/dri/card1")
        .expect("Failed to open /dev/dri/card1 again");
    let fd2 = file2.as_raw_fd();

    let mut auth = ioctl::drm::Auth::default();
    unsafe { ioctl::drm::get_magic(fd2, &mut auth).expect("Failed to get magic") };

    unsafe { ioctl::drm::auth_magic(fd1, &mut auth).expect("Failed to auth magic") };

    println!("Successfully authenticated client with magic {}", auth.magic);
}
