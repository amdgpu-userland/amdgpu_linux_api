use std::{convert::TryFrom, os::fd::AsRawFd};

use crate::drm::{
    DrmFile,
    ioctl::{self, drm::CLIENT_NAME_MAX_LEN},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClientName<'a>(&'a [u8]);

#[derive(Debug)]
pub enum ClientNameError {
    TooLong(usize),
    InvalidCharacterAt(usize),
}

impl<'a> ClientName<'a> {
    /// Panics if input fails validation
    pub const fn new(s: &'a str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len();

        if len > CLIENT_NAME_MAX_LEN {
            panic!("ClientName error: String exceeds CLIENT_NAME_MAX_LEN (64 bytes)");
        }

        let mut idx = 0;
        while idx < len {
            if !bytes[idx].is_ascii_graphic() {
                panic!(
                    "ClientName error: String contains invalid characters (must be visible ASCII, no spaces)"
                );
            }
            idx += 1;
        }

        Self(bytes)
    }
}

impl<'a> TryFrom<&'a str> for ClientName<'a> {
    type Error = ClientNameError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        let len = bytes.len();

        if len > CLIENT_NAME_MAX_LEN {
            return Err(ClientNameError::TooLong(len));
        }

        let mut idx = 0;
        while idx < len {
            if !bytes[idx].is_ascii_graphic() {
                return Err(ClientNameError::InvalidCharacterAt(idx));
            }
            idx += 1;
        }

        Ok(Self(bytes))
    }
}

/// Attach a name to a drm_file
///
/// Accepts empty string
pub fn set_client_name(drm_client: &impl DrmFile, name: ClientName<'_>) {
    let fd = drm_client.as_fd().as_raw_fd();
    let name = name.0;
    let mut args = ioctl::drm::SetClientName {
        name_len: name.len(),
        name: name.as_ptr(),
    };
    if let Err(e) = unsafe { ioctl::drm::set_client_name(fd, &mut args) } {
        // Probably ENOMEM
        panic!("set client name: {e}");
    }
}
