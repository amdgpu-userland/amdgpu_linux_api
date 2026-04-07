use amdgpu_linux_api::drm::{AmdgpuDrmRender3_64, ioctl};
use std::{
    mem::MaybeUninit,
    os::fd::{AsFd, AsRawFd},
};

fn main() {
    let drm = AmdgpuDrmRender3_64::open(128).unwrap();
    let fd = drm.as_fd();
    let mut info = ioctl::amd::InfoHwIp::default();
    let mut args = ioctl::amd::Info {
        return_pointer: (&raw mut info).cast(),
        return_size: size_of_val(&info).try_into().unwrap(),
        query: ioctl::amd::InfoQuery::HwIpInfo,
        quick_info: MaybeUninit::new(ioctl::amd::InfoUnion {
            query_hw_ip: ioctl::amd::QueryHwIp {
                r#type: ioctl::amd::HwIp::DMA,
                ip_instance: 0,
            },
        }),
    };
    if let Err(res) = unsafe { ioctl::amd::info(fd.as_raw_fd(), &mut args) } {
        todo!("info hw_ip: {res}")
    }
    println!("{info:#?}");
    println!("Ip discovery: {:#x}", info.ip_discovery_version);

    let mut count = 0u32;
    let mut args = ioctl::amd::Info {
        return_pointer: (&raw mut count).cast(),
        return_size: size_of_val(&count).try_into().unwrap(),
        query: ioctl::amd::InfoQuery::HwIpCount,
        quick_info: MaybeUninit::new(ioctl::amd::InfoUnion {
            query_hw_ip: ioctl::amd::QueryHwIp {
                r#type: ioctl::amd::HwIp::DMA,
                ip_instance: 0,
            },
        }),
    };
    if let Err(res) = unsafe { ioctl::amd::info(fd.as_raw_fd(), &mut args) } {
        todo!("info hw_ip_count: {res}")
    }
    println!("{count}")
}
