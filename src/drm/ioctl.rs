use std::ffi::c_void;

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
#[derive(Debug, Default, Clone, Copy)]
pub struct DrmClient {
    /// Set this to 0
    pub idx: i32,
    /// Is authenticated
    pub auth: i32,
    pub pid: u64,
    pub uid: u64,
    pub magic: u64,
    pub iocs: u64,
}
assert_layout!(DrmClient, size = 40, align = 8);
define_drm_ioctl!(
    /// Almost deprecated
    ///
    /// if idx==0 it will populate some fields
    /// which you can use to easily determine if this client is authenticated
    /// EINVAL otherwise
    drm_ioctl_get_client, DrmClient, 0x05, WR);

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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModeCrtc {
    pub id: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QueryHwIp {
    pub r#type: u32,

    /// Index of the IP if there are more IPs of the same
    /// type. Ignored by AMDGPU_INFO_HW_IP_COUNT.
    pub ip_instance: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ReadMMReag {
    pub dword_offset: u32,
    /** number of registers to read */
    pub count: u32,
    pub instance: u32,
    /** For future use, no flags defined so far */
    pub flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct QueryFw {
    /** AMDGPU_INFO_FW_* */
    pub fw_type: u32,

    /// Index of the IP if there are more IPs of
    /// the same type.
    pub ip_instance: u32,
    /// Index of the engine. Whether this is used depends
    /// on the firmware type. (e.g. MEC, SDMA)
    pub index: u32,
    pub _pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VbiosInfo {
    pub r#type: u32,
    pub offset: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SensorInfo {
    pub r#type: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VideoCap {
    pub r#type: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union drm_amdgpu_info_quick_info {
    mode_crtc: ModeCrtc,
    query_hw_ip: QueryHwIp,
    read_mmr_reg: ReadMMReag,
    query_fw: QueryFw,
    vbios_info: VbiosInfo,
    sensor_info: SensorInfo,
    video_cap: VideoCap,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DrmAmdgpuInfo {
    /* Where the return value will be stored */
    return_pointer: *mut c_void,
    /* The size of the return value. Just like "size" in "snprintf",
     * it limits how many bytes the kernel can write. */
    return_size: u32,
    /* The query request id. */
    query: u32,
    quick_info: drm_amdgpu_info_quick_info,
}
define_amddrm_ioctl!(amdgpu_ioctl_info, (), 0x05, W);
