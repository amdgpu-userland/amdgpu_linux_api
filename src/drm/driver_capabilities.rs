pub type Cap = u64;

/// If set to 1, the driver supports creating dumb buffers via the
/// &DRM_IOCTL_MODE_CREATE_DUMB ioctl.
pub const CAP_DUMB_BUFFER: Cap = 0x1;

/// If set to 1, the kernel supports specifying a CRTC index
/// in the high bits of &drm_wait_vblank_request.type.
///
/// Starting kernel version 2.6.39, this capability is always set to 1.
pub const CAP_VBLANK_HIGH_CRTC: Cap = 0x2;

/// The preferred bit depth for dumb buffers.
///
/// The bit depth is the number of bits used to indicate the color of a single
/// pixel excluding any padding. This is different from the number of bits per
/// pixel. For instance, XRGB8888 has a bit depth of 24 but has 32 bits per
/// pixel.
///
/// Note that this preference only applies to dumb buffers, it's irrelevant for
/// other types of buffers.
pub const CAP_DUMB_PREFERRED_DEPTH: Cap = 0x3;

/// If set to 1, the driver prefers userspace to render to a shadow buffer
/// instead of directly rendering to a dumb buffer. For best speed, userspace
/// should do streaming ordered memory copies into the dumb buffer and never
/// read from it.
///
/// Note that this preference only applies to dumb buffers, it's irrelevant for
/// other types of buffers.
pub const CAP_DUMB_PREFER_SHADOW: Cap = 0x4;

/// Bitfield of supported PRIME sharing capabilities. See CAP_PRIME_IMPORT
/// and CAP_PRIME_EXPORT.
///
/// Starting from kernel version 6.6, both CAP_PRIME_IMPORT and
/// CAP_PRIME_EXPORT are always advertised.
///
/// PRIME buffers are exposed as dma-buf file descriptors.
pub const CAP_PRIME: Cap = 0x5;

/// If this bit is set in CAP_PRIME, the driver supports importing PRIME
/// buffers via the &DRM_IOCTL_PRIME_FD_TO_HANDLE ioctl.
///
/// Starting from kernel version 6.6, this bit is always set in CAP_PRIME.
pub const CAP_PRIME_IMPORT: Cap = 0x1;

/// If this bit is set in CAP_PRIME, the driver supports exporting PRIME
/// buffers via the &DRM_IOCTL_PRIME_HANDLE_TO_FD ioctl.
///
/// Starting from kernel version 6.6, this bit is always set in CAP_PRIME.
pub const CAP_PRIME_EXPORT: Cap = 0x2;

/// If set to 0, the kernel will report timestamps with CLOCK_REALTIME in
/// struct drm_event_vblank. If set to 1, the kernel will report timestamps with
/// CLOCK_MONOTONIC.
///
/// Starting from kernel version 2.6.39, the default value for this capability
/// is 1. Starting kernel version 4.15, this capability is always set to 1.
pub const CAP_TIMESTAMP_MONOTONIC: Cap = 0x6;

/// If set to 1, the driver supports &DRM_MODE_PAGE_FLIP_ASYNC for legacy
/// page-flips.
pub const CAP_ASYNC_PAGE_FLIP: Cap = 0x7;

/// The CURSOR_WIDTH and CURSOR_HEIGHT capabilities return a valid
/// width x height combination for the hardware cursor. The intention is that a
/// hardware agnostic userspace can query a cursor plane size to use.
///
/// Note that the cross-driver contract is to merely return a valid size;
/// drivers are free to attach another meaning on top, eg. i915 returns the
/// maximum plane size.
pub const CAP_CURSOR_WIDTH: Cap = 0x8;

/// See CAP_CURSOR_WIDTH.
pub const CAP_CURSOR_HEIGHT: Cap = 0x9;

/// If set to 1, the driver supports supplying modifiers in the
/// &DRM_IOCTL_MODE_ADDFB2 ioctl.
pub const CAP_ADDFB2_MODIFIERS: Cap = 0x10;

/// If set to 1, the driver supports the &DRM_MODE_PAGE_FLIP_TARGET_ABSOLUTE and
/// &DRM_MODE_PAGE_FLIP_TARGET_RELATIVE flags in
/// &drm_mode_crtc_page_flip_target.flags for the &DRM_IOCTL_MODE_PAGE_FLIP
/// ioctl.
pub const CAP_PAGE_FLIP_TARGET: Cap = 0x11;

/// If set to 1, the kernel supports reporting the CRTC ID in
/// &drm_event_vblank.crtc_id for the &DRM_EVENT_VBLANK and
/// &DRM_EVENT_FLIP_COMPLETE events.
///
/// Starting kernel version 4.12, this capability is always set to 1.
pub const CAP_CRTC_IN_VBLANK_EVENT: Cap = 0x12;

/// If set to 1, the driver supports sync objects.
pub const CAP_SYNCOBJ: Cap = 0x13;

/// If set to 1, the driver supports timeline operations on sync objects.
pub const CAP_SYNCOBJ_TIMELINE: Cap = 0x14;

/// If set to 1, the driver supports &DRM_MODE_PAGE_FLIP_ASYNC for atomic
/// commits.
pub const CAP_ATOMIC_ASYNC_PAGE_FLIP: Cap = 0x15;

/// Driver capability for modesetting
pub trait Modeset {}
