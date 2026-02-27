#[macro_use]
mod ioctl_helpers;

/// Think OpenGL and Vulkan
pub mod drm {
    pub mod ioctl;
}

/// Bindings for Kernel Fusion Driver
/// the thing Radeon Open CoMpute (ROCM) is built on
pub mod kfd;
