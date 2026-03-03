#[macro_use]
mod ioctl_helpers;

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
