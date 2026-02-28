use amdgpu_linux_api::kfd::ioctl;
use std::os::fd::AsRawFd;

fn main() {
    let file = std::fs::File::open("/dev/kfd").unwrap();

    let mut version = ioctl::KfdVersion::default();
    let _ = unsafe { ioctl::amdkfd_ioctl_get_version(file.as_raw_fd(), &mut version) };

    println!("{version:?}");
}
