use amdgpu_linux_api::{
    drm::AmdgpuDrmRender3_64,
    kfd::{
        AcquireVm, Kfd1_18,
        apertures::AperturesNew,
        ioctl::{AllocMemoryOfGpuArgs, alloc_domain, alloc_flags, alloc_memory_of_gpu},
    },
};
use std::os::fd::{AsFd, AsRawFd};

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let devs = kfd.all_apertures().unwrap();
    let dev = &devs[0];

    let amdgpu_linux_api::kfd::AcquireVmResult::Ok(kfd) = kfd.acquire_vm(dev, &drm) else {
        panic!("Acquire VM")
    };
    let fd = kfd.as_fd();

    const SIZE: usize = 0x2_000;

    let mut args = AllocMemoryOfGpuArgs {
        va_addr: dev.gpuvm_base,
        size: SIZE,
        handle: 0,
        mmap_offset: 0,
        gpu_id: dev.gpu_id,
        flags: alloc_domain::VRAM
            | alloc_flags::WRITABLE
            | alloc_flags::PUBLIC
            | alloc_flags::UNCACHED,
    };
    let res = unsafe { alloc_memory_of_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    let offset = args.mmap_offset;
    let vram = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            drm.as_fd().as_raw_fd(),
            offset as i64,
        )
    };
    assert!(vram != libc::MAP_FAILED);
    let vram: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(vram.cast(), SIZE / 4) };
    vram.fill(0xCAFEBABE);

    println!("vram_cpu_addr: {}", vram.as_ptr().addr());
    println!("vram[0]: {}", vram[0]);

    let _ = vram;
    drop(kfd);
    drop(drm);
}
