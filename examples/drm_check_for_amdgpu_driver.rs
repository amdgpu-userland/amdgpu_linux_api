use amdgpu_linux_api::drm::ioctl;
use std::os::fd::AsRawFd;

fn main() {
    let file = std::fs::File::open("/dev/dri/renderD128").unwrap();

    let mut str_buffer = [0u8; 4096];
    let (driver_name, rest) = str_buffer.split_at_mut(1024);
    let (date, desc) = rest.split_at_mut(1024);
    let mut args = ioctl::drm::Version {
        major: 0,
        minor: 0,
        patchlevel: 0,
        name: driver_name.as_mut_ptr(),
        name_len: driver_name.len(),
        date: date.as_mut_ptr(),
        date_len: date.len(),
        desc: desc.as_mut_ptr(),
        desc_len: desc.len(),
    };
    let _ = unsafe { ioctl::drm::version(file.as_raw_fd(), &mut args) };
    println!("{args:?}");
    println!("name: {:?}", str::from_utf8(&driver_name[0..args.name_len]));
    println!("date: {:?}", str::from_utf8(&date[0..args.date_len]));
    println!("desc: {:?}", str::from_utf8(&desc[0..args.desc_len]));
}
