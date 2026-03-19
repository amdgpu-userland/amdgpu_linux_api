use amdgpu_linux_api::{
    SIGNAL_PAGES_SIZE,
    drm::AmdgpuDrmRender3_64,
    kfd::{
        self, AcquireVm, Kfd1_18,
        apertures::AperturesNew,
        ioctl::{
            AllocMemoryOfGpuArgs, CreateEventArgs, EventData, HsaHwExceptionData,
            HsaMemoryExceptionData, HsaSignalEventData, WaitEventsArgs, alloc_domain, alloc_flags,
            alloc_memory_of_gpu, create_event, event_type, wait_events, wait_result,
        },
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

    let mut args = AllocMemoryOfGpuArgs {
        va_addr: dev.gpuvm_base,
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

    let signal_pages = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            SIGNAL_PAGES_SIZE!(),
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED | libc::MAP_POPULATE | libc::MAP_NORESERVE,
            fd.as_raw_fd(),
            kfd::mmap::EVENTS | kfd::mmap::gpu_id(dev.gpu_id),
        )
    };
    assert!(signal_pages != libc::MAP_FAILED, "mapping signal pages");
    let signal_pages: *mut u64 = signal_pages.cast();
    unsafe {
        signal_pages.offset(signal_event as isize).write_volatile(1);
    };
    println!("Mapped signals to: {signal_pages:?}");

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
