use amdgpu_linux_api::drm::driver_capabilities::*;
use amdgpu_linux_api::drm::{AmdgpuDrmPrimary3_64, AmdgpuDrmRender3_64, ioctl};
use std::os::fd::{AsFd, AsRawFd};

fn main() {
    let drm_primary = AmdgpuDrmRender3_64::open(128).unwrap();
    let fd = drm_primary.as_fd().as_raw_fd();

    let caps = [
        ("CAP_DUMB_BUFFER", CAP_DUMB_BUFFER),
        ("CAP_VBLANK_HIGH_CRTC", CAP_VBLANK_HIGH_CRTC),
        ("CAP_DUMB_PREFERRED_DEPTH", CAP_DUMB_PREFERRED_DEPTH),
        ("CAP_DUMB_PREFER_SHADOW", CAP_DUMB_PREFER_SHADOW),
        ("CAP_PRIME", CAP_PRIME),
        ("CAP_PRIME_IMPORT", CAP_PRIME_IMPORT),
        ("CAP_PRIME_EXPORT", CAP_PRIME_EXPORT),
        ("CAP_TIMESTAMP_MONOTONIC", CAP_TIMESTAMP_MONOTONIC),
        ("CAP_ASYNC_PAGE_FLIP", CAP_ASYNC_PAGE_FLIP),
        ("CAP_CURSOR_WIDTH", CAP_CURSOR_WIDTH),
        ("CAP_CURSOR_HEIGHT", CAP_CURSOR_HEIGHT),
        ("CAP_ADDFB2_MODIFIERS", CAP_ADDFB2_MODIFIERS),
        ("CAP_PAGE_FLIP_TARGET", CAP_PAGE_FLIP_TARGET),
        ("CAP_CRTC_IN_VBLANK_EVENT", CAP_CRTC_IN_VBLANK_EVENT),
        ("CAP_SYNCOBJ", CAP_SYNCOBJ),
        ("CAP_SYNCOBJ_TIMELINE", CAP_SYNCOBJ_TIMELINE),
        ("CAP_ATOMIC_ASYNC_PAGE_FLIP", CAP_ATOMIC_ASYNC_PAGE_FLIP),
    ];

    for (name, cap) in caps {
        let mut args = ioctl::drm::GetCap {
            capability: cap,
            value: 0,
        };
        let res = unsafe { ioctl::drm::get_cap(fd, &mut args) };
        if res.is_ok() {
            println!("{}: {}", name, args.value);
        } else {
            println!("{}: Error ({:?})", name, res.unwrap_err());
        }
    }
}
