use amdgpu_linux_api::kfd::ioctl;
use std::{os::fd::AsRawFd, process::Command, time::Duration};

fn main() {
    let file = std::fs::File::open("/dev/kfd").unwrap();
    let _ = std::thread::scope(|scope| {
        let file_ref1 = &file;
        let file_ref2 = &file;

        scope.spawn(move || {
            let mut version = ioctl::GetVersionArgs::default();
            let _ = unsafe { ioctl::get_version(file_ref1.as_raw_fd(), &mut version) };
        });

        scope.spawn(move || {
            let mut version = ioctl::GetVersionArgs::default();
            let _ = unsafe { ioctl::get_version(file_ref2.as_raw_fd(), &mut version) };
        });
    });
    let mut version = ioctl::GetVersionArgs::default();
    let _ = unsafe { ioctl::get_version(file.as_raw_fd(), &mut version) };

    println!("Sharing a reference accross thread boundry (Sync) is fine in the same process");

    let res = std::thread::scope(|scope| {
        let _ = scope.spawn(|| {
            std::thread::sleep(Duration::from_millis(1));
        });

        let handle = scope.spawn(move || {
            let file = std::fs::File::open("/dev/kfd").unwrap();
            let mut version = ioctl::GetVersionArgs::default();
            let _ = unsafe { ioctl::get_version(file.as_raw_fd(), &mut version) };
            return file;
        });

        handle.join()
    })
    .unwrap();

    let file = res;

    let mut version = ioctl::GetVersionArgs::default();
    let _ = unsafe { ioctl::get_version(file.as_raw_fd(), &mut version) };

    println!("Getting a kfd from child thread is fine also (Send)");

    let mut child_proc = Command::new("file").stdin(file).arg("-").spawn().unwrap();
    let _ = child_proc.wait();

    println!(
        "But sending a kfd File Descriptor to another process is not allowed, check dmesg for
Using KFD FD in wrong process"
    );
}
