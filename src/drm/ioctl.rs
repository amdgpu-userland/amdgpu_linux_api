const DRM_IOCTL_BASE: u32 = 'd' as u32;
const DRM_COMMAND_BASE: u32 = 0x40;

/// Amdgpu specific
#[macro_use]
pub mod amd;
/// Common for all vendors
#[macro_use]
pub mod drm;
