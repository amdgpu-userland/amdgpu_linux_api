use amdgpu_linux_api::drm::ioctl;
use std::fs::OpenOptions;
use std::io::{self, Read};
use std::os::fd::FromRawFd;
use std::os::fd::{AsRawFd, OwnedFd};

fn main() {
    let file1 = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/dri/card1")
        .expect("Failed to open /dev/dri/card1");
    let fd1 = file1.as_raw_fd();

    // 1. Verify it is master
    unsafe { ioctl::drm::set_master(fd1).expect("Failed to set master") };
    println!("Successfully set master on /dev/dri/card1.");

    // 2. Create an empty lease from the master fd
    let mut create_lease = ioctl::drm::CreateLease {
        object_ids: std::ptr::null(),
        object_count: 0,
        flags: 0,
        lessee_id: 0,
        fd: 0,
    };

    unsafe {
        ioctl::drm::mode_create_lease(fd1, &mut create_lease).expect("Failed to create lease");
    }
    let leased_fd = create_lease.fd;
    println!(
        "Created lease with lessee_id: {}, fd: {}",
        create_lease.lessee_id, leased_fd
    );
    let _owned_lease = unsafe { OwnedFd::from_raw_fd(leased_fd) };

    // 3. Drop the master from the first file
    unsafe { ioctl::drm::drop_master(fd1).expect("Failed to drop master") };
    //drop(file1); // also works
    println!("Dropped master from the primary file.");

    // 4. Ask user to verify in debugfs
    println!("=======================================================");
    println!("Please check debugfs to verify the client statuses.");
    println!("Run the following command in another terminal:");
    println!("    sudo cat /sys/kernel/debug/dri/1/clients");
    println!("(Adjust the node index '1' to match your card index if needed)");
    println!("The leased client should no longer hold master-like status.");
    println!("=======================================================");
    println!("Press Enter to exit and close the files...");

    let mut stdin = io::stdin();
    let mut buf = [0u8; 1];
    let _ = stdin.read(&mut buf);
}
