use amdgpu_linux_api::drm::ioctl;
use std::os::fd::AsRawFd;

fn main() {
    let render = std::fs::File::open("/dev/dri/renderD128").unwrap();

    let mut args = ioctl::DrmAmdgpuGemCreate {
        input: ioctl::DrmAmdgpuGemCreateIn {
            alignment: 0,
            bo_size: 4096,
            domains: 0,
            domain_flags: 0,
        },
    };

    let res = unsafe { ioctl::amdgpu_ioctl_gem_create(render.as_raw_fd(), &mut args) };
    assert!(matches!(res, Ok(_)));

    let primary = std::fs::File::open("/dev/dri/card1").unwrap();

    println!("Check if this client is authenticated");
    //let _ = std::io::stdin().read_line(&mut String::new());

    let mut args = ioctl::DrmAmdgpuGemCreate {
        input: ioctl::DrmAmdgpuGemCreateIn {
            alignment: 0,
            bo_size: 4096,
            domains: 0,
            domain_flags: 0,
        },
    };
    let res = unsafe { ioctl::amdgpu_ioctl_gem_create(primary.as_raw_fd(), &mut args) };
    assert!(matches!(res, Err(libc::EACCES)));
    println!("But with primary node, authentication is required");
}
