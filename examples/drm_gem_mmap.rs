#![feature(ptr_cast_array)]
use amdgpu_linux_api::drm::{
    AmdgpuDrmRender3_64,
    ioctl::{
        self,
        amd::{GemCreateIn, GemMmapIn, gem_flags},
    },
};
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;

fn main() {
    let drm_file = AmdgpuDrmRender3_64::open(128).unwrap();
    let fd = drm_file.as_fd().as_raw_fd();

    const GEM_SIZE: usize = 0x1_000;

    let mut args = ioctl::amd::GemCreate {
        input: GemCreateIn {
            bo_size: GEM_SIZE,
            alignment: 0,
            domains: ioctl::amd::gem_domain::VRAM,
            domain_flags: gem_flags::CPU_ACCESS_REQUIRED, //| gem_flags::UNCACHED,
        },
    };
    let res = unsafe { ioctl::amd::gem_create(fd, &mut args) };
    assert!(res.is_ok());
    let handle = unsafe { args.output }.handle;

    let mut args = ioctl::amd::GemMmap {
        in_: GemMmapIn { handle, _pad: 0 },
    };
    let res = unsafe { ioctl::amd::gem_mmap(fd, &mut args) };
    assert!(res.is_ok());
    let gem_mmap_offset = unsafe { args.out }.addr_ptr;
    println!("Mmap offset: {gem_mmap_offset}");

    let vram_ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            GEM_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            gem_mmap_offset as i64,
        )
    };
    assert!(vram_ptr != libc::MAP_FAILED);

    let vram: &mut [u8; GEM_SIZE] = unsafe { vram_ptr.cast::<u8>().cast_array().as_mut().unwrap() };
    const SOME_BYTE_VALUE: u8 = 69;
    vram.fill(SOME_BYTE_VALUE);

    let _ = vram;
    unsafe { libc::munmap(vram_ptr, GEM_SIZE) };

    let vram_ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            GEM_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            gem_mmap_offset as i64,
        )
    };
    assert!(vram_ptr != libc::MAP_FAILED);

    let vram: &mut [u8; GEM_SIZE] = unsafe { vram_ptr.cast::<u8>().cast_array().as_mut().unwrap() };
    assert_eq!(vram[0], SOME_BYTE_VALUE);

    drop(drm_file);
}
