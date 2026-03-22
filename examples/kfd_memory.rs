use amdgpu_linux_api::{
    GPU_PAGE_SIZE,
    drm::{AmdgpuDrmRender3_64, DrmFile},
    kfd::{
        self, AcquireVm, AcquireVmResult, Kfd1_18,
        apertures::AperturesNew,
        gpu_id::AsGpuId,
        ioctl::{MemoryHandle, alloc_domain, alloc_flags},
        mmap::{self, MMIO_REMAP, Offset, RemapMmio},
    },
};
use std::os::fd::{AsFd, AsRawFd, RawFd};

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let gpus = kfd.all_apertures().unwrap();
    let gpu = gpus[0];
    let AcquireVmResult::Ok(kfd) = kfd.acquire_vm(&gpu, &drm) else {
        panic!()
    };
    let fd = kfd.as_fd();
    let size: usize = 0x1_000;

    let (_hdl, gtt_offst) = allocate_non_userptr_memory(gpu, fd, 0x10_000, size, alloc_domain::GTT);
    let mem = mmap(&drm, gtt_offst, size);
    mem.fill(0xA);

    let (_hdl, vram_offst) =
        allocate_non_userptr_memory(gpu, fd, 0x20_000, size, alloc_domain::VRAM);
    let vram = mmap(&drm, vram_offst, size);
    vram.fill(0xB);

    let (_hdl, offst) =
        allocate_non_userptr_memory(gpu, fd, 0x30_000, size, alloc_domain::MMIO_REMAP);
    println!("MMIO REMAP mmap offset: {offst:#x}");
    let mmio = mmap_mmio_remap(kfd.as_fd().as_raw_fd(), &gpu);
    let mut mmio2 = kfd.mmio(&gpu);
    mmio2.flush_hdp_reg();
    mmio2.flush_hdp_mem();
    mmio[0] = 1;
    //assert!(unsafe { mmio.as_ptr().read_volatile() } == 0);

    let (_hdl, sig_offst) =
        allocate_non_userptr_memory(gpu, fd, 0x40_000, 0x2_000, alloc_domain::DOORBELL);
    println!("DOORBELLS mmap offset: {sig_offst:#x}");
    let doorbells = mmap(&drm, sig_offst, 0x2_000);
    let (_, doorbells, _) = unsafe { doorbells.align_to_mut::<u64>() };

    let doorbells_kfd = mmap_doorbells(fd.as_raw_fd(), &gpu);
    println!("Doorbells drm mmap addr: {:#x}", doorbells.as_ptr().addr());
    println!(
        "Doorbells kfd mmap addr: {:#x}",
        doorbells_kfd.as_ptr().addr()
    );

    doorbells_kfd.fill(0xC);

    println!("Now we've set up all memory types. Try to break it");

    unsafe { libc::munmap(mem.as_mut_ptr().cast(), size) };
    unsafe { libc::munmap(vram.as_mut_ptr().cast(), size) };
    unsafe { libc::munmap(mmio.as_mut_ptr().cast(), size) };
    unsafe { libc::munmap(doorbells.as_mut_ptr().cast(), 0x2_000) };

    let mem = mmap(&drm, gtt_offst, size);
    let vram = mmap(&drm, vram_offst, size);
    let mmio = mmap_mmio_remap(kfd.as_fd().as_raw_fd(), &gpu);
    let doorbells = mmap(&drm, sig_offst, 0x2_000);
    let (_, doorbells, _) = unsafe { doorbells.align_to_mut::<u64>() };

    println!("Let's see what is in these Buffer Objects now");
    assert!(mem[0] == 0xA);
    assert!(vram[0] == 0xB);
    let _ = mmio;
    let _ = doorbells;
    //assert!(mmio[0] == 0);
    //assert!(doorbells_kfd[0] == 0xC);
}

fn mmap_mmio_remap<'a>(kfd: RawFd, gpu_id: &impl AsGpuId) -> &'a mut [u8] {
    let res = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            GPU_PAGE_SIZE!(),
            libc::PROT_WRITE | libc::PROT_READ,
            libc::MAP_SHARED,
            kfd,
            MMIO_REMAP | mmap::gpu_id(gpu_id.gpu_id()),
        )
    };
    assert!(res != libc::MAP_FAILED);
    unsafe { std::slice::from_raw_parts_mut(res.cast(), GPU_PAGE_SIZE!()) }
}

fn mmap_doorbells<'a>(kfd: RawFd, gpu_id: &impl AsGpuId) -> &'a mut [u8] {
    let size = 2 * GPU_PAGE_SIZE!();
    let res = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_WRITE | libc::PROT_READ,
            libc::MAP_SHARED,
            kfd,
            mmap::DOORBELL | mmap::gpu_id(gpu_id.gpu_id()),
        )
    };
    assert!(res != libc::MAP_FAILED);
    unsafe { std::slice::from_raw_parts_mut(res.cast(), size) }
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
