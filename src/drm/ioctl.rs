const DRM_IOCTL_BASE: u32 = 'd' as u32;
const DRM_COMMAND_BASE: u32 = 0x40;

macro_rules! define_drm_ioctl {
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, $ioctl_direction:tt) => {
        define_ioctl!(
            $(#[$meta])*
            $fn_name,
            $args_ty,
            $num,
            DRM_IOCTL_BASE,
            $ioctl_direction
        );
    };
    ($(#[$meta:meta])* $fn_name:ident, $ioctl_num:expr) => {
        define_ioctl!(
            $(#[$meta])*
            $fn_name,
            $ioctl_num,
            DRM_IOCTL_BASE
        );
    };
}
macro_rules! define_amddrm_ioctl {
    ($(#[$meta:meta])* $fn_name:ident, $args_ty:ty, $num:literal, $ioctl_direction:tt) => {
        define_ioctl!(
            $(#[$meta])*
            $fn_name,
            $args_ty,
            DRM_COMMAND_BASE + $num,
            DRM_IOCTL_BASE,
            $ioctl_direction
        );
    };
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmVersion {
    /// Major version
    pub major: i32,
    /// Minor version
    pub minor: i32,
    /// Patch level
    pub patchlevel: i32,
    /// Length of name buffer
    pub name_len: usize,
    /// Name of driver
    pub name: *mut u8,
    /// Length of date buffer
    pub date_len: usize,
    /// User-space buffer to hold date
    pub date: *mut u8,
    /// Length of desc buffer
    pub desc_len: usize,
    /// User-space buffer to hold desc
    pub desc: *mut u8,
}
assert_layout!(DrmVersion, size = 64, align = 8);

define_drm_ioctl!(drm_ioctl_version, DrmVersion, 0x0, WR);

define_drm_ioctl!(drm_ioctl_set_master, 0x1e);
define_drm_ioctl!(drm_ioctl_drop_master, 0x1f);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmAmdgpuGemCreateIn {
    /** the requested memory size */
    pub bo_size: u64,
    /** physical start_addr alignment in bytes for some HW requirements */
    pub alignment: u64,
    /** the requested memory domains */
    pub domains: u64,
    /** allocation flags */
    pub domain_flags: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrmAmdgpuGemCreateOut {
    /** returned GEM object handle */
    pub handle: u32,
    pub _pad: u32,
}

#[repr(C)]
pub union DrmAmdgpuGemCreate {
    pub input: DrmAmdgpuGemCreateIn,
    pub output: DrmAmdgpuGemCreateOut,
}
assert_layout!(DrmAmdgpuGemCreate, size = 32, align = 8);
define_amddrm_ioctl!(
    /// Creates a new gem object
    ///
    /// The resulting Gem object doesn't have to have the parameters you set here.
    /// You need to check the gem's properties lates.
    ///
    /// For example it can move the allocation to gtt if there is not enought vram free
    amdgpu_ioctl_gem_create, DrmAmdgpuGemCreate, 0x0, WR);
