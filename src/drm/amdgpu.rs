#[cfg(feature = "async")]
use std::{
    fs::OpenOptions,
    io::{Error, ErrorKind},
    os::fd::OwnedFd,
};
use std::{
    marker::PhantomData,
    os::{fd::FromRawFd, unix::fs::OpenOptionsExt},
};

#[cfg(feature = "async")]
use tokio::io::{Interest, unix::AsyncFd};

use crate::{
    capabilities::{ActiveCaps, CAP_SYS_ADMIN},
    drm::{
        PrimaryClient,
        auth::{Authenticated, Exclusive, Local, Unknown},
    },
};

/// AMD's linux driver drm specific part's abstraction
/// It implements the following for kms_driver
/// DRIVER_ATOMIC |
/// DRIVER_GEM |
/// DRIVER_RENDER | DRIVER_MODESET | DRIVER_SYNCOBJ |
/// DRIVER_SYNCOBJ_TIMELINE,
///
/// It has FOP_UNSIGNED_OFFSET
///
/// An drm's VM has 2 schedulers - immediate and delayed
///
/// Perhaps I should track whether the VM has been marked for compute by kfd::acquire_vm
///
/// There is a big difference when system has resizable BAR - large bar.
///
/// CSA - device wide, staticly assigned buffer if device's HWIP has GMC type and MCBP is on (module parameter mcbp == 1)
/// of size 1 << 21 (2 MiB) in VRAM or GTT, at the very top of address space,
/// mapped READABLE | WRITABLE | EXECUTABLE
/// I believe it is designed to be left to the firmware to manage and should only be read from by
/// the cpu
/// At + 0x0 is GFX_META_DATA for example v10_gfx_meta_data
/// For gfx8 at +0x1000 is gds_backup
/// At + 0x2000 is CSA_SDMA which is block of 64 bytes indexed by ring num (up to 31 after which
/// it's clamped to 0)
/// At + 0x3000 is CSA_VPE which is blocks of 64 bytes indexed by ring num
/// since gfx9 At the end gds_backup is located
///
/// SEQ64 - device wide, statically assigned buffer of u64 values if device's HWIP has GMC type of size 1 << 21
/// (2 MiB) in GTT, at the top right after CSA, mapped READABLE | MTYPE_UC
/// these are semaphores for userq
/// these semaphore allocation is managed via a giant bitmap with 32768 available slots
/// we can get the index via amdgpu_userq_wait_ioctl - first call it with num_fences = 0, to get
/// how many there are, and then again with enough memory to get the VA and seqno back
pub struct Amdgpu<Bar, Mcbp> {
    _bar: PhantomData<Bar>,
    mcbp: Mcbp,
}

/// An actual device handled by the amdgpu driver, from the driver's point of view
pub struct Gpu<ReBAR, Apu, Mcbp, Tmz, CoordTruncMode, Virtualization, GangSubmission> {
    _has_large_bar: PhantomData<ReBAR>,
    _is_apu: PhantomData<Apu>,
    _mcbp_on: PhantomData<Mcbp>,
    _tmz_on: PhantomData<Tmz>,
    _coord_trunc_mode: PhantomData<CoordTruncMode>,
    _virt_mode: PhantomData<Virtualization>,
    _allow_gang_submission: PhantomData<GangSubmission>,
}

// Drm doesn't have a particular version as it's more like scaffolding for other drivers, that may
// chose to change things up

pub fn try_open_primary_with_cap_sys_admin(
    num: i32,
    token: &ActiveCaps<CAP_SYS_ADMIN>,
) -> PrimaryClient<Authenticated, Local, Exclusive, Amdgpu<Unknown, Unknown>> {
    // O_EXCL not allowed -> EBUSY
    // if device is not (turned_on or dynamically_turned_off) EINVAL
    match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(0)
        .open(format! {"/dev/dri/card{num}"})
    {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }
    let _ = token;
}

pub unsafe fn try_open_primary_with_cap_sys_admin_unchecked(
    num: i32,
) -> PrimaryClient<Authenticated, Local, Exclusive, Amdgpu<Unknown, Unknown>> {
    todo!()
}

pub fn try_open_primary(
    num: i32,
) -> PrimaryClient<Unknown, Local, Exclusive, Amdgpu<Unknown, Unknown>> {
    todo!()
}

#[cfg(all(target_arch = "sparc", not(target_feature = "v9")))]
compile_error!("Drm doesn't accept sparc before v9");

#[cfg(feature = "async")]
pub fn try_open_nonblocking(num: i32) -> tokio::io::unix::AsyncFd<OwnedFd> {
    match OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(format! {"/dev/dri/card{num}"})
    {
        Err(e) => match e.raw_os_error().unwrap_or_default() {
            // Unlucky, the device is being removed
            libc::ENODEV => todo!(),
            // Probably the device is powered off
            //
            // Cpu is valid (not sparc before v9)
            //
            // Amdgpu driver defines FOP_UNSIGNED_OFFSET
            libc::EINVAL => todo!(),
            // Unlucky to run out of memory
            //
            // Can happen in drm and amdgpu specific part
            libc::ENOMEM => todo!(),
            // Amdgpu specific, ras interrupt triggered and device disabled
            libc::EHWPOISON => todo!(),
            // Amdgpu specific, if gpu is partitioned and a partition is not valid, but I don't
            // know why it could be invalid and the file to still be there
            libc::ENOENT => todo!(),
            // either not expected or non drm / driver specific so fall back to regular file errors
            _ => todo!(),
        },
        Ok(fd) => AsyncFd::with_interest(OwnedFd::from(fd), Interest::READABLE)
            .expect("Should be readable in drm subsystem"),
    }
}

// unused:
// O_APPEND
// O_ASYNC
// O_CREAT
// O_DSYNC
// O_LARGEFILE
// O_NOATIME
// O_NOCTTY
// O_NOFOLLOW
// O_SYNC
// O_TRUNK
//
// better_not:
// O_DIRECT
// O_DIRECTORY
// O_NOATIME
// O_PATH
// O_TMPFILE
//
// specifically_prohibited:
// O_EXCL
//
// used:
// O_CLOEXEC
// O_NONBLOCK
