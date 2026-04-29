use std::os::fd::RawFd;

use crate::drm::GemHandle;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Version {
    pub major: i32,
    pub minor: i32,
    pub patchlevel: i32,
    pub name_len: usize,
    pub name: *mut u8,
    pub date_len: usize,
    pub date: *mut u8,
    pub desc_len: usize,
    pub desc: *mut u8,
}
assert_layout!(Version, size = 64, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Client {
    /// Set this to 0
    pub idx: i32,
    /// Is authenticated
    pub auth: i32,
    pub pid: u64,
    pub uid: u64,
    pub magic: u64,
    pub iocs: u64,
}
assert_layout!(Client, size = 40, align = 8);

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct PrimeHandle {
    pub handle: GemHandle,
    /// open() flags for dmabuf fd
    pub flags: u32,
    /// Returned dmabuf file descriptor
    pub fd: RawFd,
}
assert_layout!(PrimeHandle, size = 12, align = 4);

pub const CLIENT_NAME_MAX_LEN: usize = 64;
pub struct SetClientName {
    pub name_len: usize,
    pub name: *const u8,
}
assert_layout!(SetClientName, size = 16, align = 8);
