use amdgpu_linux_api::{
    SIGNAL_PAGES_SIZE,
    drm::AmdgpuDrmRender3_64,
    kfd::{
        self, AcquireVm, Kfd1_18,
        apertures::AperturesNew,
        ioctl::{
            AllocMemoryOfGpuArgs, CreateEventArgs, CreateQueueArgs, EventData, GpuId,
            HsaHwExceptionData, HsaMemoryExceptionData, HsaSignalEventData, MapMemoryToGpuArgs,
            MemoryHandle, RuntimeEnableArgs, VirtualAddress, WaitEventsArgs, alloc_domain,
            alloc_flags, alloc_memory_of_gpu, create_event, create_queue, event_type,
            map_memory_to_gpu, queue_type, runtime_enable, wait_events, wait_result,
        },
    },
    sdma_new as sdma,
};
use std::os::fd::{AsFd, AsRawFd};

fn assert_map_memory(fd: impl AsRawFd, handle: MemoryHandle, dev_ids: &[GpuId]) {
    let mut args = MapMemoryToGpuArgs {
        handle,
        device_ids_array_ptr: dev_ids.as_ptr(),
        n_devices: dev_ids.len() as u32,
        n_success: 0,
    };
    let res = unsafe { map_memory_to_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    assert!(args.n_success == args.n_devices);
}

fn alloc_and_map_userptr(
    fd: impl AsRawFd,
    mem: &[u32],
    va: VirtualAddress,
    gpu_id: GpuId,
) -> MemoryHandle {
    let mut args = AllocMemoryOfGpuArgs {
        va_addr: va,
        size: mem.len(),
        handle: 0,
        mmap_offset: mem.as_ptr() as u64,
        gpu_id,
        flags: alloc_domain::USERPTR
            | alloc_flags::PUBLIC
            | alloc_flags::WRITABLE
            | alloc_flags::EXECUTABLE,
    };
    let res = unsafe { alloc_memory_of_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    assert_map_memory(fd, args.handle, &[gpu_id]);
    args.handle
}

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let devs = kfd.all_apertures().unwrap();
    let dev = &devs[0];

    let amdgpu_linux_api::kfd::AcquireVmResult::Ok(kfd) = kfd.acquire_vm(dev, &drm) else {
        panic!("Acquire VM")
    };
    let fd = kfd.as_fd();

    let mut args = AllocMemoryOfGpuArgs {
        va_addr: 0x10_000,
        size: SIGNAL_PAGES_SIZE!(),
        handle: 0,
        mmap_offset: 0,
        gpu_id: dev.gpu_id,
        flags: alloc_domain::GTT
            | alloc_flags::WRITABLE
            | alloc_flags::PUBLIC
            | alloc_flags::UNCACHED,
    };
    let res = unsafe { alloc_memory_of_gpu(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    let kfd_mem_handle = args.handle;
    assert_map_memory(fd, kfd_mem_handle, &[dev.gpu_id]);

    let mut ring_mem = [0u32; 1024];
    let _ring_handle = alloc_and_map_userptr(fd, &ring_mem, 0x20_000, dev.gpu_id);

    let mut controlls = [0u32; 1024];
    let _controlls_handle = alloc_and_map_userptr(fd, &controlls, 0x30_000, dev.gpu_id);

    let mut ctx_save_restore = [0u32; 2 * 1024];
    let _ctx_save_handle = alloc_and_map_userptr(fd, &ctx_save_restore, 0x40_000, dev.gpu_id);

    let mut args = CreateEventArgs {
        event_page_offset: (u64::from(dev.gpu_id) << 32) | kfd_mem_handle,
        event_trigger_data: 0,
        event_type: event_type::SIGNAL,
        auto_reset: 0,
        node_id: 0,
        event_id: 0,
        event_slot_index: 0,
    };
    let res = unsafe { create_event(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    assert!(args.event_id != 0);
    let signal_event = args.event_id;

    let mut args = CreateEventArgs {
        event_page_offset: 0,
        event_trigger_data: 0,
        event_type: event_type::HW_EXCEPTION,
        auto_reset: 0,
        node_id: 0,
        event_id: 0,
        event_slot_index: 0,
    };
    let res = unsafe { create_event(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    let hw_excpection_event = args;

    let mut args = CreateEventArgs {
        event_page_offset: 0,
        event_trigger_data: 0,
        event_type: event_type::MEMORY,
        auto_reset: 0,
        node_id: 0,
        event_id: 0,
        event_slot_index: 0,
    };
    let res = unsafe { create_event(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    let memory_event = args;

    let (a, b) = controlls.split_at_mut(4);
    let wptr = &mut a[0];
    let rptr = &mut b[0];
    let mut fence = sdma::v5_2::Fence {
        addr: u64::from(0x10_000 + signal_event * 8),
        data: 1,
        mtype: sdma::v5_2::Mtype::Uncached,
        ..Default::default()
    };
    sdma::v5_2::Pkt::Fence(fence).encode_linear(&mut ring_mem[0..4]);
    fence.addr += 4;
    fence.data = 0;
    sdma::v5_2::Pkt::Fence(fence).encode_linear(&mut ring_mem[4..8]);

    let trap = sdma::v5_2::Trap {
        int_context: signal_event,
    };
    sdma::v5_2::Pkt::Trap(trap).encode_linear(&mut ring_mem[8..10]);

    *wptr += 10;

    let mut args = RuntimeEnableArgs {
        r_debug: 1,
        mode_mask: 0,
        capabilities_mask: 0,
    };
    let res = unsafe { runtime_enable(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());

    let mut args = CreateQueueArgs {
        ring_base_address: ring_mem.as_ptr(),
        write_pointer_address: wptr,
        read_pointer_address: rptr,
        doorbell_offset: 0,
        ring_size: size_of_val(&ring_mem) as u32,
        gpu_id: dev.gpu_id,
        queue_type: queue_type::SDMA_BY_ENG_ID,
        queue_percentage: 0,
        queue_priority: 0xf,
        queue_id: 0,
        eop_buffer_address: std::ptr::null_mut(),
        eop_buffer_size: 0,
        ctx_save_restore_address: ctx_save_restore.as_mut_ptr(),
        ctx_save_restore_size: 0x2_000,
        ctl_stack_size: 0x1_000,
        sdma_engine_id: 0,
        pad: 0,
    };

    let res = unsafe { create_queue(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    println!("{args:#?}");

    let mut events = [
        EventData {
            data: kfd::ioctl::EventDataUnion {
                hw_exception_data: kfd::ioctl::HsaHwExceptionData::default(),
            },
            kfd_event_data_ext: std::ptr::null_mut(),
            event_id: hw_excpection_event.event_id,
            _pad: 0,
        },
        EventData {
            data: kfd::ioctl::EventDataUnion {
                memory_exception_data: HsaMemoryExceptionData::default(),
            },
            kfd_event_data_ext: std::ptr::null_mut(),
            event_id: memory_event.event_id,
            _pad: 0,
        },
        EventData {
            data: kfd::ioctl::EventDataUnion {
                signal_event_data: HsaSignalEventData { last_event_age: 1 },
            },
            kfd_event_data_ext: std::ptr::null_mut(),
            event_id: signal_event,
            _pad: 0,
        },
    ];

    let mut args = WaitEventsArgs {
        events_ptr: events.as_mut_ptr(),
        num_events: events.len() as u32,
        wait_for_all: 0,
        timeout: u32::MAX,
        wait_result: 0,
    };
    let res = unsafe { wait_events(fd.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    assert_eq!(args.wait_result, wait_result::COMPLETE);
    let hw_excpt = unsafe { events[0].data.hw_exception_data };
    if hw_excpt != HsaHwExceptionData::default() {
        println!("{hw_excpt:#?}");
    }
    let mem_excpt = unsafe { events[1].data.memory_exception_data };
    if mem_excpt != HsaMemoryExceptionData::default() {
        println!("{mem_excpt:#?}");
    }
    let signal_ev = unsafe { events[2].data.signal_event_data };
    if signal_ev.last_event_age > 1 {
        println!("Signal: {}", signal_ev.last_event_age);
    }
}
