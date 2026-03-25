#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(const_index)]
// This is too broken to be used yet
//#![feature(guard_patterns)]
// // For expanded macro debugging
// #![feature(panic_internals)]
// #![feature(fmt_helpers_for_derive)]
// #![feature(derive_clone_copy)]
// #![feature(trivial_clone)]

#[macro_use]
mod ioctl_helpers;

#[macro_export]
macro_rules! GPU_PAGE_SIZE {
    () => {
        4096
    };
}

/// Think OpenGL and Vulkan
pub mod drm;

/// Bindings for Kernel Fusion Driver
/// the thing Radeon Open CoMpute (ROCM) is built on
///
/// The entry point are versioned Kfd stucts.
///
/// Pick one to use as your application minimal version requirement.
/// The assumption is Linux never breaks userspace
/// so future versions should be backwards compatible.
///
/// Keep in mind old Kfd versioned structs use
/// implementation based on the newest kernel kfd code.
/// Which shouldn't matter if amdgpu code didn't
/// break userspace between versions.
pub mod kfd;

pub mod sdma;

/// Defines packets for different SDMA versions
///
/// These packets are then to be writen to an indirect buffer or a ring buffer
/// and read by the gpu.
///
/// You can also decode them back
pub mod sdma_new;
