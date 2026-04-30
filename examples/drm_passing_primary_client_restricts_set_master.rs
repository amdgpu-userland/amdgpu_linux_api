use std::os::fd::{AsFd, AsRawFd};
use std::process::exit;

use amdgpu_linux_api::drm::{AmdgpuDrmPrimary3_64, ioctl};

fn main() {
    // 1. Create a new primary DRM client by opening /dev/dri/card1
    let drm = AmdgpuDrmPrimary3_64::open(1).unwrap();
    let fd = drm.as_fd().as_raw_fd();
    println!("Parent: opened /dev/dri/card1, fd = {}", fd);

    // 2. Pass it to a new process via fork
    let pid = unsafe { libc::fork() };
    
    if pid < 0 {
        panic!("Fork failed");
    } else if pid == 0 {
        // Child process
        println!("Child: attempting to set_master on inherited fd {}", fd);
        
        // 3. The new process tries to invoke drm's set_master on it
        let res = unsafe { ioctl::drm::set_master(fd) };
        
        match res {
            Ok(_) => println!("Child: set_master succeeded!"),
            Err(e) => {
                let err = std::io::Error::last_os_error();
                println!("Child: set_master failed with code {} ({})", e, err);
            }
        }
        
        exit(0);
    } else {
        // Parent process
        let mut status = 0;
        unsafe { libc::waitpid(pid, &mut status, 0) };
        println!("Parent: child finished with status {}", status);
    }
}
