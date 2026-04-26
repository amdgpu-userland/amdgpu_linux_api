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
};

fn alloc_and_map_vram(fd: RawFd, size: usize) -> (GemHandle, *mut c_void) {
    let mut args = ioctl::amd::GemCreate {
        input: GemCreateIn {
            bo_size: size,
            alignment: 0,
            domains: ioctl::amd::gem_domain::VRAM,
            domain_flags: gem_flags::CPU_ACCESS_REQUIRED,
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

fn get_sdma_rings(fd: RawFd) -> Vec<(u32, u32)> {
    let mut available_sdma_rings = vec![];

    let mut info_hw_ip = ioctl::amd::InfoHwIp::default();
    let mut args = ioctl::amd::Info {
        return_pointer: (&raw mut info_hw_ip).cast(),
        return_size: size_of_val(&info_hw_ip).try_into().unwrap(),
        query: ioctl::amd::InfoQuery::HwIpInfo,
        quick_info: std::mem::MaybeUninit::new(ioctl::amd::InfoUnion {
            query_hw_ip: ioctl::amd::QueryHwIp {
                r#type: ioctl::amd::HwIp::DMA,
                ip_instance: 0,
            },
        }),
    };
    unsafe { ioctl::amd::info(fd, &mut args).unwrap() };

    for ring in 0..32 {
        if (info_hw_ip.available_rings & (1 << ring)) != 0 {
            available_sdma_rings.push((0, ring));
        }
    }

    available_sdma_rings
}

fn submit_ring(
    fd: RawFd,
    ctx_id: CtxId,
    bo_list: BoListHandle,
    va_start: VirtualAddress,
    ib_bytes: u32,
    ip_instance: u32,
    ring: u32,
) -> u64 {
    let chunk_ib = ioctl::amd::CsChunkIb {
        _pad: 0,
        flags: 0,
        va_start,
        ib_bytes,
        ip_type: ioctl::amd::HwIp::DMA,
        ip_instance,
        ring,
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
    unsafe { args.out }.handle
}

fn wait_cs(fd: RawFd, ctx_id: CtxId, cs_handle: u64, ip_instance: u32, ring: u32) {
    let mut args = ioctl::amd::CsWait {
        in_: ioctl::amd::CsWaitIn {
            handle: cs_handle,
            timeout: u64::MAX,
            ip_type: ioctl::amd::HwIp::DMA,
            ip_instance,
            ring,
            ctx_id,
        },
    };
    let res = unsafe { ioctl::amd::cs_wait(fd, &mut args) };
    assert!(res.is_ok());
    let wait_status = unsafe { args.out }.status;
    assert_eq!(wait_status, 0);
}

fn main() {
    // This is broken and hangs the gpu

    let drm_file = AmdgpuDrmRender3_64::open(128).unwrap();
    let fd = drm_file.as_fd().as_raw_fd();

    let rings = get_sdma_rings(fd);
    if rings.len() < 2 {
        println!("This example requires at least 2 SDMA rings. Found: {rings:?} Exiting.");
        return;
    }
    println!("Found SDMA rings: {rings:?}");
    let (ip_instance0, ring0) = rings[0];
    let (ip_instance1, ring1) = rings[1];

    // Allocate 1MB VRAM
    const GEM_SIZE: usize = 1024 * 1024;
    let va_base = 0x10_000u64;

    let (handle, vram_ptr) = alloc_and_map_vram(fd, GEM_SIZE);
    let vram: &mut [u32] =
        unsafe { std::slice::from_raw_parts_mut(vram_ptr.cast::<u32>(), GEM_SIZE / 4) };
    map_bo_to_va(fd, handle, va_base.try_into().unwrap(), GEM_SIZE);

    let ctx_id = create_ctx(fd);
    let bo_list = create_bo_list(fd, &[handle]);

    // Offsets in dwords
    let ib0_dw = 0;
    let ib1_dw = 1000;
    let flag_dw = 2000;
    let seq0_dw = 2001;

    // Reset flag
    vram[flag_dw] = 0;
    vram[seq0_dw] = 0;

    // Encode IB for ring 0: Polls memory until flag_dw turns to 1.
    // Putting it in a loop ensures it blocks indefinitely if retry runs out.
    let mut sz0 = 0;
    for _ in 0..10_000 {
        sz0 += sdma::v5_2::Pkt::PollRegmem(sdma::v5_2::PollRegmem {
            addr: va_base + (flag_dw * 4) as u64,
            value: 1,
            mask: 0xFFFFFFFF,
            interval: 0xFFFF,
            retry_count: 0xFFF,
            func: 3, // EQUAL
            mem_poll: true,
            ..Default::default()
        })
        .encode_linear(&mut vram[ib0_dw + sz0..]);
    }
    // Write out seq0 = 1 when successfully unblocked
    sz0 += sdma::v5_2::Pkt::Fence(sdma::v5_2::Fence {
        addr: va_base + (seq0_dw * 4) as u64,
        data: 1,
        mtype: sdma::v5_2::Mtype::Uncached,
        ..Default::default()
    })
    .encode_linear(&mut vram[ib0_dw + sz0..]);
    sz0 += sdma::v5_2::Pkt::Trap(sdma::v5_2::Trap {
        int_context: 0x0_FFC,
    })
    .encode_linear(&mut vram[ib0_dw + sz0..]);

    // Encode IB for ring 1: Writes 1 to flag_dw to unblock ring0
    let mut sz1 = 0;
    sz1 += sdma::v5_2::Pkt::Fence(sdma::v5_2::Fence {
        addr: va_base + (flag_dw * 4) as u64,
        data: 1,
        mtype: sdma::v5_2::Mtype::Uncached,
        ..Default::default()
    })
    .encode_linear(&mut vram[ib1_dw + sz1..]);
    sz1 += sdma::v5_2::Pkt::Trap(sdma::v5_2::Trap {
        int_context: 0x0_FFC,
    })
    .encode_linear(&mut vram[ib1_dw + sz1..]);

    println!("Starting Ring 0 (Blocking Poll)...");
    let t0 = std::time::SystemTime::now();

    let cs0_par = submit_ring(
        fd,
        ctx_id,
        bo_list,
        va_base + (ib0_dw * 4) as u64,
        (sz0 * 4) as u32,
        ip_instance0,
        ring0,
    );

    // Sleep on CPU for 500ms
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Check if ring 0 is stuck
    let stuck = vram[seq0_dw] == 0;
    println!(
        "After 500ms CPU sleep, seq0 == {} (0 means correctly stuck waiting for flag)",
        vram[seq0_dw]
    );
    assert!(
        stuck,
        "Ring 0 completed prematurely! Is PollRegmem working?"
    );

    println!("Submitting Ring 1 to unblock Ring 0 in parallel...");
    let cs1_par = submit_ring(
        fd,
        ctx_id,
        bo_list,
        va_base + (ib1_dw * 4) as u64,
        (sz1 * 4) as u32,
        ip_instance1,
        ring1,
    );

    wait_cs(fd, ctx_id, cs0_par, ip_instance0, ring0);
    wait_cs(fd, ctx_id, cs1_par, ip_instance1, ring1);

    let dur = std::time::SystemTime::now().duration_since(t0).unwrap();
    println!(
        "Total execution time: {:?}. Successfully proved parallel execution!",
        dur
    );

    assert_eq!(vram[seq0_dw], 1);
    assert_eq!(vram[flag_dw], 1);

    drop(drm_file);
}
