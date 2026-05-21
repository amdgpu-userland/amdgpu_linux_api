use super::DRM_COMMAND_BASE;
use super::DRM_IOCTL_BASE;

mod structs;
pub use structs::*;

pub type CtxId = u32;
pub type BoListHandle = u32;
pub type CsHandle = u64;
pub type FenceHandle = u64;
pub type SyncObjHandle = u32;
pub type SyncobjSeqNo = u64;

/// Index for instance of HW IP
/// You should most likely use 0
pub type IpInstance = u32;

/// Index for ring of an instance of HW IP
pub type IpRing = u32;

/// Fence / handle for the submission
/// Fence number is incremented on each submission
/// and it can repeat after a long while
pub type CsFence = u64;

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
    /// You need to check the gem's properties later.
    ///
    /// For example it can move the allocation to GTT if there is not enough VRAM free.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * Allocation domains and flags must be supported by the device and
    ///   valid for the requested size/alignment.
    /// * The returned GEM handle must be closed with the generic DRM
    ///   `gem_close` ioctl when no longer needed.
    gem_create, GemCreate, 0x00, WR);
define_amddrm_ioctl!(
    /// Create an mmap offset for a GEM object.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `handle` must name a valid GEM object for this DRM file.
    /// * The returned offset is only an mmap offset; callers must still use
    ///   correct mmap length/protection according to the BO.
    gem_mmap, GemMmap, 0x01, WR);
define_amddrm_ioctl!(
    /// Allocate, free, query, or configure an amdgpu command submission context.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `op` must be a valid `CtxOp`, and fields not used by that operation
    ///   should be zeroed.
    /// * `ctx_id` must name a context owned by this DRM file for operations
    ///   other than allocation.
    /// * Priorities above normal require DRM master or `CAP_SYS_NICE`.
    ctx, Ctx, 0x02, WR);
define_amddrm_ioctl!(
    /// Create, update, or destroy an amdgpu buffer-object list.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `operation` must be valid for the supplied fields.
    /// * `list_handle` must name a BO list owned by this DRM file for update
    ///   or destroy operations.
    /// * `bo_info_ptr` must be readable for `bo_number * bo_info_size` bytes
    ///   when entries are provided.
    /// * Every referenced BO handle must be valid for this DRM file.
    bo_list, BoList, 0x03, WR);
define_amddrm_ioctl!(
    /// Submit command chunks to a context.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `ctx_id` and `bo_list_handle` must be valid for this DRM file when
    ///   used.
    /// * `chunks` must be readable for `num_chunks` pointers, and every chunk
    ///   pointer must be readable for the chunk header and its `length_dw`.
    /// * Chunk payload pointers and GPU virtual addresses must remain valid
    ///   for the duration required by the kernel submission path.
    /// * Referenced BO, syncobj, fence, engine, instance, and ring values must
    ///   be valid and compatible with the target device.
    cs, Cs, 0x04, WR);
define_amddrm_ioctl!(
    /// Query amdgpu device, memory, firmware, sensor, or engine information.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `query` must be a supported `InfoQuery`.
    /// * `return_pointer` must be writable for `return_size` bytes when the
    ///   query writes output through it, or this returns `EFAULT`.
    /// * `quick_info` must contain the query-specific input fields required by
    ///   the selected query.
    info, Info, 0x05, W);
define_amddrm_ioctl!(
    /// Get or set GEM metadata such as tiling information.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `handle` must name a valid GEM object for this DRM file.
    /// * `op` must be a valid metadata operation.
    /// * For set operations, `data_size_bytes` must fit in the fixed metadata
    ///   data array.
    gem_metadata, GemMetadata, 0x06, WR);
define_amddrm_ioctl!(
    /// Wait for a GEM object to become idle.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `handle` must name a valid GEM object for this DRM file.
    /// * `timeout` is interpreted by the kernel as an absolute wait deadline.
    gem_wait_idle, GemWaitIdle, 0x07, WR);
define_amddrm_ioctl!(
    /// Map, unmap, clear, or replace a GEM object's GPU virtual-address range.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `handle` must name a valid GEM object for this DRM file when the
    ///   selected operation uses one.
    /// * `va_address`, `offset_in_bo`, and `map_size` must satisfy the device
    ///   GPUVM alignment and range requirements.
    /// * `flags` and `operation` must be valid and supported by the device.
    /// * `input_fence_syncobj_handles` must be readable for
    ///   `num_syncobj_handles` entries when non-null.
    gem_va, GemVa, 0x08, W);
define_amddrm_ioctl!(
    /// Wait for a command submission fence.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `ctx_id`, `ip_type`, `ip_instance`, and `ring` must identify a valid
    ///   submission context/engine for this DRM file.
    /// * `handle` must be a valid fence sequence number, `0`, or `!0u64`.
    /// * `timeout` is interpreted by the kernel as an absolute wait deadline.
    cs_wait, CsWait, 0x09, WR);
define_amddrm_ioctl!(
    /// Get or set a value associated with a GEM buffer.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `handle` must name a valid GEM object for this DRM file.
    /// * `op` must be a valid `GemOpOp`.
    /// * `value` is operation-dependent: it must be writable `GemCreateIn`
    ///   storage for create-info queries, an encoded domain bitmask for
    ///   placement changes, or writable `GemVmEntry` storage for mapping-info
    ///   queries.
    gem_op, GemOp, 0x10, WR);
define_amddrm_ioctl!(
    /// Create a GEM object backed by user memory.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `addr` and `size` must describe a valid user address range with the
    ///   alignment and lifetime required by amdgpu userptr.
    /// * `flags` must contain only supported userptr flags.
    /// * Userptr is not a reliable API; callers need a non-userptr fallback.
    /// * The returned GEM handle must be closed with the generic DRM
    ///   `gem_close` ioctl when no longer needed.
    gem_userptr, GemUserptr, 0x11, WR);
define_amddrm_ioctl!(
    /// Wait for one or more amdgpu fences.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `fences` must be readable for `fence_count` `Fence` entries.
    /// * Each fence must identify a valid context, engine, instance, ring, and
    ///   sequence number for this DRM file.
    /// * `wait_all` must be a valid boolean value expected by the kernel.
    wait_fences, WaitFences, 0x12, WR);
define_amddrm_ioctl!(
    /// Reserve or unreserve a VMID for this amdgpu VM.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `op` must be a valid `VmOp`.
    /// * VMID reservation is per-file VM state; callers must balance reserve
    ///   and unreserve operations according to the kernel contract.
    vm, Vm, 0x13, WR);
define_amddrm_ioctl!(
    /// Convert an amdgpu fence into a sync object, sync object fd, or sync-file fd.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `fence` must identify a valid context, engine, instance, ring, and
    ///   sequence number for this DRM file.
    /// * `what` must be a valid `FenceToHandleWhat`.
    /// * Returned syncobj handles or file descriptors must be destroyed or
    ///   closed according to their type.
    fence_to_handle, FenceToHandle, 0x14, WR);
define_amddrm_ioctl!(
    /// Override amdgpu process or context scheduling priority.
    ///
    /// # SAFETY
    /// * Requires DRM master status, or this returns `EACCES`.
    /// * `op` must be a valid `SchedOp`.
    /// * `fd`, `ctx_id`, and `priority` must be valid for the selected
    ///   scheduler operation.
    sched, Sched, 0x15, W);
define_amddrm_ioctl!(
    /// Create or free an amdgpu user queue.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `op` must be a valid `UserqOp`.
    /// * For create operations, queue, pointer, doorbell, and MQD fields must
    ///   satisfy the engine-specific size, alignment, and mapping requirements.
    /// * For free operations, `queue_id` must name a queue owned by this DRM
    ///   file.
    /// * High userq priority and secure queues can require elevated privilege
    ///   or device support.
    userq, Userq, 0x16, WR);
define_amddrm_ioctl!(
    /// Attach sync object and BO fences to a submitted user queue job.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `queue_id` must name a user queue owned by this DRM file.
    /// * `syncobj_handles`, `bo_read_handles`, and `bo_write_handles` must be
    ///   readable for their corresponding count fields when non-null.
    /// * Every referenced syncobj and GEM handle must be valid for this DRM
    ///   file.
    userq_signal, UserqSignal, 0x17, WR);
define_amddrm_ioctl!(
    /// Resolve user queue wait dependencies into GPU address/value fences.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `waitq_id` must name a user queue wait queue owned by this DRM file.
    /// * Input handle and timeline arrays must be readable for their
    ///   corresponding count fields when non-null.
    /// * `out_fences` must be writable for the input `num_fences`
    ///   `UserqFenceInfo` entries when non-null.
    /// * Every referenced syncobj and GEM handle must be valid for this DRM
    ///   file.
    userq_wait, UserqWait, 0x18, WR);
define_amddrm_ioctl!(
    /// List GEM handles visible to this DRM file.
    ///
    /// # SAFETY
    /// * Client must be authenticated or a render client, or this returns
    ///   `EACCES`.
    /// * `entries` must be writable for `num_entries` `GemListHandlesEntry`
    ///   elements when non-null.
    /// * The kernel may update `num_entries`; callers must retry with a larger
    ///   buffer if the returned count exceeds the provided capacity.
    gem_list_handles, GemListHandles, 0x19, WR);
