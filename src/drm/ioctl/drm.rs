#![expect(clippy::missing_safety_doc)]
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
// TODO: Verify these comments

define_drm_ioctl!(
    /// Query DRM driver metadata strings and version numbers.
    ///
    /// Returns `Ok(())` and fills `major`, `minor`, `patchlevel`, and the
    /// string lengths. If a string buffer is non-null and has non-zero length,
    /// the kernel copies up to that many bytes.
    ///
    /// # SAFETY
    /// * `name`, `date`, and `desc` must be null or writable for their input
    ///   length fields, or this returns `EFAULT`.
    /// * Available to render clients.
    version, Version, 0x0, WR);
define_drm_ioctl!(
    /// Get the legacy DRM unique bus/device string.
    /// Don't rely on this.
    ///
    /// Returns `Ok(())` on success. The kernel updates `unique_len` with the
    /// full string length and copies into `unique` only when the provided
    /// buffer is large enough. Returns `EFAULT` if the destination pointer is
    /// invalid.
    ///
    /// # SAFETY
    /// * `unique` must be null or writable for `unique_len` bytes, or this
    ///   returns `EFAULT`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    get_unique, Unique, 0x01, WR);
define_drm_ioctl!(
    /// Get this DRM file's authentication magic.
    ///
    /// Returns `Ok(())` and fills `magic`.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Internal magic allocation can fail with allocation/id allocator errors
    ///   such as `ENOMEM`.
    get_magic, Auth, 0x2, R);
define_drm_ioctl!(
    /// Almost deprecated
    ///
    /// if idx==0 it will populate some fields
    /// which you can use to easily determine if this client is authenticated
    /// EINVAL otherwise
    ///
    /// # SAFETY
    /// * `idx` must be 0, or this returns `EINVAL`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    get_client, Client, 0x05, WR);
define_drm_ioctl!(
    /// Returns drm driver and device driver versions
    /// regardless of input values
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * `drm_major` must be `-1` or match the kernel DRM interface major,
    ///   or this returns `EINVAL`.
    /// * `drm_minor` must be in the supported range when `drm_major` is
    ///   not `-1`, or this returns `EINVAL`.
    /// * `device_major` must be `-1` or match the driver major version, or this
    ///   returns `EINVAL`.
    /// * `device_minor` must be in the supported driver range when
    ///   `device_major` is not `-1`, or this returns `EINVAL`.
    /// * Requesting DRM interface minor >= 1 initializes the legacy bus id and
    ///   can return bus-id allocation errors such as `ENOMEM`.
    /// * The driver version fields and bus-id setup are driver-provided; a
    ///   driver can affect returned values and errors.
    set_version, SetVersion, 0x07, WR);

define_drm_ioctl!(
    /// Destroy the gem handle, if the underlying object has no more references it gets released
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support the GEM subsystem, or this returns `EOPNOTSUPP`.
    /// * `handle` must exist for this file, or this returns `EINVAL`.
    gem_close, GemClose, 0x09, W);
define_drm_ioctl!(
    /// Create a uniqe token (name) for this device which can be used by other clients to import this
    /// object.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * Driver must support the GEM subsystem, or this returns `EOPNOTSUPP`.
    /// * `handle` must exist for this file, or this returns `ENOENT`.
    /// * A global name id must be available, or this returns `ENOSPC`.
    gem_flink, GemFlink, 0x0a, WR);
define_drm_ioctl!(
    /// Imports a gem object via provided flink name.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * Driver must support the GEM subsystem, or this returns `EOPNOTSUPP`.
    /// * `name` must exist, or this returns `ENOENT`.
    /// * A per-file handle id must be available, or this returns `ENOSPC`.
    /// * Internal allocation can fail with `ENOMEM`.
    /// * Opening client must be different from the exporting client or this ioctl will return the
    /// same gem handle and the program must ensure this handle is closed (`gem_close`) at most once
    gem_open, GemOpen, 0x0b, WR);
define_drm_ioctl!(
    /// Query a DRM capability.
    ///
    /// Returns `Ok(())` and writes `value`.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Unknown capabilities return `EINVAL`.
    /// * Capabilities which require modesetting return `EOPNOTSUPP` when the
    ///   driver does not support modesetting.
    /// * Returned values come from core state and driver mode configuration; a
    ///   driver can affect returned values.
    get_cap, GetCap, 0x0c, WR);
define_drm_ioctl!(
    /// Set a per-file DRM client capability.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Capability must be from the valid set, or this returns `EINVAL`.
    /// * Boolean capabilities must have value 0 or 1, or this returns `EINVAL`.
    /// * Atomic capability requires an atomic-capable driver, or this returns
    ///   `EOPNOTSUPP`.
    /// * Writeback and cursor-hotspot capabilities require atomic to be enabled,
    ///   or this returns `EINVAL`.
    /// * Cursor hotspot also requires driver support, or this returns
    ///   `EOPNOTSUPP`.
    set_client_cap, SetClientCap, 0x0d, W);
define_drm_ioctl!(
    /// Authenticate another DRM file by magic value.
    ///
    /// Returns `Ok(())` when a matching magic is found and authenticated.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * `magic` must have been created by `get_magic`, or this returns
    ///   `EINVAL`.
    auth_magic, Auth, 0x11, W);
define_drm_ioctl!(
    /// Become DRM master for this device.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * The file must be allowed to become master; otherwise this returns
    ///   `EACCES` or driver/core master-acquire errors.
    /// * Master acquisition can call driver hooks; a driver can affect returned
    ///   errors.
    set_master, 0x1e);
define_drm_ioctl!(
    /// Drop DRM master status for this file.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * The file must currently be master, or this returns `EINVAL`/`EACCES`
    ///   from the core master checks.
    /// * Master drop can call driver hooks; a driver can affect returned errors.
    drop_master, 0x1f);
define_drm_ioctl!(
    /// Export a GEM handle to a DMA-BUF fd.
    ///
    /// Returns `Ok(())` and fills `fd` with a new file descriptor.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * `flags` must be valid open flags for the exported fd, or this returns
    ///   `EINVAL`.
    /// * `handle` must exist and be exportable, or this returns `ENOENT` or a
    ///   driver-specific error.
    /// * Allocating the new fd/export can fail with errors such as `EMFILE` or
    ///   `ENOMEM`.
    /// * Export is driver-backed; a driver can affect returned errors.
    prime_handle_to_fd, PrimeHandle, 0x2d, WR);
define_drm_ioctl!(
    /// Import a DMA-BUF fd to a GEM handle.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * `fd` must be a valid DMA-BUF fd, or this returns `EINVAL` or fd lookup
    ///   errors.
    /// * The DMA-BUF must be importable by the driver, or this returns a
    ///   driver-specific error.
    /// * Creating a handle can fail with errors such as `EMFILE` or `ENOMEM`.
    /// * If the current client already has a handle for this object it is returned and the program
    /// must ensure it is closed at most one time.
    /// * Import is driver-backed; a driver can affect returned values and
    ///   errors.
    prime_fd_to_handle, PrimeHandle, 0x2e, WR);

define_drm_ioctl!(
    /// Query the current scanout sequence number for a CRTC.
    ///
    /// Returns whether the CRTC is active, the current sequence number and its
    /// timestamp.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Device must support vblank, or this returns `EOPNOTSUPP`.
    /// * `crtc_id` must name a valid CRTC visible to this file, or this returns
    ///   `ENOENT`.
    /// * Acquiring the vblank counter can return vblank-core errors such as
    ///   `EINVAL`.
    crtc_get_sequence, CrtcGetSequence, 0x3b, WR);
define_drm_ioctl!(
    /// Queue an event to be delivered at a CRTC sequence.
    ///
    /// Returns `Ok(())` after queuing the event and updates `sequence` to the
    /// actual target sequence.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Device must support vblank, or this returns `EOPNOTSUPP`.
    /// * `crtc_id` must name a valid CRTC visible to this file, or this returns
    ///   `ENOENT`.
    /// * `flags` may only contain `RELATIVE` and `NEXT_ON_MISS`, or this returns
    ///   `EINVAL`.
    /// * Event allocation can fail with `ENOMEM`.
    /// * Acquiring the vblank counter can return vblank-core errors such as
    ///   `EINVAL`.
    crtc_queue_sequence, CrtcQueueSequence, 0x3c, WR);

define_drm_ioctl!(
    /// Get global KMS resource IDs and framebuffer size limits.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Array pointers must be null or writable for their count fields, or
    ///   this returns `EFAULT`.
    /// * Leased clients only see leased objects.
    /// * Returned limits and object lists come from driver mode configuration;
    ///   a driver can affect returned values.
    mode_getresources, ModeCardRes, 0xA0, WR);
define_drm_ioctl!(
    /// Get CRTC state.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `crtc_id` must name a valid CRTC visible to this file, or this returns
    ///   `ENOENT`.
    /// * Returned CRTC state comes from driver mode configuration; a driver can
    ///   affect returned values.
    mode_getcrtc, ModeCrtc, 0xA1, WR);
define_drm_ioctl!(
    /// Set CRTC state.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * CRTC, framebuffer and connector IDs must be valid and visible to this
    ///   file, or this returns `ENOENT`.
    /// * Connector array must be readable for `count_connectors` entries, or
    ///   this returns `EFAULT`.
    /// * Mode data and coordinates must pass core and driver validation, or this
    ///   returns `EINVAL`.
    /// * Modeset is driver-backed; a driver can affect returned errors.
    mode_setcrtc, ModeCrtc, 0xA2, WR);
define_drm_ioctl!(
    /// Move and/or replace a legacy cursor image.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting/cursor updates, or this returns
    ///   `EOPNOTSUPP`.
    /// * `flags` must be within `DRM_MODE_CURSOR_FLAGS`, or this returns
    ///   `EINVAL`.
    /// * `crtc_id` must name a valid CRTC visible to this file, or this returns
    ///   `ENOENT`.
    /// * Cursor handle, size and position are driver-validated; a driver can
    ///   affect returned errors.
    mode_cursor, ModeCursor, 0xA3, WR);
define_drm_ioctl!(
    /// Read a CRTC gamma table.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `crtc_id` must name a valid CRTC visible to this file, or this returns
    ///   `ENOENT`.
    /// * `gamma_size` must match the CRTC gamma size, or this returns `EINVAL`.
    /// * Color pointers must reference writable arrays of `gamma_size` u16
    ///   entries, or this returns `EFAULT`.
    /// * Gamma readback uses driver state; a driver can affect returned values
    ///   and errors.
    mode_getgamma, ModeCrtcLut, 0xA4, WR);
define_drm_ioctl!(
    /// Set a CRTC gamma table.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Driver must provide legacy gamma hooks, or this returns `ENOSYS`.
    /// * `crtc_id` must name a valid CRTC visible to this file, or this returns
    ///   `ENOENT`.
    /// * `gamma_size` must match the CRTC gamma size, or this returns `EINVAL`.
    /// * Color pointers must reference readable arrays of `gamma_size` u16
    ///   entries, or this returns `EFAULT`.
    /// * Gamma programming is driver-backed; a driver can affect returned
    ///   errors.
    mode_setgamma, ModeCrtcLut, 0xA5, WR);
define_drm_ioctl!(
    /// Get encoder metadata.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `encoder_id` must name a valid encoder visible to this file, or this
    ///   returns `ENOENT`.
    /// * Returned metadata comes from driver mode configuration; a driver can
    ///   affect returned values.
    mode_getencoder, ModeGetEncoder, 0xA6, WR);
define_drm_ioctl!(
    /// Get connector metadata and associated modes/properties/encoders.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `connector_id` must name a valid connector visible to this file, or
    ///   this returns `ENOENT`.
    /// * Array pointers must be null or writable for their count fields, or
    ///   this returns `EFAULT`.
    /// * Force-probing requires current DRM master, or this returns `EACCES`.
    /// * Connector probing and mode lists are driver-backed; a driver can affect
    ///   returned values and errors.
    mode_getconnector, ModeGetConnector, 0xA7, WR);
define_drm_ioctl!(
    /// Get KMS property metadata.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `prop_id` must name a property, or this returns `ENOENT`.
    /// * Pointers must be null or writable for their count fields, or this
    ///   returns `EFAULT`.
    mode_getproperty, ModeGetProperty, 0xAA, WR);
define_drm_ioctl!(
    /// Set a legacy connector property.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Connector/property IDs and value must be valid, or this returns
    ///   `ENOENT`/`EINVAL`.
    /// * Property setting can call driver hooks; a driver can affect returned
    ///   errors.
    mode_setproperty, ModeConnectorSetProperty, 0xAB, WR);
define_drm_ioctl!(
    /// Read a KMS property blob.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `blob_id` must name a blob, or this returns `ENOENT`.
    /// * `data` must be null or writable for `length` bytes, or this returns
    ///   `EFAULT`.
    mode_getpropblob, ModeGetBlob, 0xAC, WR);
define_drm_ioctl!(
    /// Get legacy framebuffer metadata.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `fb_id` must name a framebuffer visible to this file, or this returns
    ///   `ENOENT`.
    /// * DRM master or CAP_SYS_ADMIN may receive a fresh GEM handle which must
    ///   be closed.
    /// * Handle lookup/creation can return GEM/driver errors.
    mode_getfb, ModeFbCmd, 0xAD, WR);
define_drm_ioctl!(
    /// Add a legacy single-plane framebuffer.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Dimensions, depth, bpp and pitch must pass core validation, or this
    ///   returns `EINVAL`/`ERANGE`.
    /// * `handle` must name a valid GEM object accepted by the driver, or this
    ///   returns `ENOENT`/driver errors.
    /// * Framebuffer allocation/id allocation can fail with `ENOMEM`/`ENOSPC`.
    /// * Framebuffer creation is driver-backed; a driver can affect returned
    ///   errors.
    mode_addfb, ModeFbCmd, 0xAE, WR);
define_drm_ioctl!(
    /// Remove a framebuffer and disable users if necessary.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Framebuffer id must be non-zero, or this returns `EINVAL`.
    /// * Framebuffer id must be visible to this file, or this returns `ENOENT`.
    /// * Removing an in-use framebuffer can trigger driver modeset/plane hooks;
    ///   a driver can affect returned errors.
    mode_rmfb, ModeRmfb, 0xAF, WR);
define_drm_ioctl!(
    /// Schedule a legacy page flip.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting/page flips, or this returns
    ///   `EOPNOTSUPP`.
    /// * `reserved` and `flags` must be valid, or this returns `EINVAL`.
    /// * `crtc_id` and `fb_id` must name visible objects, or this returns
    ///   `ENOENT`.
    /// * Leased files must own the CRTC, or this returns `EACCES`.
    /// * Returns `EBUSY` if a flip is already pending.
    /// * Page flip execution is driver-backed; a driver can affect returned
    ///   errors.
    mode_page_flip, ModeCrtcPageFlip, 0xB0, WR);
define_drm_ioctl!(
    /// Mark framebuffer regions dirty.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting and dirtyfb, or this returns
    ///   `EOPNOTSUPP`.
    /// * `fb_id` must name a framebuffer, or this returns `ENOENT`.
    /// * Dirty flags and clip counts must be valid, or this returns `EINVAL`.
    /// * `clips_ptr` must be readable for `num_clips` clip rectangles, or this
    ///   returns `EFAULT`.
    /// * Dirty handling is driver-backed; a driver can affect returned errors.
    mode_dirtyfb, ModeFbDirtyCmd, 0xB1, WR);
define_drm_ioctl!(
    /// Create a KMS dumb buffer object.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support dumb buffer creation, or this returns
    ///   `EOPNOTSUPP`.
    /// * `flags` must be zero, or this returns `EINVAL`.
    /// * Dimensions and bpp must be accepted by the driver.
    /// * Dumb buffer allocation is driver-backed; a driver can affect returned
    ///   values and errors.
    mode_create_dumb, ModeCreateDumb, 0xB2, WR);
define_drm_ioctl!(
    /// Get a fake mmap offset for a dumb buffer.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support dumb mmap, or this returns `EOPNOTSUPP`.
    /// * `handle` must name a valid dumb buffer for this file, or this returns
    ///   `ENOENT`/`EINVAL`.
    /// * Mmap-offset creation is driver-backed; a driver can affect returned
    ///   values and errors.
    mode_map_dumb, ModeMapDumb, 0xB3, WR);
define_drm_ioctl!(
    /// Destroy a KMS dumb buffer object.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * `handle` must name a valid dumb buffer for this file, or this returns
    ///   `EINVAL`/`ENOENT`.
    /// * Destruction is driver/GEM-backed; a driver can affect returned errors.
    mode_destroy_dumb, ModeDestroyDumb, 0xB4, WR);
define_drm_ioctl!(
    /// Get plane object IDs.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting and universal planes, or this returns
    ///   `EOPNOTSUPP`.
    /// * `plane_id_ptr` must be null or writable for `count_planes` object IDs,
    ///   or this returns `EFAULT`.
    /// * Returned plane list comes from driver mode configuration; a driver can
    ///   affect returned values.
    mode_getplaneresources, ModeGetPlaneRes, 0xB5, WR);
define_drm_ioctl!(
    /// Get plane metadata.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting and universal planes, or this returns
    ///   `EOPNOTSUPP`.
    /// * `plane_id` must name a valid plane visible to this file, or this
    ///   returns `ENOENT`.
    /// * `format_type_ptr` must be null or writable for `count_format_types`
    ///   values, or this returns `EFAULT`.
    /// * Returned plane state and formats come from driver mode configuration; a
    ///   driver can affect returned values.
    mode_getplane, ModeGetPlane, 0xB6, WR);
define_drm_ioctl!(
    /// Set a plane's framebuffer and source/destination rectangles.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting and universal planes, or this returns
    ///   `EOPNOTSUPP`.
    /// * Plane, CRTC and framebuffer IDs must be valid and visible, or this
    ///   returns `ENOENT`.
    /// * Flags and source/destination rectangles must be valid, or this returns
    ///   `EINVAL`/`ERANGE`.
    /// * Plane update is driver-backed; a driver can affect returned errors.
    mode_setplane, ModeSetPlane, 0xB7, WR);
define_drm_ioctl!(
    /// Add a multi-plane framebuffer.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Format, dimensions, flags, handles, pitches, offsets and modifiers
    ///   must pass validation, or this returns `EINVAL`/`ERANGE`/`ENOENT`.
    /// * Modifier use requires driver modifier support, or this returns
    ///   `EOPNOTSUPP`.
    /// * Framebuffer creation is driver-backed; a driver can affect returned
    ///   errors.
    mode_addfb2, ModeFbCmd2, 0xB8, WR);
define_drm_ioctl!(
    /// Get properties attached to a KMS object.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Object id/type must be valid and visible, or this returns `ENOENT`.
    /// * Pointers must be null or writable for `count_props` entries, or this
    ///   returns `EFAULT`.
    /// * Returned properties come from driver mode configuration; a driver can
    ///   affect returned values.
    mode_obj_getproperties, ModeObjGetProperties, 0xB9, WR);
define_drm_ioctl!(
    /// Set a property on a KMS object.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * Object/property IDs and value must be valid and writable by this
    ///   client, or this returns `ENOENT`/`EINVAL`.
    /// * Property setting can call driver hooks; a driver can affect returned
    ///   errors.
    mode_obj_setproperty, ModeObjSetProperty, 0xBA, WR);
define_drm_ioctl!(
    /// Move and/or replace a cursor image with hotspot coordinates.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Same requirements and core errors as `mode_cursor`.
    /// * Hotspot coordinates are driver-validated; a driver can affect returned
    ///   errors.
    mode_cursor2, ModeCursor2, 0xBB, WR);
define_drm_ioctl!(
    /// Perform or test an atomic KMS commit.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support atomic modesetting, or this returns `EOPNOTSUPP`.
    /// * Client must enable atomic capability first, or this returns `EINVAL`.
    /// * Flags must be within `DRM_MODE_ATOMIC_FLAGS`, or this returns `EINVAL`.
    /// * Object/property/value arrays must be readable and internally
    ///   consistent, or this returns `EFAULT`/`EINVAL`/`ENOENT`.
    /// * Atomic check/commit is driver-backed; a driver can affect returned
    ///   errors.
    mode_atomic, ModeAtomic, 0xBC, WR);
define_drm_ioctl!(
    /// Create a user property blob.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `data` must be readable for `length` bytes, or this returns `EFAULT`.
    /// * Blob allocation/id allocation can fail with `ENOMEM`.
    mode_createpropblob, ModeCreateBlob, 0xBD, WR);
define_drm_ioctl!(
    /// Destroy a user property blob.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `blob_id` must name a user-created blob owned by this file, or this
    ///   returns `ENOENT`.
    mode_destroypropblob, ModeDestroyBlob, 0xBE, WR);

define_drm_ioctl!(
    /// Create a DRM sync object.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobjs, or this returns `EOPNOTSUPP`.
    /// * `flags` may only contain `SIGNALED`, or this returns `EINVAL`.
    /// * Syncobj allocation/handle allocation can fail with `ENOMEM`/`ENOSPC`.
    /// * The returned handle must eventually be destroyed with
    ///   `syncobj_destroy`.
    syncobj_create, SyncobjCreate, 0xBF, WR);
define_drm_ioctl!(
    /// Destroy a DRM sync object handle.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobjs, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * `handle` must be valid for this file, or this returns `EINVAL`.
    syncobj_destroy, SyncobjDestroy, 0xC0, WR);
define_drm_ioctl!(
    /// Export a syncobj handle to an fd.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobjs, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * Flags may only contain `EXPORT_SYNC_FILE` and `TIMELINE`, or this
    ///   returns `EINVAL`.
    /// * `point` must be zero unless `EXPORT_SYNC_FILE` or `TIMELINE` requires
    ///   it, or this returns `EINVAL`.
    /// * `handle` must be valid, or this returns `EINVAL`.
    /// * Exporting can fail with fd/fence errors such as `EMFILE`, `ENOENT`, or
    ///   `ENOMEM`.
    syncobj_handle_to_fd, SyncobjHandle, 0xC1, WR);
define_drm_ioctl!(
    /// Import a syncobj or sync_file fd to a handle.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobjs, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * Flags may only contain `IMPORT_SYNC_FILE` and `TIMELINE`, or this
    ///   returns `EINVAL`.
    /// * `point` must be zero unless `IMPORT_SYNC_FILE` or `TIMELINE` requires
    ///   it, or this returns `EINVAL`.
    /// * `fd` must be a compatible syncobj or sync_file fd, or this returns
    ///   `EINVAL` or fd lookup errors.
    /// * Handle allocation can fail with `ENOMEM`/`ENOSPC`.
    syncobj_fd_to_handle, SyncobjHandle, 0xC2, WR);
define_drm_ioctl!(
    /// Wait for binary sync objects.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobjs, or this returns `EOPNOTSUPP`.
    /// * Flags may only contain `WAIT_ALL`, `WAIT_FOR_SUBMIT`, and
    ///   `WAIT_DEADLINE`, or this returns `EINVAL`.
    /// * `handles` must point to `count_handles` valid handles, or this returns
    ///   `EFAULT`/`ENOENT`.
    /// * Missing unsignaled fences without `WAIT_FOR_SUBMIT` return `EINVAL`.
    /// * Timeout returns `ETIME`; interrupted waits return `ERESTARTSYS`.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_wait, SyncobjWait, 0xC3, WR);
define_drm_ioctl!(
    /// Reset binary sync objects to unsignaled.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobjs, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * `count_handles` must be non-zero, or this returns `EINVAL`.
    /// * `handles` must point to `count_handles` valid handles, or this returns
    ///   `EFAULT`/`ENOENT`.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_reset, SyncobjArray, 0xC4, WR);
define_drm_ioctl!(
    /// Signal binary sync objects.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobjs, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * `count_handles` must be non-zero, or this returns `EINVAL`.
    /// * `handles` must point to `count_handles` valid handles, or this returns
    ///   `EFAULT`/`ENOENT`.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_signal, SyncobjArray, 0xC5, WR);

define_drm_ioctl!(
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must initialize at least one CRTC, or this returns
    ///   `EOPNOTSUPP`.
    /// * Request must not set `SIGNAL`, or this returns `EINVAL`.
    /// * Request must not contain unknown type/flag bits, or this returns
    ///   `EINVAL`.
    /// * CRTC index encoded in the request must exist, or this returns
    ///   `EINVAL`.
    /// * Acquiring the vblank counter can return vblank-core errors such as
    ///   `EINVAL`.
    /// * Vblank behavior can be affected by driver CRTC/vblank setup; a driver
    ///   can affect returned values and errors.
    wait_vblank, WaitVblank, 0x3a, WR);

define_drm_ioctl!(
    /// It does validate objects
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `flags` may only contain `O_CLOEXEC` and `O_NONBLOCK`, or this returns
    ///   `EINVAL`.
    /// * A leased client cannot create a lease, or this returns `EINVAL`.
    /// * `object_ids` must point to `object_count` readable object IDs, or this
    ///   returns `EFAULT`.
    /// * Object IDs must form a valid lease set, or this returns `EINVAL`.
    /// * Allocations can fail with `ENOMEM`.
    /// * Creating the lessee fd can fail with `EMFILE` or fd allocation errors.
    mode_create_lease, CreateLease, 0xC6, WR);
define_drm_ioctl!(
    /// List lessee IDs created from this master.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * `lessees_ptr` must be null or writable for `count_lessees` lease IDs,
    ///   or this returns `EFAULT`.
    mode_list_lessees, ListLessees, 0xC7, WR);
define_drm_ioctl!(
    /// List object IDs leased to this file.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * `objects_ptr` must be null or writable for `count_objects` object IDs,
    ///   or this returns `EFAULT`.
    mode_get_lease, GetLease, 0xC8, WR);
define_drm_ioctl!(
    /// Revoke a lease created from this master.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `lessee_id` must name a current lessee of this master, or the ioctl
    ///   succeeds without revoking anything.
    mode_revoke_lease, RevokeLease, 0xC9, WR);

define_drm_ioctl!(
    /// Wait for timeline sync object points.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobj timelines, or this returns `EOPNOTSUPP`.
    /// * Flags may only contain `WAIT_ALL`, `WAIT_FOR_SUBMIT`,
    ///   `WAIT_AVAILABLE`, and `WAIT_DEADLINE`, or this returns `EINVAL`.
    /// * `handles` and `points` must point to `count_handles` entries, or this
    ///   returns `EFAULT`.
    /// * Handles must be valid, or this returns `ENOENT`.
    /// * Missing unsignaled fences without `WAIT_FOR_SUBMIT`/`WAIT_AVAILABLE`
    ///   return `EINVAL`.
    /// * Timeout returns `ETIME`; interrupted waits return `ERESTARTSYS`.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_timeline_wait, SyncobjTimelineWait, 0xCA, WR);
define_drm_ioctl!(
    /// Query timeline sync object points.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobj timelines, or this returns `EOPNOTSUPP`.
    /// * Flags may only contain `LAST_SUBMITTED`, or this returns `EINVAL`.
    /// * `count_handles` must be non-zero, or this returns `EINVAL`.
    /// * `handles` must point to `count_handles` valid handles, or this returns
    ///   `EFAULT`/`ENOENT`.
    /// * `points` must point to `count_handles` writable points, or this returns
    ///   `EFAULT`.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_query, SyncobjTimelineArray, 0xCB, WR);
define_drm_ioctl!(
    /// Transfer a fence between sync objects.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobj timelines, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * Source and destination handles must be valid, or this returns `ENOENT`.
    /// * Source fence lookup can return `EINVAL` for missing fences unless
    ///   allowed by flags.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_transfer, SyncobjTransfer, 0xCC, WR);
define_drm_ioctl!(
    /// Signal timeline sync object points.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobj timelines, or this returns `EOPNOTSUPP`.
    /// * `flags` must be zero, or this returns `EINVAL`.
    /// * `count_handles` must be non-zero, or this returns `EINVAL`.
    /// * `handles` and `points` must point to `count_handles` entries, or this
    ///   returns `EFAULT`.
    /// * Handles must be valid, or this returns `ENOENT`.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_timeline_signal, SyncobjTimelineArray, 0xCD, WR);
define_drm_ioctl!(
    /// Get multi-plane framebuffer metadata.
    ///
    /// On success, fills the framebuffer metadata. If the caller is DRM master
    /// or has CAP_SYS_ADMIN, GEM handles are fresh handles and must be closed.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * `fb_id` must name a framebuffer visible to the caller, or this returns
    ///   `ENOENT`.
    /// * `flags` must not contain unknown bits, or this returns `EINVAL`.
    /// * DRM master or CAP_SYS_ADMIN may receive fresh GEM handles which must
    ///   be closed.
    /// * Handle lookup/creation can return GEM/driver errors.
    /// * Framebuffer metadata is driver-backed; a driver can affect returned
    ///   values and errors.
    mode_getfb2, ModeFbCmd2, 0xCE, WR);
define_drm_ioctl!(
    /// Register an eventfd to be signalled by a syncobj point.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support syncobj timelines, or this returns `EOPNOTSUPP`.
    /// * Flags must be zero or `WAIT_AVAILABLE`, or this returns `EINVAL`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * `handle` must be valid, or this returns `ENOENT`.
    /// * `fd` must be a valid eventfd, or this returns eventfd lookup errors
    ///   such as `EINVAL`/`EBADF`.
    /// * Internal allocation can fail with `ENOMEM`.
    syncobj_eventfd, SyncobjEventfd, 0xCF, WR);
define_drm_ioctl!(
    /// Close a framebuffer without disabling active planes.
    ///
    /// # SAFETY
    /// * Not allowed on render nodes, or this returns `EACCES`.
    /// * Driver must support modesetting, or this returns `EOPNOTSUPP`.
    /// * `pad` must be zero, or this returns `EINVAL`.
    /// * `fb_id` must name a framebuffer visible to this file, or this returns
    ///   `ENOENT`.
    mode_closefb, ModeCloseFb, 0xD0, WR);

define_drm_ioctl!(
    /// Attach a name to a drm_file
    ///
    /// Having a name allows for easier tracking and debugging.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * `name_len` must be <= `DRM_CLIENT_NAME_MAX_LEN`, or this returns
    ///   `EINVAL`.
    /// * `name` must be readable for `name_len` bytes, or this returns
    ///   `EFAULT`.
    /// * Name bytes must contain exactly `name_len` bytes before the appended
    ///   NUL and must all be printable non-whitespace ASCII, or this returns
    ///   `EINVAL`.
    /// * Internal allocation can fail with `ENOMEM`.
    set_client_name, SetClientName, 0xD1, WR);
define_drm_ioctl!(
    /// Move a GEM object to a different handle number.
    ///
    /// On success, the old handle is closed and future operations must use
    /// `new_handle`.
    ///
    /// # SAFETY
    /// * Available to render clients.
    /// * Driver must support GEM, or this returns `EOPNOTSUPP`.
    /// * `handle` must exist for this file, or this returns `ENOENT`.
    /// * `new_handle` must be available, or this returns id allocator errors
    ///   such as `ENOSPC`.
    /// * If `handle == new_handle`, the ioctl succeeds without changing the
    ///   handle table.
    /// * PRIME bookkeeping can return additional core errors.
    gem_change_handle, GemChangeHandle, 0xD2, WR);
