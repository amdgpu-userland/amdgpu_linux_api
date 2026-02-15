use std::{io::stdin, os::fd::AsRawFd};

#[test]
fn opening_kfd() {
    let kfd = amdkfd::Kfd::open().expect("Please run this on linux with a modern AMD gpu");
    let version = kfd.version();
    println!("{version:?}");
    drop(kfd);
}

#[test]
fn creating_small_sdma_queue() {
    let mut ring = [0u32; 1024];
    let mut rptr = 0;
    let mut wptr = 0;
    let kfd = amdkfd::Kfd::open().expect("Hello");
    let mut out = amdkfd::KfdIoctlCreateQueueArgs {
        gpu_id: 34961,
        queue_type: amdkfd::QueueType::Sdma as u32,
        ring_base_address: &raw const ring as u64,
        // It's not really const
        read_pointer_address: &raw mut rptr as u64,
        write_pointer_address: &raw const wptr as u64,
        ring_size: 1024,
        ..Default::default()
    };
    let res = unsafe {
        libc::ioctl(
            kfd.file.as_raw_fd(),
            amdkfd::AMDKFD_IOC_CREATE_QUEUE,
            &raw mut out,
        )
    };
    println!("{res}, errno: {}", std::io::Error::last_os_error());
    let mut _line = String::new();
    stdin().read_line(&mut _line);
}
