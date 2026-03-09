use amdgpu_linux_api::kfd::ioctl;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;

fn main() {
    let file = std::fs::File::open("/dev/kfd").unwrap();

    let mut args = ioctl::GetProcessAperturesArgs::default();
    #[allow(deprecated)]
    let _ = unsafe { ioctl::get_process_apertures(file.as_raw_fd(), &mut args) };

    let gpu_id = args.process_apertures[0].gpu_id;

    let mut args = ioctl::SmiEventsArgs {
        gpuid: gpu_id,
        anon_fd: 0,
    };
    let res = unsafe { ioctl::smi_events(file.as_raw_fd(), &mut args) };

    assert!(res.is_ok());
    let smi_fd = unsafe { OwnedFd::from_raw_fd(args.anon_fd) };

    //use amdgpu_linux_api::kfd::ioctl::smi_event::*;
    //let flags = msk(VMFAULT) | msk(PROCESS_START) | msk(PROCESS_END) | msk(ALL_PROCESS);
    let flags = u64::MAX;
    let bytes = flags.to_ne_bytes();
    let _ = unsafe {
        libc::write(
            smi_fd.as_raw_fd(),
            bytes.as_ptr() as *const libc::c_void,
            bytes.len(),
        )
    };

    let mut args: libc::stat64 = unsafe { std::mem::zeroed() };
    let _ = unsafe { libc::fstat64(smi_fd.as_raw_fd(), &raw mut args) };
    println!("{args:#?}");
    loop {
        let mut fds = [libc::pollfd {
            fd: smi_fd.as_raw_fd(),
            events: 1,
            revents: 0,
        }];
        let res = unsafe { libc::poll(fds.as_mut_ptr(), 1, -1) };
        if res < 0 {
            break;
        }
        if res > 0 {
            let mut buff = [0u8; ioctl::SMI_EVENT_MSG_SIZE];
            let res = unsafe {
                libc::read(
                    smi_fd.as_raw_fd(),
                    buff.as_mut_ptr() as *mut libc::c_void,
                    ioctl::SMI_EVENT_MSG_SIZE,
                )
            };
            assert!(res > 0);
            let res = res as usize;

            let res = str::from_utf8(&buff[..res]).unwrap();
            println!("{res}");
        }
    }
}
