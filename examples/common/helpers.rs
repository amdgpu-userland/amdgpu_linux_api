#![allow(dead_code)]

use std::{
    fs::File,
    io,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::fs::OpenOptionsExt,
    },
};

pub struct RawRenderNode(File);

impl RawRenderNode {
    pub fn open(minor: u32) -> io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open(format!("/dev/dri/renderD{minor}"))?;
        Ok(Self(file))
    }
}

impl AsFd for RawRenderNode {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

unsafe impl amdgpu_linux_api::drm::DrmFile for RawRenderNode {}
unsafe impl amdgpu_linux_api::drm::DrmRenderFile for RawRenderNode {}
unsafe impl amdgpu_linux_api::drm::AmdgpuDrmFile for RawRenderNode {}

pub struct RawPrimaryNode(File);

impl RawPrimaryNode {
    pub fn open(minor: u32) -> io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .custom_flags(libc::O_CLOEXEC)
            .open(format!("/dev/dri/card{minor}"))?;
        Ok(Self(file))
    }
}

impl AsFd for RawPrimaryNode {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

unsafe impl amdgpu_linux_api::drm::DrmFile for RawPrimaryNode {}
unsafe impl amdgpu_linux_api::drm::DrmPrimaryFile for RawPrimaryNode {}
unsafe impl amdgpu_linux_api::drm::AmdgpuDrmFile for RawPrimaryNode {}
