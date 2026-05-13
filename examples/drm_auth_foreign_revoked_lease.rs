use amdgpu_linux_api::drm::ioctl;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use uds::UnixStreamExt; // Provides send_fds and recv_fds

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let fd = OwnedFd::from(File::open("/dev/dri/card1").unwrap());
        if let Err(_) = unsafe { ioctl::drm::set_master(fd.as_raw_fd()) } {
            panic!(
                "This example requires primary client to have master status.
Try to run it in a getty tty"
            );
        }

        // --- PARENT PROCESS ---
        // 1. Create a pair of connected sockets
        let (parent_sock, child_sock) = UnixStream::pair()?;

        // 2. Spawn the child, passing the socket FD as inherited FD 3
        let mut child = std::process::Command::new(std::env::current_exe()?)
            .arg("child")
            .stdin(OwnedFd::from(child_sock))
            .spawn()?;

        // 3. Open a file we want to share
        let mut args = ioctl::drm::CreateLease {
            object_ids: std::ptr::null(),
            object_count: 0,
            flags: libc::O_CLOEXEC,
            lessee_id: 0,
            fd: 0,
        };
        unsafe { ioctl::drm::mode_create_lease(fd.as_raw_fd(), &mut args) }
            .expect("Creating lease");
        let file_to_share = args.fd;
        let mut args = ioctl::drm::RevokeLease {
            lessee_id: args.lessee_id,
        };
        // Revoking a lease doesn't change it's master status
        unsafe { ioctl::drm::mode_revoke_lease(fd.as_raw_fd(), &mut args) }
            .expect("Revoking lease before sending out");
        // But dropping master on lessor does
        unsafe { ioctl::drm::drop_master(fd.as_raw_fd()) }.expect("Dropping master on lessor");

        // 4. Send the FD via the Unix Socket
        parent_sock.send_fds(b"here is your fd", &[file_to_share.as_raw_fd()])?;

        child.wait()?;
        println!("[Parent] Child finished and successfully read and modified memory.");
    } else {
        // --- CHILD PROCESS ---
        // The child inherits the socket. For simplicity in this demo,
        // let's assume the socket is available (e.g., via stdin/stdout redirection).
        let child_sock = unsafe { UnixStream::from_raw_fd(0) };

        let mut buffer = [0u8; 15];
        let mut fds = [0i32; 1];

        // 1. Receive the FD from the parent
        let (bytes_read, fds_read) = child_sock.recv_fds(&mut buffer, &mut fds)?;

        if fds_read > 0 {
            let shared_file = unsafe { OwnedFd::from_raw_fd(fds[0]) };

            println!(
                "[Child] Received msg: '{}'",
                String::from_utf8_lossy(&buffer[..bytes_read])
            );
            println!("[Child] File descriptor: {}", shared_file.as_raw_fd());
            let mut buffer = Box::new([0; 1024]);
            let mut args = ioctl::drm::GetLease {
                count_objects: buffer.len().try_into().unwrap(),
                pad: 0,
                objects_ptr: buffer.as_mut_ptr(),
            };
            unsafe { ioctl::drm::mode_get_lease(shared_file.as_raw_fd(), &mut args) }.expect(
                "If the driver doesn't support modesetting this client could not be a lease",
            );
        }
        println!("[Child] Exiting");
    }

    Ok(())
}
