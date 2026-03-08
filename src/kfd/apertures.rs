use std::{mem::MaybeUninit, os::fd::AsRawFd};

use super::ioctl;
use crate::kfd::KfdFile;

#[derive(Debug)]
pub enum AperturesError {
    /// Internal kcalloc() failed
    NoMem,
    CopyingBackToUser,
    Unexpected(ioctl::Errno),
}

#[derive(Debug)]
pub enum AperturesErrorLimited {
    BufferTooLarge,
    /// Internal kcalloc() failed
    NoMem,
    CopyingBackToUser,
    Unexpected(ioctl::Errno),
}

pub trait Apertures: KfdFile {
    /// Remember that devices can be removed after this call, so
    /// return values might not be valid after some time
    fn apertures<'buf>(
        &self,
        buffer: &'buf mut [ioctl::ProcessDeviceApertures; 7],
    ) -> &'buf mut [ioctl::ProcessDeviceApertures] {
        let fd = self.as_fd().as_raw_fd();
        let mut args = ioctl::GetProcessAperturesArgs::default();

        #[expect(deprecated)]
        if let Err(e) = unsafe { ioctl::get_process_apertures(fd, &mut args) } {
            match e {
                _ => panic!("unexpected get_process_apertures: {e}"),
            }
        }

        *buffer = args.process_apertures;
        let len: usize = args.num_of_nodes as usize;

        &mut buffer[0..len]
    }
}

pub trait AperturesNew: KfdFile {
    /// Remember that devices can be removed after this call, so
    /// return values might not be valid after some time
    fn all_apertures(&self) -> Result<Vec<ioctl::ProcessDeviceApertures>, AperturesError> {
        let fd = self.as_fd().as_raw_fd();
        let mut args = ioctl::GetProcessAperturesNewArgs {
            num_of_nodes: 0,
            ..Default::default()
        };
        // Gets num_of_nodes
        let res = unsafe { ioctl::get_process_apertures_new(fd, &mut args) };
        debug_assert!(
            res.is_ok(),
            "When num_of_nodes = 0, it shouldn't be able to throw"
        );

        let mut vec: Vec<MaybeUninit<ioctl::ProcessDeviceApertures>> =
            Vec::with_capacity(args.num_of_nodes as usize);
        unsafe { vec.set_len(args.num_of_nodes as usize) };

        args.kfd_process_device_apertures_ptr =
            vec.as_mut_ptr() as *mut ioctl::ProcessDeviceApertures;
        if let Err(e) = unsafe { ioctl::get_process_apertures_new(fd, &mut args) } {
            let er = match e {
                libc::ENOMEM => AperturesError::NoMem,
                libc::EFAULT => AperturesError::CopyingBackToUser,
                _ => AperturesError::Unexpected(e),
            };
            return Err(er);
        }

        // SAFETY: the ioctl has initialized all elements
        Ok(unsafe {
            std::mem::transmute::<
                Vec<MaybeUninit<ioctl::ProcessDeviceApertures>>,
                Vec<ioctl::ProcessDeviceApertures>,
            >(vec)
        })
    }

    /// Please call with relatively small array.
    /// There should be at least 1 gpu (len = 1)
    /// Old kfd limit was 7
    ///
    /// Remember that devices can be removed after this call, so
    /// return values might not be valid after some time
    fn apertures_limited<'buf>(
        &self,
        buffer: &'buf mut [ioctl::ProcessDeviceApertures],
    ) -> Result<&'buf mut [ioctl::ProcessDeviceApertures], AperturesErrorLimited> {
        let fd = self.as_fd().as_raw_fd();
        let mut args = ioctl::GetProcessAperturesNewArgs {
            num_of_nodes: u32::try_from(buffer.len())
                .map_err(|_| AperturesErrorLimited::BufferTooLarge)?,
            kfd_process_device_apertures_ptr: buffer.as_mut_ptr(),
            _pad: 0,
        };

        if let Err(e) = unsafe { ioctl::get_process_apertures_new(fd, &mut args) } {
            let er = match e {
                libc::ENOMEM => AperturesErrorLimited::NoMem,
                libc::EFAULT => AperturesErrorLimited::CopyingBackToUser,
                _ => AperturesErrorLimited::Unexpected(e),
            };
            return Err(er);
        }
        let len = args.num_of_nodes as usize;

        Ok(&mut buffer[..len])
    }
}
