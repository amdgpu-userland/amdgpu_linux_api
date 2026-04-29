use super::DRM_IOCTL_BASE;

mod structs;
pub use structs::*;

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
define_drm_ioctl!(
    ///
    /// # SAFETY
    /// todo
    version, Version, 0x0, WR);
define_drm_ioctl!(
    /// Almost deprecated
    ///
    /// if idx==0 it will populate some fields
    /// which you can use to easily determine if this client is authenticated
    /// EINVAL otherwise
    ///
    /// # SAFETY
    /// todo
    get_client, Client, 0x05, WR);
define_drm_ioctl!(
    ///
    /// # SAFETY
    /// todo
    set_master, 0x1e);
define_drm_ioctl!(
    ///
    /// # SAFETY
    /// todo
    drop_master, 0x1f);
define_drm_ioctl!(
    ///
    /// # SAFETY
    /// todo
    prime_handle_to_fd, PrimeHandle, 0x2d, WR);
define_drm_ioctl!(
    ///
    /// # SAFETY
    /// todo
    prime_fd_to_handle, PrimeHandle, 0x2e, WR);

define_drm_ioctl!(
    /// Attach a name to a drm_file
    ///
    /// Having a name allows for easier tracking and debugging.
    ///
    /// # SAFETY
    /// The length of the name (without null ending char) must be
    /// <= DRM_CLIENT_NAME_MAX_LEN.
    /// The call will fail if the name contains whitespaces or non-printable chars.
    set_client_name, SetClientName, 0xD1, WR);
