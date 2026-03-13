use super::DRM_COMMAND_BASE;
use super::DRM_IOCTL_BASE;

mod structs;
pub use structs::*;

pub type CtxId = u32;
pub type BoListHandle = u32;

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

define_amddrm_ioctl!(
    /// Creates a new gem object
    ///
    /// The resulting Gem object doesn't have to have the parameters you set here.
    /// You need to check the gem's properties lates.
    ///
    /// For example it can move the allocation to gtt if there is not enought vram free
    gem_create, GemCreate, 0x00, WR);
define_amddrm_ioctl!(gem_mmap, GemMmap, 0x01, WR);
define_amddrm_ioctl!(ctx, Ctx, 0x02, WR);
define_amddrm_ioctl!(bo_list, BoList, 0x03, WR);
define_amddrm_ioctl!(info, Info, 0x05, W);
define_amddrm_ioctl!(gem_metadata, GemMetadata, 0x06, WR);
define_amddrm_ioctl!(gem_va, GemVa, 0x08, W);
