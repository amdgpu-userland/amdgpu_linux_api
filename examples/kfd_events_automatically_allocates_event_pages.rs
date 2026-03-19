use amdgpu_linux_api::kfd::Kfd1_18;
use amdgpu_linux_api::kfd::ioctl::{CreateEventArgs, create_event, event_type};
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;

fn main() {
    let kfd = Kfd1_18::open().unwrap();
    let fd = kfd.as_fd().as_raw_fd();
    let mut args = CreateEventArgs {
        event_page_offset: 0,
        event_trigger_data: 0,
        event_type: event_type::SIGNAL,
        auto_reset: 0,
        node_id: 0,
        event_id: 0,
        event_slot_index: 0,
    };
    let res = unsafe { create_event(fd, &mut args) };
    assert!(res.is_ok());
}
