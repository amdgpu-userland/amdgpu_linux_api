#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_result_trait_fn)]
#![feature(const_index)]
// This is too broken to be used yet
//#![feature(guard_patterns)]
// // For expanded macro debugging
// #![feature(panic_internals)]
// #![feature(fmt_helpers_for_derive)]
// #![feature(derive_clone_copy)]
// #![feature(trivial_clone)]

#[macro_use]
mod ioctl_helpers;

#[macro_export]
macro_rules! GPU_PAGE_SIZE {
    () => {
        4096
    };
}

/// Think OpenGL and Vulkan
pub mod drm;

/// Bindings for Kernel Fusion Driver
/// the thing Radeon Open CoMpute (ROCM) is built on
///
/// The entry point are versioned Kfd stucts.
///
/// Pick one to use as your application minimal version requirement.
/// The assumption is Linux never breaks userspace
/// so future versions should be backwards compatible.
///
/// Keep in mind old Kfd versioned structs use
/// implementation based on the newest kernel kfd code.
/// Which shouldn't matter if amdgpu code didn't
/// break userspace between versions.
pub mod kfd;

/// Defines packets for different SDMA versions
///
/// These packets are then to be writen to a ring buffer
/// and read by the gpu
///
/// Hopefully a packet has a fixed size
/// It doesn't
///
/// There is a scheme emerging
/// - creating packets
/// - writing packets to IB
/// - writing packets to ring_buffer
/// - reading packets from IB (decode)
/// - reading packets from ring_buffer (wrapping_decode)
///
/// Most packets have fixed size but some have variable length for example NOP packet
///
/// We do know this length during compile time only on the encode path.
///
/// We could model encode like this
/// ```rust
/// Interface for the programmer to initialize and interruct with packet info
/// pub struct Packet<T> {
///     pub small_field: bool,
///     pub data: T,
/// }
///
/// pub type PacketConst<const N: usize> = Packet<[u64; N]>;
/// pub type PacketNotConst<'data> = Packet<&'data [u64]>;
/// pub type PacketVec = Packet<Vec<u64>>;
///
/// impl<const N: usize> PacketConst<N> {
///     pub const fn encode(&self) -> [u64; N + 1] {
///         let mut res = [0u64; N + 1];
///         res[0] = self.small_field as u64;
///         res[1..].copy_from_slice(&self.data);
///         res
///     }
/// }
/// ```
/// But it is pretty annoying to use such an array and as you can see this implementation allocates
/// more memory on the stack only to copy it elsewere
///
/// One benefit is that such a definition doesn't worry if the place the memory will be copied to
/// has enough space and it is clear how much memory will be written
///
/// Another benefit is it doesn't worry about wrapping like with ring buffer
///
/// On decode path the size cannot be generalized over, because decode uses a common
/// enum to distinguish which packet type it is. And enums don't like generic variants with
/// unpredictable types (lengths).
pub mod sdma;

pub mod sdma_new;
