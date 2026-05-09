use amdgpu_linux_api::drm::auth::WasMaster;
use amdgpu_linux_api::drm::ioctl;
use std::fs::OpenOptions;
use std::os::fd::FromRawFd;
use std::os::fd::{AsRawFd, OwnedFd};

fn main() {
    let file1 = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/dri/card1")
        .expect("Failed to open /dev/dri/card1");
    let fd1 = file1.as_raw_fd();

    // 1. Obtain master privileges on the primary fd
    unsafe { ioctl::drm::set_master(fd1).expect("Failed to set master") };

    // 2. Create an empty lease from the master fd
    let mut create_lease = ioctl::drm::CreateLease {
        object_ids: std::ptr::null(),
        object_count: 0,
        flags: 0, // Optionally libc::O_CLOEXEC
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
    let owned_lease = unsafe { OwnedFd::from_raw_fd(leased_fd) };
    //let _ = unsafe { ioctl::drm::drop_master(file1.as_raw_fd()) };

    // 3. Open the device again as a client
    let file2 = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/dri/card1")
        .expect("Failed to open /dev/dri/card1 again");
    let fd2 = file2.as_raw_fd();

    // 4. Get the magic for the new client
    let mut auth = ioctl::drm::Auth::default();
    unsafe { ioctl::drm::get_magic(fd2, &mut auth).expect("Failed to get magic") };
    println!("Got client magic: {}", auth.magic);

    // 5. Authenticate the client using the leased master fd
    unsafe {
        ioctl::drm::auth_magic(leased_fd, &mut auth).expect("Failed to auth magic on leased fd");
    }

    println!("Successfully authenticated client via the leased master!");
    drop(file1);
    drop(file2);
    drop(owned_lease);
}
