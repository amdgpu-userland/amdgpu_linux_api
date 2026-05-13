use std::{
    fs::File,
    mem::MaybeUninit,
    os::fd::{AsRawFd, OwnedFd},
};

use amdgpu_linux_api::drm::ioctl;

fn main() {
    let file = File::open("/dev/dri/renderD128").unwrap();
    let fd = OwnedFd::from(file);

    let mut dev_info: MaybeUninit<ioctl::amd::InfoDevice> = MaybeUninit::zeroed();
    let mut args = ioctl::amd::Info {
        return_pointer: dev_info.as_mut_ptr().cast(),
        return_size: size_of_val(&dev_info) as u32,
        query: ioctl::amd::InfoQuery::DevInfo,
        quick_info: MaybeUninit::uninit(),
    };
    let res = unsafe { ioctl::amd::info(fd.as_raw_fd(), &mut args) };
    res.unwrap();
    let dev_info = unsafe { dev_info.assume_init() };
    println!("{dev_info:#?}");
    let mut mem_info: MaybeUninit<ioctl::amd::InfoMemory> = MaybeUninit::zeroed();
    let mut args = ioctl::amd::Info {
        return_pointer: (&raw mut mem_info).cast(),
        return_size: size_of_val(&mem_info).try_into().unwrap(),
        query: ioctl::amd::InfoQuery::Memory,
        quick_info: MaybeUninit::uninit(),
    };
    let _ = unsafe { ioctl::amd::info(fd.as_raw_fd(), &mut args) }.unwrap();
    let mem_info = unsafe { mem_info.assume_init() };
    println!("{mem_info:#?}");

    println!(
        "TMZ: {}",
        dev_info.ids_flags & ioctl::amd::ids_flags::TMZ != 0
    );
    println!(
        "MCBP: {}",
        dev_info.ids_flags & ioctl::amd::ids_flags::PREEMPTION != 0
    );
    println!(
        "IsAPU: {}",
        dev_info.ids_flags & ioctl::amd::ids_flags::FUSION != 0
    );
    println!(
        "HasGangSubmit: {}",
        dev_info.ids_flags & ioctl::amd::ids_flags::GAMG_SUBMIT != 0
    );
    println!(
        "Trunc Coord Conformant Mode: {}",
        dev_info.ids_flags & ioctl::amd::ids_flags::CONFORMANT_TRUNC_COORD != 0
    );
    println!(
        "Virt Mode: {:?}",
        (dev_info.ids_flags >> ioctl::amd::ids_flags::MODE_SHIFT)
            & ioctl::amd::ids_flags::MODE_MASK
    );
    println!(
        "IsLargeBar: {}",
        mem_info.cpu_accessible_vram.total_heap_size == mem_info.vram.total_heap_size
    );
}
