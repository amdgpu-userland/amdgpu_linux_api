#[path = "common/helpers.rs"] mod helpers;
use std::os::fd::{AsFd, AsRawFd};

use helpers::RawRenderNode;
use amdgpu_linux_api::drm::{ioctl};

fn main() {
    let drm = RawRenderNode::open(128).unwrap();
    let _ = unsafe { ioctl::drm::set_master(drm.as_fd().as_raw_fd()) };
    println!(
        "Neither opening a render client nor issuing a set_master ioctl should change authentication status.
Because these ioctls are not marked RENDER_ALLOW.
Hit enter to exit."
    );
    let _ = std::io::stdin().read_line(&mut String::new());
}
