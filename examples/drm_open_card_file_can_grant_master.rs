use std::os::fd::AsRawFd;

fn main() {
    println!("Run this test when there is no drm_master for default device.");
    println!("For example use `chvt 3` or CTRL+ALT+3");

    let file = std::fs::File::open("/dev/dri/card1").unwrap();

    let _ = unsafe { amdgpu_linux_api::drm::ioctl::drm::set_master(file.as_raw_fd()) };

    println!("You can check if these are masters in debugfs clients file");
    let _ = std::io::stdin().read_line(&mut String::new());

    let file2 = std::fs::File::open("/dev/dri/card1").unwrap();
    let _ = unsafe { amdgpu_linux_api::drm::ioctl::drm::set_master(file2.as_raw_fd()) };

    println!("Now there should be two clients, the new one should not be master");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe { amdgpu_linux_api::drm::ioctl::drm::drop_master(file.as_raw_fd()) };

    println!("Now the first client should no longer be master and still be authenticated");
    let _ = std::io::stdin().read_line(&mut String::new());
}
