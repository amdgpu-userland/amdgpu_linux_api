use amdgpu_linux_api::drm;
use amdgpu_linux_api::drm::ioctl::amd::*;
use amdgpu_linux_api::drm::ioctl::drm::*;
use amdgpu_linux_api::drm::*;
use amdgpu_linux_api::kfd;
use amdgpu_linux_api::kfd::apertures::*;
use amdgpu_linux_api::kfd::ioctl::*;
use amdgpu_linux_api::kfd::mmap::Offset;
use amdgpu_linux_api::kfd::*;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;

fn allocate_non_userptr_memory(
    gpu: kfd::ioctl::ProcessDeviceApertures,
    fd: std::os::unix::prelude::BorrowedFd<'_>,
    va: u64,
    size: usize,
    domain_flag: u32,
) -> (MemoryHandle, Offset) {
    let mut args = kfd::ioctl::AllocMemoryOfGpuArgs {
        va_addr: va,
        size,
        handle: 0,
        mmap_offset: 0,
        gpu_id: gpu.gpu_id,
        flags: domain_flag | alloc_flags::PUBLIC | alloc_flags::WRITABLE | alloc_flags::EXECUTABLE,
    };
    let res = unsafe { kfd::ioctl::alloc_memory_of_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    let handle = args.handle;
    let mmap_offset = args.mmap_offset;
    let mut args = kfd::ioctl::MapMemoryToGpuArgs {
        handle: args.handle,
        device_ids_array_ptr: [gpu.gpu_id].as_ptr(),
        n_devices: 1,
        n_success: 0,
    };
    let res = unsafe { kfd::ioctl::map_memory_to_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    assert!(args.n_success == args.n_devices);
    (handle, mmap_offset as Offset)
}

fn mmap<'a>(drm: &impl DrmFile, offset: Offset, size: usize) -> &'a mut [u8] {
    let res = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            drm.as_fd().as_raw_fd(),
            offset as i64,
        )
    };
    assert!(res != libc::MAP_FAILED);
    unsafe { std::slice::from_raw_parts_mut(res.cast(), size) }
}

use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use uds::UnixStreamExt; // Provides send_fds and recv_fds

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let kfd = Kfd1_18::open().unwrap();
        let devs = kfd.all_apertures().unwrap();
        let drm = AmdgpuDrmRender3_64::open(128).unwrap();
        let kfd = match kfd.acquire_vm(&devs[0], &drm) {
            AcquireVmResult::Ok(x) => x,
            _ => panic!(),
        };
        let fd = kfd.as_fd().as_raw_fd();
        let (handle, offst) = allocate_non_userptr_memory(
            devs[0],
            kfd.as_fd(),
            0x10_000,
            0x1_000,
            alloc_domain::VRAM,
        );
        let vram = mmap(&drm, offst, 0x1_000);
        vram.fill(0xC9);

        println!("Allocating Vram in kfd, handle: {handle}");

        let mut args = ExportDmabufArgs {
            handle,
            flags: 0,
            dmabuf_fd: 0,
        };
        let res = unsafe { export_dmabuf(fd, &mut args) };
        assert!(res.is_ok());

        let dmabuf_fd = args.dmabuf_fd;
        println!("Exporting kfd's memory as dmabuf: {dmabuf_fd}");

        // --- PARENT PROCESS ---
        // 1. Create a pair of connected sockets
        let (parent_sock, child_sock) = UnixStream::pair()?;

        // 2. Spawn the child, passing the socket FD as inherited FD 3
        let mut child = std::process::Command::new(std::env::current_exe()?)
            .arg("child")
            .stdin(OwnedFd::from(child_sock))
            .spawn()?;

        // 3. Open a file we want to share
        let file_to_share = dmabuf_fd;

        // 4. Send the FD via the Unix Socket
        parent_sock.send_fds(b"here is your fd", &[file_to_share.as_raw_fd()])?;

        child.wait()?;
        assert!(vram[0] == 0x33);
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

            let drm = AmdgpuDrmRender3_64::open(128).unwrap();
            let mut args = drm::ioctl::drm::PrimeHandle {
                handle: 0,
                flags: libc::O_RDWR as u32,
                fd: shared_file.as_raw_fd(),
            };
            let res = unsafe { prime_fd_to_handle(drm.as_fd().as_raw_fd(), &mut args) };
            assert!(res.is_ok());
            let mut arsg = drm::ioctl::amd::GemMmap {
                in_: GemMmapIn {
                    handle: args.handle,
                    _pad: 0,
                },
            };
            let res = unsafe { gem_mmap(drm.as_fd().as_raw_fd(), &mut arsg) };
            assert!(res.is_ok());
            let imported_vram = mmap(&drm, unsafe { arsg.out }.addr_ptr as i64, 0x1_000);
            assert!(imported_vram[0] == 0xC9);
            imported_vram.fill(0x33);
            println!("[Child] Successfully mmaped memory and changed it");
        }
        println!("[Child] Exiting");
    }

    Ok(())
}
