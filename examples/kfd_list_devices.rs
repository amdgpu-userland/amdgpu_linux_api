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
    println!("{vec:#?}")
}
