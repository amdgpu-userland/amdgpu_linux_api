#![feature(ptr_cast_array)]
use amdgpu_linux_api::{
    drm::{
        AmdgpuDrmRender3_64, GemHandle,
        ioctl::{
            self,
            amd::{
                BoListEntry, BoListHandle, BoListIn, CsIn, CtxId, CtxIn, GemCreateIn, GemMmapIn,
                GemVa, gem_flags, gem_va, map_flags,
            },
        },
    },
    kfd::ioctl::VirtualAddress,
    sdma,
};
use std::os::fd::AsFd;
use std::{
    ffi::c_void,
    os::fd::{AsRawFd, RawFd},
    time::Duration,
};

fn alloc_and_map_vram(fd: RawFd, size: usize) -> (GemHandle, *mut c_void) {
    let mut args = ioctl::amd::GemCreate {
        input: GemCreateIn {
            bo_size: size,
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
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            gem_mmap_offset as i64,
        )
    };
    assert!(vram_ptr != libc::MAP_FAILED);
    (handle, vram_ptr)
}

fn map_bo_to_va(fd: RawFd, handle: GemHandle, va: VirtualAddress, size: usize) {
    let mut args = GemVa {
        handle,
        _pad: 0,
        operation: ioctl::amd::VaOp::Map,
        flags: map_flags::PAGE_EXECUTABLE | map_flags::PAGE_READABLE | map_flags::PAGE_WRITEABLE,
        va_address: va as usize,
        offset_in_bo: 0,
        map_size: size,
        vm_timeline_point: 0,
        vm_timeline_syncobj_out: 0,
        num_syncobj_handles: 0,
        input_fence_syncobj_handles: std::ptr::null(),
    };
    let res = unsafe { gem_va(fd, &mut args) };
    assert!(res.is_ok())
}

fn create_ctx(fd: RawFd) -> CtxId {
    let mut args = ioctl::amd::Ctx {
        in_: CtxIn {
            op: ioctl::amd::CtxOp::AllocCtx,
            flags: 0,
            ctx_id: 0,
            priority: ioctl::amd::CtxPriority::Normal,
        },
    };
    let res = unsafe { ioctl::amd::ctx(fd, &mut args) };
    assert!(res.is_ok());
    unsafe { args.out.alloc }.ctx_id
}

fn create_bo_list(fd: RawFd, list: &[GemHandle]) -> BoListHandle {
    let mut bos = Vec::new();
    for handle in list {
        bos.push(BoListEntry {
            bo_handle: *handle,
            bo_priority: 0,
        })
    }
    let mut args = ioctl::amd::BoList {
        in_: BoListIn {
            operation: ioctl::amd::BoListOp::Create,
            list_handle: 0,
            bo_number: bos.len() as u32,
            bo_info_size: size_of::<BoListEntry>() as u32,
            bo_info_ptr: bos.as_ptr(),
        },
    };
    let res = unsafe { ioctl::amd::bo_list(fd, &mut args) };
    assert!(res.is_ok());
    unsafe { args.out }.list_handle
}

fn main() {
    let drm_file = AmdgpuDrmRender3_64::open(128).unwrap();
    let fd = drm_file.as_fd().as_raw_fd();

    const GEM_SIZE: usize = 0x1_000;

    let (handle, vram_ptr) = alloc_and_map_vram(fd, GEM_SIZE);
    let vram: &mut [u32; GEM_SIZE / 4] =
        unsafe { vram_ptr.cast::<u32>().cast_array().as_mut().unwrap() };
    map_bo_to_va(fd, handle, 0x10_000, GEM_SIZE);

    let fence_pkt = sdma::v5::Fence {
        addr: u64::from(0x10_000u32 + 0x0_FFC),
        value: 1,
        mtype: sdma::v5::Mtype::Uncached,
        ..Default::default()
    };
    [vram[0], vram[1], vram[2], vram[3]] = fence_pkt.enc();
    let int_pkt = sdma::v5::Trap {
        context_id: 0x0_FFC,
    };
    [vram[4], vram[5]] = int_pkt.enc();

    let ctx_id = create_ctx(fd);
    let bo_list = create_bo_list(fd, &[handle]);
    let chunk_ib = ioctl::amd::CsChunkIb {
        _pad: 0,
        flags: 0,
        va_start: 0x10_000,
        ib_bytes: 4 * 6,
        ip_type: ioctl::amd::HwIp::DMA,
        ip_instance: 0,
        ring: 0,
    };
    let chunk = ioctl::amd::CsChunk {
        chunk_id: ioctl::amd::ChunkId::IB,
        length_dw: size_of_val(&chunk_ib) as u32,
        chunk_data: &raw const chunk_ib as u64,
    };
    let chunks = [&raw const chunk];

    let mut args = ioctl::amd::Cs {
        in_: CsIn {
            ctx_id,
            bo_list_handle: bo_list,
            num_chunks: chunks.len() as u32,
            flags: 0,
            chunks: chunks.as_ptr(),
        },
    };
    let res = unsafe { ioctl::amd::cs(fd, &mut args) };
    assert!(res.is_ok());
    std::thread::sleep(Duration::from_millis(1));
    assert!(vram[1023] == 1);

    drop(drm_file);
}
