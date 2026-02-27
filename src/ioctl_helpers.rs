macro_rules! ioctl_param {
    (WR, $ioctl_base:expr, $num:expr, $args_ty:ty) => {
        libc::_IOWR::<$args_ty>($ioctl_base, $num)
    };
    (W, $ioctl_base:expr, $num:expr, $args_ty:ty) => {
        libc::_IOW::<$args_ty>($ioctl_base, $num)
    };
    (R, $ioctl_base:expr, $num:expr, $args_ty:ty) => {
        libc::_IOR::<$args_ty>($ioctl_base, $num)
    };
}

macro_rules! define_ioctl {
    ($(#[$meta:meta])* $fn_name:ident, $args_type:ty, $ioctl_num:expr, $ioctl_base:expr, $ioctl_direction:tt) => {
        $(#[$meta])*
        pub unsafe fn $fn_name(fd: libc::c_int, args: &mut $args_type) -> Result<(), libc::c_int> {
        let ptr: *mut $args_type = args;
        let res =
        unsafe { libc::ioctl(fd, ioctl_param!($ioctl_direction, $ioctl_base, $ioctl_num, $args_type), ptr) };
        if res != 0 {
        return Err( unsafe {* libc::__errno_location()});
        }
        Ok(())
        }
    };
}
