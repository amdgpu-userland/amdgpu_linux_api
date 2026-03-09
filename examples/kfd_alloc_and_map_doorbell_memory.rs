use amdgpu_linux_api::drm::AmdgpuDrmRender3_64;
use amdgpu_linux_api::kfd::AcquireVm;
use amdgpu_linux_api::kfd::AcquireVmResult;
use amdgpu_linux_api::kfd::Kfd1_18;
use amdgpu_linux_api::kfd::apertures::AperturesNew;
use amdgpu_linux_api::kfd::ioctl::*;
use amdgpu_linux_api::kfd::mmap;
use libc;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;
fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let apertures = kfd.all_apertures().unwrap();
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let kfd = match kfd.acquire_vm(&apertures[0], &drm) {
        AcquireVmResult::Ok(x) => x,
        _ => panic!(),
    };

    let gpu_id = apertures[0].gpu_id;
    let mut args = AllocMemoryOfGpuArgs {
        va_addr: 0,
        size: 2 * 4096,
        handle: 0,
        mmap_offset: 0,
        gpu_id,
        flags: alloc_domain::DOORBELL | alloc_flags::WRITABLE,
    };
    let res = unsafe { alloc_memory_of_gpu(kfd.as_fd().as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    // println!("Gpu id: {:#b}", gpu_id);
    // println!("Mmap offset: {:#b}", args.mmap_offset);

    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            8 * 1024,
            libc::PROT_WRITE,
            libc::MAP_SHARED,
            kfd.as_fd().as_raw_fd(),
            ((mmap::DOORBELL) | mmap::gpu_id(gpu_id)) as i64,
        )
    };
    if ptr.addr() == usize::MAX {
        let res = unsafe { *libc::__errno_location() };
        panic!("Mapping error: {res}")
    }
    let doorbell: &mut [u64] = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u64, 1024) };
    doorbell.fill(100);

    println!("Go see /proc/{}/maps", std::process::id());
    let _ = std::io::stdin().read_line(&mut String::new());
}
