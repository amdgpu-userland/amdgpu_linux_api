const DRM_IOCTL_BASE: u32 = 'd' as u32;
const DRM_COMMAND_BASE: u32 = 0x40;

/// Amdgpu specific
#[macro_use]
pub mod amd;

/// Common for all vendors
///
/// This module exposes thin wrappers over the generic DRM ioctls defined by
/// `include/uapi/drm/drm.h` and dispatched by `drivers/gpu/drm/drm_ioctl.c`.
/// The functions are intentionally direct ioctl bindings: they do not validate
/// arguments, manage object lifetimes, retry ioctls, convert errno values, or
/// provide higher-level Rust abstractions.
///
/// Most functions return `Ok(())` when the kernel ioctl returns 0, or `Err(errno)`
/// with the current libc errno when the ioctl fails. Output values are written
/// into the argument structs in-place.
///
/// # Groups
///
/// ## Driver and file metadata
///
/// - [`version`](self::drm::version()) queries driver name, description, and driver version.
/// - [`get_unique`](self::drm::get_unique()) queries the legacy bus/device unique string.
/// - [`get_client`](self::drm::get_client()) exposes legacy client information.
/// - [`set_client_name`](self::drm::set_client_name()) attaches a debug name to the DRM file.
///
/// ## Authentication and master state
///
/// - [`get_magic`](self::drm::get_magic()) creates or returns this file's authentication magic.
/// - [`auth_magic`](self::drm::auth_magic()) authenticates another DRM file by magic.
/// - [`set_master`](self::drm::set_master()) and [`drop_master`](self::drm::drop_master()) change DRM master ownership.
/// - [`set_version`](self::drm::set_version()) sets legacy DRM interface version state.
///
/// ## Capabilities
///
/// - [`get_cap`](self::drm::get_cap()) queries device/driver capabilities.
/// - [`set_client_cap`](self::drm::set_client_cap()) enables per-file client capabilities such as universal
///   planes or atomic modesetting.
///
/// ## GEM and PRIME sharing
///
/// - [`gem_close`](self::drm::gem_close()) closes a GEM handle.
/// - [`gem_flink`](self::drm::gem_flink()) and [`gem_open`](self::drm::gem_open()) use legacy global GEM names.
/// - [`gem_change_handle`](self::drm::gem_change_handle()) moves a GEM object to a requested handle number.
/// - [`prime_handle_to_fd`](self::drm::prime_handle_to_fd()) exports a GEM handle as a DMA-BUF fd.
/// - [`prime_fd_to_handle`](self::drm::prime_fd_to_handle()) imports a DMA-BUF fd as a GEM handle.
///
/// ## Vblank and CRTC sequences
///
/// - [`wait_vblank`](self::drm::wait_vblank()) waits for or queues legacy vblank events.
/// - [`crtc_get_sequence`](self::drm::crtc_get_sequence()) queries a CRTC vblank sequence.
/// - [`crtc_queue_sequence`](self::drm::crtc_queue_sequence()) queues a CRTC sequence event.
///
/// ## KMS resources and legacy modesetting
///
/// - [`mode_getresources`](self::drm::mode_getresources()) lists framebuffers, CRTCs, connectors, and encoders.
/// - [`mode_getcrtc`](self::drm::mode_getcrtc()) and [`mode_setcrtc`](self::drm::mode_setcrtc()) query or set CRTC state.
/// - [`mode_getencoder`](self::drm::mode_getencoder()) and [`mode_getconnector`](self::drm::mode_getconnector()) query display topology.
/// - [`mode_getgamma`](self::drm::mode_getgamma()) and [`mode_setgamma`](self::drm::mode_setgamma()) access legacy CRTC gamma tables.
/// - [`mode_cursor`](self::drm::mode_cursor()) and [`mode_cursor2`](self::drm::mode_cursor2()) update legacy cursor state.
///
/// ## KMS planes, framebuffers, and dumb buffers
///
/// - [`mode_getplaneresources`](self::drm::mode_getplaneresources()), [`mode_getplane`](self::drm::mode_getplane()), and [`mode_setplane`](self::drm::mode_setplane()) operate
///   on KMS planes.
/// - [`mode_getfb`](self::drm::mode_getfb()), [`mode_getfb2`](self::drm::mode_getfb2()), [`mode_addfb`](self::drm::mode_addfb()), [`mode_addfb2`](self::drm::mode_addfb2()),
///   [`mode_rmfb`](self::drm::mode_rmfb()), and [`mode_closefb`](self::drm::mode_closefb()) operate on KMS framebuffers.
/// - [`mode_page_flip`](self::drm::mode_page_flip()) schedules a legacy page flip.
/// - [`mode_dirtyfb`](self::drm::mode_dirtyfb()) marks framebuffer regions dirty.
/// - [`mode_create_dumb`](self::drm::mode_create_dumb()), [`mode_map_dumb`](self::drm::mode_map_dumb()), and [`mode_destroy_dumb`](self::drm::mode_destroy_dumb()) operate
///   on KMS dumb buffers.
///
/// ## KMS properties and atomic commits
///
/// - [`mode_getproperty`](self::drm::mode_getproperty()) and [`mode_getpropblob`](self::drm::mode_getpropblob()) query property metadata.
/// - [`mode_setproperty`](self::drm::mode_setproperty()) sets legacy connector properties.
/// - [`mode_obj_getproperties`](self::drm::mode_obj_getproperties()) and [`mode_obj_setproperty`](self::drm::mode_obj_setproperty()) query or set
///   properties on any KMS object.
/// - [`mode_createpropblob`](self::drm::mode_createpropblob()) and [`mode_destroypropblob`](self::drm::mode_destroypropblob()) manage user property
///   blobs.
/// - [`mode_atomic`](self::drm::mode_atomic()) performs or tests an atomic KMS commit.
///
/// ## Sync objects
///
/// - [`syncobj_create`](self::drm::syncobj_create()) and [`syncobj_destroy`](self::drm::syncobj_destroy()) manage sync object handles.
/// - [`syncobj_handle_to_fd`](self::drm::syncobj_handle_to_fd()) and [`syncobj_fd_to_handle`](self::drm::syncobj_fd_to_handle()) export/import sync
///   objects or sync files.
/// - [`syncobj_wait`](self::drm::syncobj_wait()), [`syncobj_reset`](self::drm::syncobj_reset()), and [`syncobj_signal`](self::drm::syncobj_signal()) operate on
///   binary sync objects.
/// - [`syncobj_timeline_wait`](self::drm::syncobj_timeline_wait()), [`syncobj_query`](self::drm::syncobj_query()), [`syncobj_transfer`](self::drm::syncobj_transfer()),
///   [`syncobj_timeline_signal`](self::drm::syncobj_timeline_signal()), and [`syncobj_eventfd`](self::drm::syncobj_eventfd()) operate on timeline
///   sync objects.
///
/// ## DRM leasing
///
/// - [`mode_create_lease`](self::drm::mode_create_lease()) creates a lessee DRM file.
/// - [`mode_list_lessees`](self::drm::mode_list_lessees()) lists lessees.
/// - [`mode_get_lease`](self::drm::mode_get_lease()) lists objects leased to the current file.
/// - [`mode_revoke_lease`](self::drm::mode_revoke_lease()) revokes a lease.
///
/// # Safety
///
/// All functions in this module are unsafe because they pass caller-provided
/// pointers and raw file descriptors to the kernel. Callers must ensure:
///
/// - The file descriptor refers to an appropriate DRM node.
/// - Input pointers reference readable memory for the requested sizes.
/// - Output pointers reference writable memory for the requested sizes.
/// - Object IDs and handles belong to the DRM file and remain valid for the
///   duration of the ioctl.
/// - Returned file descriptors and GEM/syncobj handles are closed or destroyed
///   according to the corresponding ioctl contract.
///
/// Individual function docs list the main kernel requirements and common errno
/// values from the generic DRM implementation. Drivers may still impose
/// additional validation or return driver-specific errors where noted.
#[macro_use]
pub mod drm;
