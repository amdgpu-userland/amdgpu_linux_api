use amdgpu_linux_api::drm;
use amdgpu_linux_api::drm::ioctl::gem_domain::*;
use amdgpu_linux_api::drm::ioctl::gem_flags::*;
use amdgpu_linux_api::kfd::ioctl;
use std::mem::MaybeUninit;
use std::os::fd::AsRawFd;

fn main() {
    let file = std::fs::File::open("/dev/kfd").unwrap();

    let mut args = ioctl::GetProcessAperturesNewArgs {
        num_of_nodes: 0,
        ..Default::default()
    };
    // Gets num_of_nodes
    let _ = unsafe { ioctl::get_process_apertures_new(file.as_raw_fd(), &mut args) };

    let mut vec: Vec<MaybeUninit<ioctl::ProcessDeviceApertures>> =
        Vec::with_capacity(args.num_of_nodes as usize);
    unsafe { vec.set_len(args.num_of_nodes as usize) };

    args.kfd_process_device_apertures_ptr = vec.as_mut_ptr() as *mut ioctl::ProcessDeviceApertures;
    let res = unsafe { ioctl::get_process_apertures_new(file.as_raw_fd(), &mut args) };

    assert!(res.is_ok());

    let vec = unsafe {
        std::mem::transmute::<
            Vec<MaybeUninit<ioctl::ProcessDeviceApertures>>,
            Vec<ioctl::ProcessDeviceApertures>,
        >(vec)
    };

    let mut available_args = ioctl::GetAvailableMemoryArgs::default();
    available_args.gpu_id = vec[0].gpu_id;
    let _ = unsafe { ioctl::get_available_memory(file.as_raw_fd(), &mut available_args) };

    let drm_file = std::fs::File::open("/dev/dri/renderD128").unwrap();

    let mut args = drm::ioctl::DrmAmdgpuGemCreate {
        input: drm::ioctl::DrmAmdgpuGemCreateIn {
            alignment: 0,
            bo_size: 4096,
            domains: VRAM,
            domain_flags: VM_ALWAYS_VALID | CPU_ACCESS_REQUIRED,
        },
    };
    let res = unsafe { drm::ioctl::amdgpu_ioctl_gem_create(drm_file.as_raw_fd(), &mut args) };
    assert!(res.is_ok());
    let handle = unsafe { args.output.handle };

    let mut args = drm::ioctl::DrmAmdgpuGemVa {
        _pad: 0,
        flags: 0,
        handle,
        input_fence_syncobj_handles: std::ptr::null(),
        map_size: 4096,
        num_syncobj_handles: 0,
        offset_in_bo: 0,
        va_address: 0x10000,
        operation: drm::ioctl::va_op::MAP,
        vm_timeline_point: 0,
        vm_timeline_syncobj_out: 0,
    };
    let _ = unsafe { drm::ioctl::amdgpu_ioctl_gem_va(drm_file.as_raw_fd(), &mut args) };
    println!("Mapped a gem with handle: {handle:x}");
    let _ = std::io::stdin().read_line(&mut String::new());

    let _ = unsafe {
        ioctl::acquire_vm(
            file.as_raw_fd(),
            &mut ioctl::AcquireVmArgs {
                drm_fd: drm_file.as_raw_fd(), // valid fd is positive
                gpu_id: vec[0].gpu_id,
            },
        )
    };
}
