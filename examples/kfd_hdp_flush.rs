use std::{sync::atomic::AtomicBool, time::Duration};

use amdgpu_linux_api::{
    drm::AmdgpuDrmRender3_64,
    kfd::{AcquireVm, Kfd1_18, apertures::AperturesNew, mmap::RemapMmio},
};
use nix::sys::signal::*;

static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn sig_handler(sig: libc::c_int) {
    STOP.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let devs = kfd.all_apertures().unwrap();
    let mut mmio = kfd.mmio(&devs[0]);
    let handler = SigHandler::Handler(sig_handler);
    let drm_file = AmdgpuDrmRender3_64::open(128).unwrap();
    let kfd = kfd.acquire_vm(&devs[0], &drm_file);

    let sa = SigAction::new(handler, SaFlags::empty(), SigSet::all());
    unsafe { sigaction(Signal::SIGINT, &sa) }.unwrap();
    println!("Hit CTRL+c to terminate");
    loop {
        mmio.flush_hdp_mem();
        mmio.flush_hdp_reg();
        std::thread::sleep(Duration::from_micros(100));
        if STOP.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
    }
    drop(kfd);
}
