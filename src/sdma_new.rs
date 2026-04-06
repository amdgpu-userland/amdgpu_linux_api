pub const HEADER_MASK: u32 = 0xffff;

const trait FieldDecode: Sized {
    fn decode(val: u32) -> Option<Self>;
}

macro_rules! impl_decode {
    ($($simple_type:ty),*) => {
        $(impl const FieldDecode for $simple_type {
            fn decode(val: u32) -> Option<Self> {
                Self::try_from(val).ok()
            }
        })*
    };
}

impl_decode!(u8, u16, u32, i8, i16, i32);
impl const FieldDecode for bool {
    fn decode(val: u32) -> Option<Self> {
        Some(val != 0)
    }
}

macro_rules! field_enum {
    (
        $(#[$attr:meta])*
        $name:ident : $mask:literal {
            $(
                $(#[$vattr:meta])*
                $variant:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$attr])*
        #[repr(u32)]
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Default)]
        pub enum $name {
            $(
                $(#[$vattr])*
                $variant = $value,
            )*
        }

        const _: () = {
            $(
                assert!(
                    $value <= $mask,
                    concat!(
                        stringify!($name), "::", stringify!($variant),
                        " = ", stringify!($value),
                        " exceeds mask ", stringify!($mask),
                    )
                );
            )*
        };

        impl const super::FieldDecode for $name {
            fn decode(val: u32) -> Option<Self> {
                match val {
                    $($value => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

/// Define a pub struct for an sdma packet
///
/// Packets are encoded / decoded as dwords.
///
/// Some fields of a packet don't fit a dword - addresses for example.
/// You can use `join` section to declare these.
///
/// Packets can have variable length but only on one field
/// use dyn section to mark which masked dword the length is at
/// and what is the first dword this dynamic part starts at.
///
/// It must be the last dword so we don't overwrite other fields!
///
/// ## Examples
/// ### Fixed length packet
/// ```ignore
/// packet!(
/// CopyStruct {
///     @bits
///     dw[0] = {
///         /// Perhaps direction of operation
///         & 0x1 << 31 = detile: bool;
///     }
///     dw[5] = {
///         & 0x7ff << 0  = stride: u16;
///         // In umr these two are switched
///         & 0x3  << 16  = struct_sw: u8;
///         & 0x1  << 22  = struct_ha: bool;
///         & 0x3  << 24  = linear_sw: u8;
///         & 0x1  << 30  = linear_ha: bool;
///     }
///     @full
///     dw[3] = start_index: u32;
///     dw[4] = count: u32;
///     @join
///     dw[1], dw[2] = sb_addr: u64;
///     dw[6], dw[7] = linear_addr: u64;
/// }
/// );
/// ```
///
/// ### Variable length packet
/// ```ignore
/// packet!(
/// Nop<'pkt> {
///     @dyn
///     dw[1..] = data: &'pkt [u32],
///     dw[0] & 0x3fff << 16 = len
/// });
/// ```
macro_rules! packet {
    (@max_dw $cur:expr) => { $cur };
    (@max_dw $cur:expr, $next:literal $($rest:tt)*) => {
        packet!(@max_dw {
            let c = $cur;
            let n = $next;
            let gt = (n > c) as usize;
            c * (1usize - gt) + n * gt
        } $($rest)*)
    };
    (
        $(#[$attr:meta])*
        $variant:ident $(<$vlife:lifetime>)?
        {
            $(@bits
                $(
                    dw[$dw:literal] = {
                        $(
                            $(#[$fattr:meta])*
                            & $mask:literal << $shift:literal = $field:ident : $ftype:ty ;
                        )+
                    }
                )+
            )?
            $(@full
                $(
                    $(#[$f_fattr:meta])*
                    dw[$fdw:literal] = $f_field:ident: $f_ftype:ty ;
                )+
            )?
            $(@join
                $(
                    $(#[$c_fattr:meta])*
                    dw[$lo_dw:literal], dw[$hi_dw:literal] = $c_field:ident: $c_ftype:ty ;
                )+
            )?
            $(@dyn
                $(#[$dyn_fattr:meta])*
                dw[$dyn_dw:literal ..] = $dyn_field:ident : & $dyn_life: lifetime [u32],
                dw[$dyn_len:literal] $(& $dyn_mask:literal << $dyn_shift:literal)? = len
            )?
        }
) => {
        $(#[$attr])*
        pub struct $variant $(<$vlife>)? {
        $($($(
            $(#[$fattr])*
            pub $field: $ftype,
        )*)*)?
        $($(
            $(#[$f_fattr])*
            pub $f_field: $f_ftype,
        )*)?
        $($(
            $(#[$c_fattr])*
            pub $c_field: $c_ftype,
        )*)?
        $(
            $(#[$dyn_fattr])*
            pub $dyn_field: & $dyn_life [u32]
        )?
        }


        impl $(<$vlife>)? $variant $(<$vlife>)? {
            pub const STATIC_DWORDS: usize = 1 + packet!(@max_dw 0usize $($(, $dw)*)? $($(, $fdw)*)?
        $($(, $lo_dw, $hi_dw)*)? $(, $dyn_len)?);

            pub const fn partial_encode_linear(&self, buff: &mut [u32]) -> usize {
                let actual_dwords: usize = Self::STATIC_DWORDS $(+ self.$dyn_field.len())?;
                if buff.len() < actual_dwords { panic!("Buffer too small to copy data to") }

                $($(

                $(
                if self.$field as u32 > $mask {
                    panic!(concat!("encoding ", stringify!($variant), " field ", stringify!($field), " doesn't fit mask ", stringify!($mask)));
                }
                )*

                let mut mask: u32 = 0;
                $(mask |= $mask << $shift;)*

                let mut value: u32 = 0;

                $(value |= (self.$field as u32) << $shift;)*

                buff[$dw] &= !mask;
                buff[$dw] |= value;
                )*)?

                $($(buff[$fdw] = self.$f_field as u32;)*)?

                $($(
                buff[$lo_dw] = self.$c_field as u32;
                buff[$hi_dw] = (self.$c_field as u64 >> 32) as u32;
                )*)?

                $(
                const _: () = {assert!($dyn_dw == $variant::STATIC_DWORDS, "Dynamic part must start at the end");};

                let extra_length = self.$dyn_field.len();
                let len_u32 = match u32::try_from(extra_length) {
                    Ok(x) => x,
                    Err(_) => panic!("Too many extra dwords, u32 limit")
                };

                $(
                if len_u32 > $dyn_mask {
                    panic!(concat!("Too many extra dwords, max: ",stringify!($len_mask)))
                }

                let mask: u32 = $dyn_mask << $dyn_shift;
                buff[$dyn_len] &= !mask;
                )?

                buff[$dyn_len] |= len_u32 $(<< $dyn_shift)?;
                buff[$dyn_dw..($dyn_dw + extra_length)].copy_from_slice(self.$dyn_field);
                )?
                actual_dwords
            }
    pub const fn partial_decode_linear(buff: & $($vlife)? [u32]) -> (usize, $variant $(<$vlife>)?) {
        let actual_size = $variant::STATIC_DWORDS $(+ {
            if buff.len() < ($dyn_len + 1) {
                panic!(concat!("Not enough dwords for ", stringify!($variant)));
            }
            let len = buff[$dyn_len];
            $(let len = (len >> $dyn_shift) & $dyn_mask;)?
            len as usize
        })?;

        if buff.len() < actual_size {
            panic!(concat!("Not enough dwords for ", stringify!($variant)));
        }

        $($(
        let dw = buff[$dw];
        $(let Some($field) = <$ftype as super::FieldDecode>::decode((dw >> $shift) & $mask) else {panic!("Field")};)*
        )*)?

        $($(
        let Some($f_field) = <$f_ftype as super::FieldDecode>::decode(buff[$fdw]) else {panic!("Field")};
        )*)?

        $($(
        let $c_field = <$c_ftype>::from((u64::from(buff[$hi_dw]) << 32) | u64::from(buff[$lo_dw]));
        )*)?

        $(
        let dyn_len = buff[$dyn_len] as usize;
        let $dyn_field = &buff[$dyn_dw..($dyn_dw + dyn_len)];
        )?

        let pkt = $variant {
        $($($($field),*,)*)?
        $($($f_field),*,)?
        $($($c_field),*,)?
        $($dyn_field)?
        };
        (actual_size, pkt)
    }
    }
};
}

/// Define a unifying enum for every packet type with discriminants for encoding / decoding
///
/// Packets are defined with a header.
///
/// It distinguishes the packet by op, subop and extra bits in first dword.
///
/// If subop is not given it defaults to 0.
///
/// ## Example
/// ```ignore
/// unify!(Pkt<'pkt> {
///     @match_extra op = 1, subop = 0, dw[0] >> 27 & 0x1 => {
///         0 => CopyTiled
///         1 => CopyLinearToTiledBroadcast
///     }
///     op = 0 => Nop<'pkt>
/// });
/// ```
macro_rules! unify {
    (@op_match $op:literal $subop:literal) => {($op, $subop)};
    (@op_match $op:literal) => {($op, 0)};
    (
    $enum_name:ident $(<$life:lifetime>)?
    {
    $(
    @match_extra
    op = $ex_op:literal
    , subop = $ex_subop:literal
    , dw[0] >> $exshift:literal & $exmask:literal => {
        $($ex:literal => $ex_variant:ident $(<$ex_vlife:lifetime>)?)+
    }
    )*
    $(
    op = $op:literal $(, subop = $subop:literal)? => $variant:ident $(<$vlife:lifetime>)?
    )*
    }
) => {
    pub enum $enum_name $(<$life>)? {
    $($($ex_variant($ex_variant $(<$ex_vlife>)?),)*)*
    $($variant($variant $(<$vlife>)?),)*
    }

    impl $(<$life>)? $enum_name $(<$life>)? {
    pub const fn encode_linear(&self, buff: &mut [u32]) -> usize {
    match self {
    $(
    $(
    $enum_name :: $ex_variant(pkt) => { let res = pkt.partial_encode_linear(buff); buff[0] &= !(0xffff | $exmask << $exshift); buff[0] |= $ex_op as u32 | ($ex_subop as u32) << 8 | ($ex as u32) << $exshift;  res},
    )*
    )*

    $(
    $enum_name :: $variant (x) => { let res = x.partial_encode_linear(buff); buff[0] &= !0xffff; buff[0] |= $op as u32 $(| ($subop as u32) << 8)?; res },
    )*

    }
    }

    pub const fn decode_linear(buff: & $($life)? [u32]) -> Option<(usize,$enum_name $(<$life>)?)> {
    if buff.is_empty() {
        panic!("Not even 1dw to read");
    }
    let header = buff[0];
    let op = header as u8;
    let subop = (header >> 8) as u8;
    match (op, subop) {
    $(
    ($ex_op, $ex_subop) => {
    let extra = header >> $exshift & $exmask;
    match extra {
    $(
    $ex => { let (res, pkt) = $ex_variant::partial_decode_linear(buff); Some((res, $enum_name::$ex_variant(pkt))) },
    )*
    _ => panic!("Unhandled")
    }
    }
    )*
    $(
    unify!(@op_match $op $($subop)?) => { let (res, pkt) = $variant::partial_decode_linear(buff); Some((res, $enum_name::$variant(pkt))) },
    )*
    (_, _) => panic!("Unhandled")
    }
    }
    }
};
}

/// GCN 3: Topaz
///
/// See `kernel/drivers/gpu/drm/amd/amdgpu/iceland_sdma_pkt_open.h`
/// It defines ATOMIC op, but has no packet body definition
pub mod v2_4 {
    // Confirmed
    packet!(CopyLinear {
        @bits
        // dw[0] = {
        //     // Not in iceland_sdma_pkt_open
        //     & 0x1 << 25 = backwards: bool;
        // }
        dw[1] = {
            & 0x003FFFFF << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 16 = dst_sw: u8;
            & 0x1 << 22 = dst_ha: bool;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 30 = src_ha: bool;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    // Confirmed
    packet!(CopyLinearBroadcast {
        @bits
        dw[1] = {
            & 0x003FFFFF << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 8  = dst2_sw: u8;
            & 0x1 << 14  = dst2_ha: u8;
            & 0x3 << 16 = dst1_sw: u8;
            & 0x1 << 22  = dst1_ha: u8;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 30 = src_ha: u8;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst1_addr: u64;
        dw[7], dw[8] = dst2_addr: u64;
    });

    // Confirmed
    packet!(CopyTiled {
        @bits
        dw[0] = {
            /// Pehaps the direction
            /// - false: Linear -> Tiled
            /// - true: Tiled -> Linear
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x7ff << 0  = pitch_in_tile: u16;
            & 0x3fff << 16 = height: u16;
        }
        dw[4] = {
            & 0x003fffff << 0 = slice_pitch: u32;
        }
        dw[5] = {
            & 0x7  << 0  = element_size: u8;
            & 0xf  << 3  = array_mode: u8;
            & 0x7  << 8  = mit_mode: u8;
            & 0x7  << 11 = tilesplit_size: u8;
            & 0x3  << 15 = bank_w: u8;
            & 0x3  << 18 = bank_h: u8;
            & 0x3  << 21 = num_bank: u8;
            & 0x3  << 24 = mat_aspt: u8;
            & 0x1f << 26 = pipe_config: u8;
        }
        dw[6] = {
            & 0x3fff << 0  = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0xfff << 0 = z: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[10] = {
            & 0x0007ffff << 0 = linear_pitch: u32;
        }
        dw[11] = {
            & 0x000fffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[8], dw[9] = linear_addr: u64;
    });

    // Confirmed
    packet!(CopyLinearToTiledBroadcast {
        @bits
        dw[0] = {
            // Frame to field in umr
            & 0x1 << 26 = videocopy: bool;
        }
        dw[5] = {
            & 0x7ff << 0 = pitch_in_tile: u16;
            & 0x3fff << 16 = height: u16;
        }
        dw[6] = {
            & 0x003fffff << 0 = slice_pitch: u32;
        }
        dw[7] = {
            & 0x7  << 0  = element_size: u8;
            & 0xf  << 3  = array_mode: u8;
            & 0x7  << 8  = mit_mode: u8;
            & 0x7  << 11 = tilesplit_size: u8;
            & 0x3  << 15 = bank_w: u8;
            & 0x3  << 18 = bank_h: u8;
            & 0x3  << 21 = num_bank: u8;
            & 0x3  << 24 = mat_aspt: u8;
            & 0x1f << 26 = pipe_config: u8;
        }
        dw[8] = {
            & 0x3fff << 0  = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[9] = {
            & 0xfff << 0 = z: u16;
        }
        dw[10] = {
            & 0x3 << 8 = dst2_sw: u8;
            & 0x1 << 14 = dst2_ha: bool;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[13] = {
            & 0x0007ffff << 0 = linear_pitch: u32;
        }
        dw[14] = {
            & 0x000fffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = tiled_addr0: u64;
        dw[3], dw[4] = tiled_addr1: u64;
        dw[11], dw[12] = linear_addr: u64;
    });

    // Confirmed
    packet!(
    /// NOP (variable length)
    Nop<'pkt> {
        @dyn
        dw[1..] = data: &'pkt [u32],
        dw[0] & 0x3fff << 16 = len
    });

    // Confirmed
    packet!(
    /// COPY STRUCT (SOA)
    CopyStruct {
        @bits
        dw[0] = {
            /// Perhaps direction of operation
            & 0x1 << 31 = detile: bool;
        }
        dw[5] = {
            & 0x7ff << 0  = stride: u16;
            // In umr these two are switched
            & 0x3  << 16  = struct_sw: u8;
            & 0x1  << 22  = struct_ha: bool;
            & 0x3  << 24  = linear_sw: u8;
            & 0x1  << 30  = linear_ha: bool;
        }
        @full
        dw[3] = start_index: u32;
        dw[4] = count: u32;
        @join
        dw[1], dw[2] = sb_addr: u64;
        dw[6], dw[7] = linear_addr: u64;
    });

    // Confirmed
    packet!(CopyLinearSubWindow {
        @bits
        dw[0] = {
            & 0x7 << 29 = elementsize: u8;
        }
        dw[3] = {
            & 0x3fff << 0  = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x7ff  << 0  = src_z: u16;
            & 0x3fff << 16 = src_pitch: u16;
        }
        dw[5] = {
            & 0x0fffffff << 0 = src_slice_pitch: u32;
        }
        dw[8] = {
            & 0x3fff << 0  = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[9] = {
            & 0x7ff  << 0  = dst_z: u16;
            & 0x3fff << 16 = dst_pitch: u16;
        }
        dw[10] = {
            & 0x0fffffff << 0 = dst_slice_pitch: u32;
        }
        dw[11] = {
            & 0x3fff << 0  = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[12] = {
            & 0x7ff << 0  = rect_z: u16;
            & 0x3   << 16  = dst_sw: u8;
            & 0x1   << 22  = dst_ha: bool;
            & 0x3   << 24  = src_sw: u8;
            & 0x1   << 30  = src_ha: bool;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[6], dw[7] = dst_addr: u64;
    });

    // Confirmed
    packet!(CopyTiledSubWindow {
        @bits
        dw[0] = {
            /// Perhaps direction
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x3fff << 0  = tiled_x: u16;
            & 0x3fff << 16 = tiled_y: u16;
        }
        dw[4] = {
            & 0x7ff  << 0  = tiled_z: u16;
            & 0xfff << 16 = pitch_in_tile: u16;
        }
        dw[5] = {
            & 0x0fffffff << 0 = slice_pitch: u32;
        }
        dw[6] = {
            & 0x7  << 0  = element_size: u8;
            & 0xf  << 3  = array_mode: u8;
            & 0x7  << 8  = mit_mode: u8;
            & 0x7  << 11 = tilesplit_size: u8;
            & 0x3  << 15 = bank_w: u8;
            & 0x3  << 18 = bank_h: u8;
            & 0x3  << 21 = num_bank: u8;
            & 0x3  << 24 = mat_aspt: u8;
            & 0x1f << 26 = pipe_config: u8;
        }
        dw[9] = {
            & 0x3fff << 0  = linear_x: u16;
            & 0x3fff << 16 = linear_y: u16;
        }
        dw[10] = {
            & 0x7ff  << 0  = linear_z: u16;
            & 0x3fff << 16 = linear_pitch: u16;
        }
        dw[11] = {
            & 0x0fffffff << 0 = linear_slice_pitch: u32;
        }
        dw[12] = {
            & 0x3fff << 0  = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[13] = {
            & 0x7ff << 0   = rect_z: u16;
            & 0x3   << 16  = linear_sw: u8;
            & 0x3   << 22  = tile_sw: u8;
        }
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[7], dw[8] = linear_addr: u64;
    });

    // Confirmed
    packet!(
        /// Copy T2T SubWindow
        CopyTiledToTiled {
        @bits
        dw[3] = {
            & 0x3fff << 0  = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x7ff << 0 = src_z: u16;
            & 0xfff << 16 = src_pitch_in_tile: u16;
        }
        dw[5] = {
            & 0x003fffff << 0 = src_slice_pitch: u32;
        }
        dw[6] = {
            & 0x7  << 0  = src_element_size: u8;
            & 0xf  << 3  = src_array_mode: u8;
            & 0x7  << 8  = src_mit_mode: u8;
            & 0x7  << 11 = src_tilesplit_size: u8;
            & 0x3  << 15 = src_bank_w: u8;
            & 0x3  << 18 = src_bank_h: u8;
            & 0x3  << 21 = src_num_bank: u8;
            & 0x3  << 24 = src_mat_aspt: u8;
            & 0x1f << 26 = src_pipe_config: u8;
        }
        dw[9] = {
            & 0x3fff << 0  = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[10] = {
            & 0x7ff << 0 = dst_z: u16;
            & 0xfff << 16 = dst_pitch_in_tile: u16;
        }
        dw[11] = {
            & 0x003fffff << 0 = dst_slice_pitch: u32;
        }
        dw[12] = {
            & 0x7  << 0  = dst_element_size: u8;
            & 0xf  << 3  = dst_array_mode: u8;
            & 0x7  << 8  = dst_mit_mode: u8;
            & 0x7  << 11 = dst_tilesplit_size: u8;
            & 0x3  << 15 = dst_bank_w: u8;
            & 0x3  << 18 = dst_bank_h: u8;
            & 0x3  << 21 = dst_num_bank: u8;
            & 0x3  << 24 = dst_mat_aspt: u8;
            & 0x1f << 26 = dst_pipe_config: u8;
        }
        dw[13] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[14] = {
            & 0x7ff << 0 = rect_z: u16;
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[7], dw[8] = dst_addr: u64;
    });

    // Confirmed
    packet!(
    /// WRITE LINEAR (with variable data payload)
    WriteUntiled<'pkt> {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
        }
        dw[3] = {
            & 0x3 << 24 = sw: u8;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        @dyn
        dw[4..] = data: &'pkt [u32],
        dw[3] & 0x003fffff << 0 = len
    });

    // Confirmed
    packet!(
    /// WRITE TILED (with variable data payload)
    WriteTiled<'pkt> {
        @bits
        dw[3] = {
            & 0x7ff  << 0  = pitch_in_tile: u16;
            & 0x3fff << 16 = height: u16;
        }
        dw[4] = {
            & 0x003fffff << 0 = slice_pitch: u32;
        }
        dw[5] = {
            & 0x7  << 0  = element_size: u8;
            & 0xf  << 3  = array_mode: u8;
            & 0x7  << 8  = mit_mode: u8;
            & 0x7  << 11 = tilesplit_size: u8;
            & 0x3  << 15 = bank_w: u8;
            & 0x3  << 18 = bank_h: u8;
            & 0x3  << 21 = num_bank: u8;
            & 0x3  << 24 = mat_aspt: u8;
            & 0x1f << 26 = pipe_config: u8;
        }
        dw[6] = {
            & 0x3fff << 0  = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0xfff  << 0  = z: u16;
            & 0x3    << 24 = sw: u8;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        @dyn
        dw[9..] = data: &'pkt [u32],
        dw[8] & 0x003fffff << 0 = len
    });

    // Confirmed
    packet!(IndirectBuffer {
        @bits
        dw[0] = {
            & 0xf << 16 = vmid: u8;
        }
        dw[3] = {
            & 0x000fffff << 0 = ib_size: u32;
        }
        @join
        dw[1], dw[2] = ib_base: u64;
        // In umr ib_csa_addr
        dw[4], dw[5] = csa_addr: u64;
    });

    // Confirmed
    packet!(Fence {
        @full
        dw[3] = data: u32;
        @join
        dw[1], dw[2] = addr: u64;
    });

    // Confirmed
    packet!(Trap {
        @bits
        dw[1] = {
            & 0x0fffffff << 0 = int_context: u32;
        }
    });

    // Confirmed
    packet!(Semaphore {
        @bits
        dw[0] = {
            & 0x1 << 29 = write_one: bool;
            & 0x1 << 30 = signal: bool;
            & 0x1 << 31 = mailbox: bool;
        }
        @join
        dw[1], dw[2] = addr: u64;
    });

    // Confirmed
    packet!(PollRegmem {
        @bits
        dw[0] = {
            & 0x1 << 26 = hdp_flush: bool;
            & 0x7 << 28 = function: u8;
            & 0x1 << 31 = mem_poll: bool;
        }
        dw[5] = {
            & 0xffff << 0  = interval: u16;
            & 0xfff  << 16 = retry_count: u16;
        }
        @full
        dw[3] = value: u32;
        dw[4] = mask: u32;
        @join
        dw[1], dw[2] = addr_or_reg: u64;
    });

    // Confirmed
    packet!(CondExe {
        @bits
        dw[4] = {
            & 0x3fff << 0 = exec_count: u16;
        }
        @full
        dw[3] = reference: u32;
        @join
        dw[1], dw[2] = addr: u64;
    });

    // Confirmed
    packet!(ConstFill {
        @bits
        dw[0] = {
            // swap
            & 0x3 << 16 = sw: u8;
            & 0x3 << 30 = fill_size: u8;
        }
        dw[4] = {
            & 0x003fffff << 0 = byte_count: u32;
        }
        @full
        dw[3] = data: u32;
        @join
        dw[1], dw[2] = dst_addr: u64;
    });

    // Confirmed
    packet!(
    /// SDMA_PKT_WRITE_INCR
    GenPtepde {
        @bits
        dw[9] = {
            & 0x0007ffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        dw[3], dw[4] = mask: u64;
        dw[5], dw[6] = init: u64;
        dw[7], dw[8] = incr: u64;
    });

    // Confirmed
    packet!(TimestampSet {
        @join
        dw[1], dw[2] = init_data: u64;
    });

    // Confirmed
    packet!(TimestampGet {
        @join
        /// Needs to be 8byte aligned
        dw[1], dw[2] = write_addr: u64;
    });

    // Confirmed
    packet!(TimestampGetGlobal {
        @join
        /// Needs to be 8byte aligned
        dw[1], dw[2] = write_addr: u64;
    });

    // Confirmed
    packet!(
        /// Register write
        SrbmWrite {
        @bits
        dw[0] = {
            & 0xf << 28 = byte_enable: u8;
        }
        dw[1] = {
            /// Register
            & 0xffff << 0 = addr: u16;
        }
        @full
        dw[2] = data: u32;
    });

    // Confirmed
    packet!(PreExe {
        @bits
        dw[0] = {
            & 0xff << 16 = dev_sel: u8;
        }
        dw[1] = {
            & 0x3fff << 0 = exec_count: u16;
        }
    });

    unify!(Pkt<'pkt> {
        @match_extra op =1, subop = 0, dw[0] >> 27 & 0x1 => {
            0 => CopyLinear
            1 => CopyLinearBroadcast
        }

        // discriminant not confirmed
        // 27 -> broadcast
        // 26 -> videocopy (frame_to_field in umr)
        //
        // Since it uses the same mem layout in umr irrespective of videocopy I'm
        // going to treat it as if only broadcast is a discriminant
        @match_extra op = 1, subop = 1, dw[0] >> 27 & 0x1 => {
            0 => CopyTiled
            1 => CopyLinearToTiledBroadcast
        }
        op = 0 => Nop<'pkt>
        op = 1, subop = 3 => CopyStruct
        op = 1, subop = 4 => CopyLinearSubWindow
        op = 1, subop = 5 => CopyTiledSubWindow
        op = 1, subop = 6 => CopyTiledToTiled
        op = 2, subop = 0 => WriteUntiled<'pkt>
        op = 2, subop = 1 => WriteTiled<'pkt>
        op = 4 => IndirectBuffer
        op = 5 => Fence
        op = 6 => Trap
        op = 7, subop = 0 => Semaphore
        op = 8, subop = 0 => PollRegmem
        op = 9 => CondExe
        op = 11, subop = 0 => ConstFill
        op = 12, subop = 0 => GenPtepde
        op = 13, subop = 0 => TimestampSet
        op = 13, subop = 1 => TimestampGet
        op = 13, subop = 2 => TimestampGetGlobal
        op = 14, subop = 0 => SrbmWrite
        op = 15 => PreExe
    });
}

/// GCN 3, 4: Tonga, Corrizo, Fiji, Stoney, Polaris 10, Polaris 12, Vegam
///
/// ## Changes
/// - added ATOMIC
pub mod v3_0 {
    pub use super::v2_4::{
        CondExe, ConstFill, CopyLinear, CopyLinearBroadcast, CopyLinearSubWindow,
        CopyLinearToTiledBroadcast, CopyStruct, CopyTiled, CopyTiledSubWindow, CopyTiledToTiled,
        Fence, GenPtepde, IndirectBuffer, Nop, PollRegmem, PreExe, Semaphore, SrbmWrite,
        TimestampGet, TimestampGetGlobal, TimestampSet, Trap, WriteTiled, WriteUntiled,
    };

    // Confirmed
    packet!(Atomic {
        @bits
        dw[0] = {
            & 0x1 << 16 = r#loop: bool;
            & 0x7f << 25 = op: u8;
        }
        dw[7] = {
            & 0x1fff << 0 = loop_interval: u16;
        }
        @join
        dw[1], dw[2] = addr: u64;
        dw[3], dw[4] = src_data: u64;
        dw[5], dw[6] = cmp_data: u64;
    });

    unify!(Pkt<'pkt> {
        @match_extra op =1, subop = 0, dw[0] >> 27 & 0x1 => {
            0 => CopyLinear
            1 => CopyLinearBroadcast
        }

        // discriminant not confirmed
        // 27 -> broadcast
        // 26 -> videocopy (frame_to_field in umr)
        //
        // Since it uses the same mem layout in umr irrespective of videocopy I'm
        // going to treat it as if only broadcast is a discriminant
        @match_extra op = 1, subop = 1, dw[0] >> 27 & 0x1 => {
            0 => CopyTiled
            1 => CopyLinearToTiledBroadcast
        }
        op = 0 => Nop<'pkt>
        op = 1, subop = 3 => CopyStruct
        op = 1, subop = 4 => CopyLinearSubWindow
        op = 1, subop = 5 => CopyTiledSubWindow
        op = 1, subop = 6 => CopyTiledToTiled
        op = 2, subop = 0 => WriteUntiled<'pkt>
        op = 2, subop = 1 => WriteTiled<'pkt>
        op = 4 => IndirectBuffer
        op = 5 => Fence
        op = 6 => Trap
        op = 7, subop = 0 => Semaphore
        op = 8, subop = 0 => PollRegmem
        op = 9 => CondExe
        op = 10 => Atomic
        op = 11, subop = 0 => ConstFill
        op = 12, subop = 0 => GenPtepde
        op = 13, subop = 0 => TimestampSet
        op = 13, subop = 1 => TimestampGet
        op = 13, subop = 2 => TimestampGetGlobal
        op = 14, subop = 0 => SrbmWrite
        op = 15 => PreExe
    });
}

/// GCN 5 (v4.0 - v4.4.0): Vega 10, Raven, Vega 12, Raven 2, Picasso, Renoir, Vega 20, Arcturus, Aldebaran
///
/// ## Changes
/// - added DUMMY_TRAP
/// - renamed GEN_PTEPDE to PTEPDE
/// - added SDMA_SUBOP_COPY_DIRTY_PAGE
/// - added SDMA_SUBOP_COPY_LINEAR_PHY
/// - added SDMA_SUBOP_PTEPDE_COPY
/// - added SDMA_SUBOP_PTEPDE_RMW
/// - added SDMA_SUBOP_PTEPDE_COPY_BACKWARDS
/// - added SDMA_SUBOP_DATA_FILL_MULTI
/// - added SDMA_SUBOP_POLL_REG_WRITE_MEM
/// - added SDMA_SUBOP_POLL_DBIT_WRITE_MEM
/// - added SDMA_SUBOP_POLL_MEM_VERIFY
/// - COPY_LINEAR:
///     * added `encrypt`
///     * added `tmz`
///     * removed `src_ha`
///     * removed `dst_ha`
/// - COPY_BROADCAST:
///     * added `encrypt`
///     * added `tmz`
///     * removed `src_ha`
///     * removed `dst1_ha`
///     * removed `dst2_ha`
/// - COPY_LINEAR_SUBWIN:
///     * added `tmz`
///     * increased `src_pitch` size
///     * increased `dst_pitch` size
///     * removed `src_ha`
///     * removed `dst_ha`
/// - COPY_TILED, COPY_L2T_BROADCAST, COPY_T2T, COPY_TILED_SUBWIN:
///     * added `encrypt`
///     * added `tmz`
///     * decreased `z` size
///     * **Major Layout Change:** Exact field-to-field mapping is pending clarification
/// - COPY_STRUCT
///     * added `tmz`
///     * removed `struct_ha`
///     * removed `linear_ha`
///     * moved `struct_sw`
///     * moved `linear_sw`
/// - WRITE_UNTILED
///     * added `encrypt`
///     * added `tmz`
///     * decreased `count` size
/// - WRITE_TILED
///     * **Major Layout Change:** Exact field-to-field mapping is pending clarification
/// - SRBM_WRITE
///     * increased `addr` size
/// - ATOMIC
///     * added `tmz`
pub mod v4_0 {

    packet!(CopyLinear {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 27 = broadcast: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    packet!(CopyDirtyPage {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 31 = all: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 16 = dst_sw: u8;
            & 0x1 << 19 = dst_gcc: bool;
            & 0x1 << 20 = dst_sys: bool;
            & 0x1 << 22 = dst_snoop: bool;
            & 0x1 << 23 = dst_gpa: bool;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 28 = src_sys: bool;
            & 0x1 << 30 = src_snoop: bool;
            & 0x1 << 31 = src_gpa: bool;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    packet!(CopyPhysicalLinear {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 16 = dst_sw: u8;
            & 0x1 << 19 = dst_gcc: bool;
            & 0x1 << 20 = dst_sys: bool;
            & 0x1 << 21 = dst_log: bool;
            & 0x1 << 22 = dst_snoop: bool;
            & 0x1 << 23 = dst_gpa: bool;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 27 = src_gcc: bool;
            & 0x1 << 28 = src_sys: bool;
            & 0x1 << 30 = src_snoop: bool;
            & 0x1 << 31 = src_gpa: bool;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    packet!(CopyBroadcastLinear {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 27 = broadcast: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 8 = dst2_sw: u8;
            & 0x3 << 16 = dst1_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst1_addr: u64;
        dw[7], dw[8] = dst2_addr: u64;
    });

    packet!(CopyLinearSubwin {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x7 << 29 = elementsize: u8;
        }
        dw[3] = {
            & 0x3fff << 0 = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x7ff << 0 = src_z: u16;
            & 0x7ffff << 13 = src_pitch: u32;
        }
        dw[5] = {
            & 0xfffffff << 0 = src_slice_pitch: u32;
        }
        dw[8] = {
            & 0x3fff << 0 = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[9] = {
            & 0x7ff << 0 = dst_z: u16;
            & 0x7ffff << 13 = dst_pitch: u32;
        }
        dw[10] = {
            & 0xfffffff << 0 = dst_slice_pitch: u32;
        }
        dw[11] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[12] = {
            & 0x7ff << 0 = rect_z: u16;
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[6], dw[7] = dst_addr: u64;
    });

    packet!(CopyTiled {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0xf << 20 = mip_max: u8;
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[4] = {
            & 0x3fff << 0 = height: u16;
            & 0x7ff << 16 = depth: u16;
        }
        dw[5] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xffff << 16 = epitch: u16;
        }
        dw[6] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0x7ff << 0 = z: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[10] = {
            & 0x7ffff << 0 = linear_pitch: u32;
        }
        dw[12] = {
            & 0xfffff << 0 = count: u32;
        }
        @full
        dw[11] = linear_slice_pitch: u32;
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[8], dw[9] = linear_addr: u64;
    });

    packet!(CopyL2tBroadcast {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0xf << 20 = mip_max: u8;
            & 0x1 << 26 = videocopy: bool;
            & 0x1 << 27 = broadcast: bool;
        }
        dw[5] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[6] = {
            & 0x3fff << 0 = height: u16;
            & 0x7ff << 16 = depth: u16;
        }
        dw[7] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xffff << 16 = epitch: u16;
        }
        dw[8] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[9] = {
            & 0x7ff << 0 = z: u16;
        }
        dw[10] = {
            & 0x3 << 8 = dst2_sw: u8;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[13] = {
            & 0x7ffff << 0 = linear_pitch: u32;
        }
        dw[15] = {
            & 0xfffff << 0 = count: u32;
        }
        @full
        dw[14] = linear_slice_pitch: u32;
        @join
        dw[1], dw[2] = tiled_addr0: u64;
        dw[3], dw[4] = tiled_addr1: u64;
        dw[11], dw[12] = linear_addr: u64;
    });

    packet!(CopyT2t {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0xf << 20 = mip_max: u8;
        }
        dw[3] = {
            & 0x3fff << 0 = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x7ff << 0 = src_z: u16;
            & 0x3fff << 16 = src_width: u16;
        }
        dw[5] = {
            & 0x3fff << 0 = src_height: u16;
            & 0x7ff << 16 = src_depth: u16;
        }
        dw[6] = {
            & 0x7 << 0 = src_element_size: u8;
            & 0x1f << 3 = src_swizzle_mode: u8;
            & 0x3 << 9 = src_dimension: u8;
            & 0xffff << 16 = src_epitch: u16;
        }
        dw[9] = {
            & 0x3fff << 0 = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[10] = {
            & 0x7ff << 0 = dst_z: u16;
            & 0x3fff << 16 = dst_width: u16;
        }
        dw[11] = {
            & 0x3fff << 0 = dst_height: u16;
            & 0x7ff << 16 = dst_depth: u16;
        }
        dw[12] = {
            & 0x7 << 0 = dst_element_size: u8;
            & 0x1f << 3 = dst_swizzle_mode: u8;
            & 0x3 << 9 = dst_dimension: u8;
            & 0xffff << 16 = dst_epitch: u16;
        }
        dw[13] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[14] = {
            & 0x7ff << 0 = rect_z: u16;
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[7], dw[8] = dst_addr: u64;
    });

    packet!(CopyTiledSubwin {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0xf << 20 = mip_max: u8;
            & 0xf << 24 = mip_id: u8;
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = tiled_x: u16;
            & 0x3fff << 16 = tiled_y: u16;
        }
        dw[4] = {
            & 0x7ff << 0 = tiled_z: u16;
            & 0x3fff << 16 = width: u16;
        }
        dw[5] = {
            & 0x3fff << 0 = height: u16;
            & 0x7ff << 16 = depth: u16;
        }
        dw[6] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xffff << 16 = epitch: u16;
        }
        dw[9] = {
            & 0x3fff << 0 = linear_x: u16;
            & 0x3fff << 16 = linear_y: u16;
        }
        dw[10] = {
            & 0x7ff << 0 = linear_z: u16;
            & 0x3fff << 16 = linear_pitch: u16;
        }
        dw[11] = {
            & 0xfffffff << 0 = linear_slice_pitch: u32;
        }
        dw[12] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[13] = {
            & 0x7ff << 0 = rect_z: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[7], dw[8] = linear_addr: u64;
    });

    packet!(CopyStruct {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 31 = detile: bool;
        }
        dw[5] = {
            & 0x7ff << 0 = stride: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = struct_sw: u8;
        }
        @full
        dw[3] = start_index: u32;
        dw[4] = count: u32;
        @join
        dw[1], dw[2] = sb_addr: u64;
        dw[6], dw[7] = linear_addr: u64;
    });

    packet!(WriteUntiled<'a> {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
        }
        dw[3] = {
            & 0x3 << 24 = sw: u8;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        @dyn
        dw[4..] = data: &'a [u32],
        dw[3] & 0xfffff << 0 = len
    });

    packet!(WriteTiled<'a> {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0xf << 20 = mip_max: u8;
        }
        dw[3] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[4] = {
            & 0x3fff << 0 = height: u16;
            & 0x7ff << 16 = depth: u16;
        }
        dw[5] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xffff << 16 = epitch: u16;
        }
        dw[6] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0x7ff << 0 = z: u16;
            & 0x3 << 24 = sw: u8;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        @dyn
        dw[9..] = data: &'a [u32],
        dw[8] & 0xfffff << 0 = len
    });

    packet!(PtepdeCopy {
        @bits
        dw[0] = {
            & 0x1 << 31 = ptepde_op: bool;
        }
        dw[7] = {
            & 0x7ffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[3], dw[4] = dst_addr: u64;
        dw[5], dw[6] = mask: u64;
    });

    packet!(PtepdeCopyBackwards {
        @bits
        dw[0] = {
            & 0x3 << 28 = pte_size: u8;
            & 0x1 << 30 = direction: bool;
            & 0x1 << 31 = ptepde_op: bool;
        }
        dw[5] = {
            & 0xff << 0 = mask_first_xfer: u8;
            & 0xff << 8 = mask_last_xfer: u8;
        }
        dw[6] = {
            & 0x1ffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[3], dw[4] = dst_addr: u64;
    });

    packet!(PtepdeRmw {
        @bits
        dw[0] = {
            & 0x1 << 19 = gcc: bool;
            & 0x1 << 20 = sys: bool;
            & 0x1 << 22 = snp: bool;
            & 0x1 << 23 = gpa: bool;
        }
        @join
        dw[1], dw[2] = addr: u64;
        dw[3], dw[4] = mask: u64;
        dw[5], dw[6] = value: u64;
    });
    pub use super::v3_0::GenPtepde as WriteIncr;

    pub use super::v3_0::IndirectBuffer;

    pub use super::v3_0::Semaphore;

    pub use super::v3_0::Fence;

    packet!(SrbmWrite {
        @bits
        dw[0] = {
            & 0xf << 28 = byte_en: u8;
        }
        dw[1] = {
            // Increased size
            & 0x3ffff << 0 = addr: u32;
        }
        @full
        dw[2] = data: u32;
    });

    pub use super::v3_0::PreExe;

    pub use super::v3_0::CondExe;

    pub use super::v3_0::ConstFill;

    // Needs confirmation
    packet!(DataFillMulti<'pkt> {
        @bits
        dw[0] = {
            & 0x1 << 31 = memlog_clr: bool;
        }
        @full
        dw[1] = byte_stride: u32;
        dw[2] = dma_count: u32;
        @join
        dw[3], dw[4] = dst_addr: u64;
        @dyn
        dw[6..] = data: &'pkt [u32],
        dw[5] & 0x3ffffff << 0 = len
    });

    pub use super::v3_0::PollRegmem;

    packet!(PollRegWriteMem {
        @bits
        dw[1] = {
            & 0x3fffffff << 2 = src_addr: u32;
        }
        @join
        dw[2], dw[3] = dst_addr: u64;
    });

    packet!(PollDbitWriteMem {
        @bits
        dw[0] = {
            & 0x3 << 16 = ea: u8;
        }
        dw[3] = {
            & 0xfffffff << 4 = start_page: u32;
        }
        @full
        dw[4] = page_num: u32;
        @join
        dw[1], dw[2] = dst_addr: u64;
    });

    packet!(PollMemVerify {
        @bits
        dw[0] = {
            & 0x1 << 31 = mode: bool;
        }
        @full
        dw[1] = pattern: u32;
        dw[12] = reserved: u32;
        @join
        dw[2], dw[3] = cmp0_start: u64;
        dw[4], dw[5] = cmp0_end: u64;
        dw[6], dw[7] = cmp1_start: u64;
        dw[8], dw[9] = cmp1_end: u64;
        dw[10], dw[11] = rec: u64;
    });

    packet!(Atomic {
        @bits
        dw[0] = {
            & 0x1 << 16 = r#loop: bool;
            // New
            & 0x1 << 18 = tmz: bool;
            & 0x7f << 25 = atomic_op: u8;
        }
        dw[7] = {
            & 0x1fff << 0 = loop_interval: u16;
        }
        @join
        dw[1], dw[2] = addr: u64;
        dw[3], dw[4] = src_data: u64;
        dw[5], dw[6] = cmp_data: u64;
    });

    packet!(DummyTrap {
        @bits
        dw[1] = {
            & 0xfffffff << 0 = int_context: u32;
        }
    });

    pub use super::v3_0::Nop;
    pub use super::v3_0::TimestampGet;
    pub use super::v3_0::TimestampGetGlobal;
    pub use super::v3_0::TimestampSet;
    pub use super::v3_0::Trap;

    unify!(Pkt<'pkt> {
        @match_extra op =1, subop = 0, dw[0] >> 27 & 0x1 => {
            0 => CopyLinear
            1 => CopyBroadcastLinear
        }

        // discriminant not confirmed
        // 27 -> broadcast
        // 26 -> videocopy (frame_to_field in umr)
        //
        // Since it uses the same mem layout in umr irrespective of videocopy I'm
        // going to treat it as if only broadcast is a discriminant
        @match_extra op = 1, subop = 1, dw[0] >> 27 & 0x1 => {
            0 => CopyTiled
            1 => CopyL2tBroadcast
        }
        op = 0 => Nop<'pkt>
        op = 1, subop = 3 => CopyStruct
        op = 1, subop = 4 => CopyLinearSubwin
        op = 1, subop = 5 => CopyTiledSubwin
        op = 1, subop = 6 => CopyT2t
        op = 1, subop = 7 => CopyDirtyPage
        op = 1, subop = 8 => CopyPhysicalLinear
        //op = 1, subop = 16 => CopyLinearBc
        //op = 1, subop = 17 => CopyTiledBc
        //op = 1, subop = 20 => CopyLinearSubwindBc
        //op = 1, subop = 21 => CopyTiledSubwindBc
        //op = 1, subop = 22 => CopyT2tSubwindBc
        //op = 1, subop = 36 => CopySubwinLarge
        op = 2, subop = 0 => WriteUntiled<'pkt>
        op = 2, subop = 1 => WriteTiled<'pkt>
        //op = 2, subop = 17 => WriteTiledBc<'pkt>
        op = 4 => IndirectBuffer
        op = 5 => Fence
        op = 6 => Trap
        op = 7, subop = 0 => Semaphore
        //op = 7, subop = 1 => MemIncr
        op = 8, subop = 0 => PollRegmem
        op = 8, subop = 1 => PollRegWriteMem
        op = 8, subop = 2 => PollDbitWriteMem
        op = 8, subop = 3 => PollMemVerify
        //op = 8, subop = 4 => Invalidation
        op = 9 => CondExe
        op = 10 => Atomic
        op = 11, subop = 0 => ConstFill
        op = 11, subop = 1 => DataFillMulti<'pkt>
        op = 12, subop = 0 => WriteIncr
        op = 12, subop = 1 => PtepdeCopy
        op = 12, subop = 2 => PtepdeRmw
        // Not in UMR
        op = 12, subop = 3 => PtepdeCopyBackwards
        op = 13, subop = 0 => TimestampSet
        op = 13, subop = 1 => TimestampGet
        op = 13, subop = 2 => TimestampGetGlobal
        op = 14, subop = 0 => SrbmWrite
        op = 15 => PreExe
    });
}

/// GCN 5 (v4.4.2, v4.4.3, v4.4.4)
///
/// ## Changes
/// - COPY_LINEAR: dst and src cache policy
/// - COPY_LINEAR_BROADCAST: dst, dst2, src cache policy
pub mod v4_4 {
    pub use super::v4_0::*;
    packet!(CopyLinear {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 27 = broadcast: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 16 = dst_sw: u8;
            & 0xf << 18 = dst_cache_policy: u8;
            & 0x3 << 24 = src_sw: u8;
            & 0xf << 26 = src_cache_policy: u8;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });
}

/// Rdna 1: Navi 14, Navi 12, Navi 10, Cyan Skillfish, Cyan Skillfish 2
///
/// See `kernel/drivers/gpu/drm/amd/amdgpu/navi10_sdma_pkt_open.h`
///
/// ## Changes
pub mod v5_0 {
    packet!(CopyLinear {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 25 = backwards: bool;
            & 0x1 << 27 = broadcast: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    packet!(CopyLinearBc {
        @bits
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 16 = dst_sw: u8;
            & 0x1 << 22 = dst_ha: bool;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 30 = src_ha: bool;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    packet!(CopyDirtyPage {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 31 = all: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x7 << 3 = dst_mtype: u8;
            & 0x3 << 6 = dst_l2_policy: u8;
            & 0x7 << 11 = src_mtype: u8;
            & 0x3 << 14 = src_l2_policy: u8;
            & 0x3 << 16 = dst_sw: u8;
            & 0x1 << 19 = dst_gcc: bool;
            & 0x1 << 20 = dst_sys: bool;
            & 0x1 << 22 = dst_snoop: bool;
            & 0x1 << 23 = dst_gpa: bool;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 28 = src_sys: bool;
            & 0x1 << 30 = src_snoop: bool;
            & 0x1 << 31 = src_gpa: bool;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    packet!(CopyPhysicalLinear {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x7 << 3 = dst_mtype: u8;
            & 0x3 << 6 = dst_l2_policy: u8;
            & 0x7 << 11 = src_mtype: u8;
            & 0x3 << 14 = src_l2_policy: u8;
            & 0x3 << 16 = dst_sw: u8;
            & 0x1 << 19 = dst_gcc: bool;
            & 0x1 << 20 = dst_sys: bool;
            & 0x1 << 21 = dst_log: bool;
            & 0x1 << 22 = dst_snoop: bool;
            & 0x1 << 23 = dst_gpa: bool;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 27 = src_gcc: bool;
            & 0x1 << 28 = src_sys: bool;
            & 0x1 << 30 = src_snoop: bool;
            & 0x1 << 31 = src_gpa: bool;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst_addr: u64;
    });

    packet!(CopyBroadcastLinear {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 27 = broadcast: bool;
        }
        dw[1] = {
            & 0x3fffff << 0 = count: u32;
        }
        dw[2] = {
            & 0x3 << 8 = dst2_sw: u8;
            & 0x3 << 16 = dst1_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[3], dw[4] = src_addr: u64;
        dw[5], dw[6] = dst1_addr: u64;
        dw[7], dw[8] = dst2_addr: u64;
    });

    packet!(CopyLinearSubwin {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x7 << 29 = elementsize: u8;
        }
        dw[3] = {
            & 0x3fff << 0 = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x1fff << 0 = src_z: u16;
            & 0x7ffff << 13 = src_pitch: u32;
        }
        dw[5] = {
            & 0xfffffff << 0 = src_slice_pitch: u32;
        }
        dw[8] = {
            & 0x3fff << 0 = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[9] = {
            & 0x1fff << 0 = dst_z: u16;
            & 0x7ffff << 13 = dst_pitch: u32;
        }
        dw[10] = {
            & 0xfffffff << 0 = dst_slice_pitch: u32;
        }
        dw[11] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[12] = {
            & 0x1fff << 0 = rect_z: u16;
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[6], dw[7] = dst_addr: u64;
    });

    packet!(CopyLinearSubwinBc {
        @bits
        dw[0] = {
            & 0x7 << 29 = elementsize: u8;
        }
        dw[3] = {
            & 0x3fff << 0 = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x7ff << 0 = src_z: u16;
            & 0x3fff << 13 = src_pitch: u16;
        }
        dw[5] = {
            & 0xfffffff << 0 = src_slice_pitch: u32;
        }
        dw[8] = {
            & 0x3fff << 0 = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[9] = {
            & 0x7ff << 0 = dst_z: u16;
            & 0x3fff << 13 = dst_pitch: u16;
        }
        dw[10] = {
            & 0xfffffff << 0 = dst_slice_pitch: u32;
        }
        dw[11] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[12] = {
            & 0x7ff << 0 = rect_z: u16;
            & 0x3 << 16 = dst_sw: u8;
            & 0x1 << 22 = dst_ha: bool;
            & 0x3 << 24 = src_sw: u8;
            & 0x1 << 30 = src_ha: bool;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[6], dw[7] = dst_addr: u64;
    });

    packet!(CopyTiled {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[4] = {
            & 0x3fff << 0 = height: u16;
            & 0x1fff << 16 = depth: u16;
        }
        dw[5] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xf << 16 = mip_max: u8;
        }
        dw[6] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0x1fff << 0 = z: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x1 << 20 = linear_cc: bool;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[10] = {
            & 0x7ffff << 0 = linear_pitch: u32;
        }
        dw[12] = {
            & 0x3fffff << 0 = count: u32;
        }
        @full
        dw[11] = linear_slice_pitch: u32;
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[8], dw[9] = linear_addr: u64;
    });

    packet!(CopyTiledBc {
        @bits
        dw[0] = {
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[4] = {
            & 0x3fff << 0 = height: u16;
            & 0x7ff << 16 = depth: u16;
        }
        dw[5] = {
            & 0x7 << 0 = element_size: u8;
            & 0xf << 3 = array_mode: u8;
            & 0x7 << 8 = mit_mode: u8;
            & 0x7 << 11 = tilesplit_size: u8;
            & 0x3 << 15 = bank_w: u8;
            & 0x3 << 18 = bank_h: u8;
            & 0x3 << 21 = num_bank: u8;
            & 0x3 << 24 = mat_aspt: u8;
            & 0x1f << 26 = pipe_config: u8;
        }
        dw[6] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0x7ff << 0 = z: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[10] = {
            & 0x7ffff << 0 = linear_pitch: u32;
        }
        dw[11] = {
            & 0xfffff << 2 = count: u32;
        }
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[8], dw[9] = linear_addr: u64;
    });

    packet!(CopyL2tBroadcast {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 26 = videocopy: bool;
            & 0x1 << 27 = broadcast: bool;
        }
        dw[5] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[6] = {
            & 0x3fff << 0 = height: u16;
            & 0x1fff << 16 = depth: u16;
        }
        dw[7] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xf << 16 = mip_max: u8;
        }
        dw[8] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[9] = {
            & 0x1fff << 0 = z: u16;
        }
        dw[10] = {
            & 0x3 << 8 = dst2_sw: u8;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[13] = {
            & 0x7ffff << 0 = linear_pitch: u32;
        }
        dw[15] = {
            & 0x3fffff << 0 = count: u32;
        }
        @full
        dw[14] = linear_slice_pitch: u32;
        @join
        dw[1], dw[2] = tiled_addr0: u64;
        dw[3], dw[4] = tiled_addr1: u64;
        dw[11], dw[12] = linear_addr: u64;
    });

    packet!(CopyT2t {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 19 = dcc: bool;
            & 0x1 << 31 = dcc_dir: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x1fff << 0 = src_z: u16;
            & 0x3fff << 16 = src_width: u16;
        }
        dw[5] = {
            & 0x3fff << 0 = src_height: u16;
            & 0x1fff << 16 = src_depth: u16;
        }
        dw[6] = {
            & 0x7 << 0 = src_element_size: u8;
            & 0x1f << 3 = src_swizzle_mode: u8;
            & 0x3 << 9 = src_dimension: u8;
            & 0xf << 16 = src_mip_max: u8;
            & 0xf << 20 = src_mip_id: u8;
        }
        dw[9] = {
            & 0x3fff << 0 = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[10] = {
            & 0x1fff << 0 = dst_z: u16;
            & 0x3fff << 16 = dst_width: u16;
        }
        dw[11] = {
            & 0x3fff << 0 = dst_height: u16;
            & 0x1fff << 16 = dst_depth: u16;
        }
        dw[12] = {
            & 0x7 << 0 = dst_element_size: u8;
            & 0x1f << 3 = dst_swizzle_mode: u8;
            & 0x3 << 9 = dst_dimension: u8;
            & 0xf << 16 = dst_mip_max: u8;
            & 0xf << 20 = dst_mip_id: u8;
        }
        dw[13] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[14] = {
            & 0x1fff << 0 = rect_z: u16;
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        dw[17] = {
            & 0x7f << 0 = data_format: u8;
            & 0x1 << 7 = color_transform_disable: bool;
            & 0x1 << 8 = alpha_is_on_msb: bool;
            & 0x7 << 9 = number_type: u8;
            & 0x3 << 12 = surface_type: u8;
            & 0x3 << 24 = max_comp_block_size: u8;
            & 0x3 << 26 = max_uncomp_block_size: u8;
            & 0x1 << 28 = write_compress_enable: bool;
            & 0x1 << 29 = meta_tmz: bool;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[7], dw[8] = dst_addr: u64;
        dw[15], dw[16] = meta_addr: u64;
    });

    packet!(CopyT2tBc {
        @bits
        dw[3] = {
            & 0x3fff << 0 = src_x: u16;
            & 0x3fff << 16 = src_y: u16;
        }
        dw[4] = {
            & 0x7ff << 0 = src_z: u16;
            & 0x3fff << 16 = src_width: u16;
        }
        dw[5] = {
            & 0x3fff << 0 = src_height: u16;
            & 0x7ff << 16 = src_depth: u16;
        }
        dw[6] = {
            & 0x7 << 0 = src_element_size: u8;
            & 0xf << 3 = src_array_mode: u8;
            & 0x7 << 8 = src_mit_mode: u8;
            & 0x7 << 11 = src_tilesplit_size: u8;
            & 0x3 << 15 = src_bank_w: u8;
            & 0x3 << 18 = src_bank_h: u8;
            & 0x3 << 21 = src_num_bank: u8;
            & 0x3 << 24 = src_mat_aspt: u8;
            & 0x1f << 26 = src_pipe_config: u8;
        }
        dw[9] = {
            & 0x3fff << 0 = dst_x: u16;
            & 0x3fff << 16 = dst_y: u16;
        }
        dw[10] = {
            & 0x7ff << 0 = dst_z: u16;
            & 0x3fff << 16 = dst_width: u16;
        }
        dw[11] = {
            & 0x3fff << 0 = dst_height: u16;
            & 0xfff << 16 = dst_depth: u16;
        }
        dw[12] = {
            & 0x7 << 0 = dst_element_size: u8;
            & 0xf << 3 = dst_array_mode: u8;
            & 0x7 << 8 = dst_mit_mode: u8;
            & 0x7 << 11 = dst_tilesplit_size: u8;
            & 0x3 << 15 = dst_bank_w: u8;
            & 0x3 << 18 = dst_bank_h: u8;
            & 0x3 << 21 = dst_num_bank: u8;
            & 0x3 << 24 = dst_mat_aspt: u8;
            & 0x1f << 26 = dst_pipe_config: u8;
        }
        dw[13] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[14] = {
            & 0x7ff << 0 = rect_z: u16;
            & 0x3 << 16 = dst_sw: u8;
            & 0x3 << 24 = src_sw: u8;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[7], dw[8] = dst_addr: u64;
    });

    packet!(CopyTiledSubwin {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 19 = dcc: bool;
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = tiled_x: u16;
            & 0x3fff << 16 = tiled_y: u16;
        }
        dw[4] = {
            & 0x1fff << 0 = tiled_z: u16;
            & 0x3fff << 16 = width: u16;
        }
        dw[5] = {
            & 0x3fff << 0 = height: u16;
            & 0x1fff << 16 = depth: u16;
        }
        dw[6] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xf << 16 = mip_max: u8;
            & 0xf << 20 = mip_id: u8;
        }
        dw[9] = {
            & 0x3fff << 0 = linear_x: u16;
            & 0x3fff << 16 = linear_y: u16;
        }
        dw[10] = {
            & 0x1fff << 0 = linear_z: u16;
            & 0x3fff << 16 = linear_pitch: u16;
        }
        dw[11] = {
            & 0xfffffff << 0 = linear_slice_pitch: u32;
        }
        dw[12] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[13] = {
            & 0x1fff << 0 = rect_z: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        dw[16] = {
            & 0x7f << 0 = data_format: u8;
            & 0x1 << 7 = color_transform_disable: bool;
            & 0x1 << 8 = alpha_is_on_msb: bool;
            & 0x7 << 9 = number_type: u8;
            & 0x3 << 12 = surface_type: u8;
            & 0x3 << 24 = max_comp_block_size: u8;
            & 0x3 << 26 = max_uncomp_block_size: u8;
            & 0x1 << 28 = write_compress_enable: bool;
            & 0x1 << 29 = meta_tmz: bool;
        }
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[7], dw[8] = linear_addr: u64;
        dw[14], dw[15] = meta_addr: u64;
    });

    packet!(CopyTiledSubwinBc {
        @bits
        dw[0] = {
            & 0x1 << 31 = detile: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = tiled_x: u16;
            & 0x3fff << 16 = tiled_y: u16;
        }
        dw[4] = {
            & 0x7ff << 0 = tiled_z: u16;
            & 0x3fff << 16 = width: u16;
        }
        dw[5] = {
            & 0x3fff << 0 = height: u16;
            & 0x7ff << 16 = depth: u16;
        }
        dw[6] = {
            & 0x7 << 0 = element_size: u8;
            & 0xf << 3 = array_mode: u8;
            & 0x7 << 8 = mit_mode: u8;
            & 0x7 << 11 = tilesplit_size: u8;
            & 0x3 << 15 = bank_w: u8;
            & 0x3 << 18 = bank_h: u8;
            & 0x3 << 21 = num_bank: u8;
            & 0x3 << 24 = mat_aspt: u8;
            & 0x1f << 26 = pipe_config: u8;
        }
        dw[9] = {
            & 0x3fff << 0 = linear_x: u16;
            & 0x3fff << 16 = linear_y: u16;
        }
        dw[10] = {
            & 0x7ff << 0 = linear_z: u16;
            & 0x3fff << 16 = linear_pitch: u16;
        }
        dw[11] = {
            & 0xfffffff << 0 = linear_slice_pitch: u32;
        }
        dw[12] = {
            & 0x3fff << 0 = rect_x: u16;
            & 0x3fff << 16 = rect_y: u16;
        }
        dw[13] = {
            & 0x7ff << 0 = rect_z: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = tile_sw: u8;
        }
        @join
        dw[1], dw[2] = tiled_addr: u64;
        dw[7], dw[8] = linear_addr: u64;
    });

    packet!(CopyStruct {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 31 = detile: bool;
        }
        dw[5] = {
            & 0x7ff << 0 = stride: u16;
            & 0x3 << 16 = linear_sw: u8;
            & 0x3 << 24 = struct_sw: u8;
        }
        @full
        dw[3] = start_index: u32;
        dw[4] = count: u32;
        @join
        dw[1], dw[2] = sb_addr: u64;
        dw[6], dw[7] = linear_addr: u64;
    });

    packet!(WriteUntiled<'a> {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
        }
        dw[3] = {
            & 0x3 << 24 = sw: u8;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        @dyn
        dw[4..] = data0: &'a [u32],
        dw[3] & 0x000fffff << 0 = len
    });

    packet!(WriteTiled<'a> {
        @bits
        dw[0] = {
            & 0x1 << 16 = encrypt: bool;
            & 0x1 << 18 = tmz: bool;
        }
        dw[3] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[4] = {
            & 0x3fff << 0 = height: u16;
            & 0x1fff << 16 = depth: u16;
        }
        dw[5] = {
            & 0x7 << 0 = element_size: u8;
            & 0x1f << 3 = swizzle_mode: u8;
            & 0x3 << 9 = dimension: u8;
            & 0xf << 16 = mip_max: u8;
        }
        dw[6] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0x1fff << 0 = z: u16;
            & 0x3 << 24 = sw: u8;
        }
        dw[8] = {
            & 0xfffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        @dyn
        dw[9..] = data: &'a [u32],
        dw[8] & 0x000fffff << 0 = len
    });

    packet!(WriteTiledBc<'a> {
        @bits
        dw[3] = {
            & 0x3fff << 0 = width: u16;
        }
        dw[4] = {
            & 0x3fff << 0 = height: u16;
            & 0x7ff << 16 = depth: u16;
        }
        dw[5] = {
            & 0x7 << 0 = element_size: u8;
            & 0xf << 3 = array_mode: u8;
            & 0x7 << 8 = mit_mode: u8;
            & 0x7 << 11 = tilesplit_size: u8;
            & 0x3 << 15 = bank_w: u8;
            & 0x3 << 18 = bank_h: u8;
            & 0x3 << 21 = num_bank: u8;
            & 0x3 << 24 = mat_aspt: u8;
            & 0x1f << 26 = pipe_config: u8;
        }
        dw[6] = {
            & 0x3fff << 0 = x: u16;
            & 0x3fff << 16 = y: u16;
        }
        dw[7] = {
            & 0x7ff << 0 = z: u16;
            & 0x3 << 24 = sw: u8;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        @dyn
        dw[9..] = data: &'a [u32],
        dw[8] & 0x000fffff << 2 = len
    });

    packet!(PtepdeCopy {
        @bits
        dw[0] = {
            & 0x1 << 18 = tmz: bool;
            & 0x1 << 31 = ptepde_op: bool;
        }
        dw[7] = {
            & 0x7ffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[3], dw[4] = dst_addr: u64;
        dw[5], dw[6] = mask: u64;
    });

    packet!(PtepdeCopyBackwards {
        @bits
        dw[0] = {
            & 0x3 << 28 = pte_size: u8;
            & 0x1 << 30 = direction: bool;
            & 0x1 << 31 = ptepde_op: bool;
        }
        dw[5] = {
            & 0xff << 0 = mask_first_xfer: u8;
            & 0xff << 8 = mask_last_xfer: u8;
        }
        dw[6] = {
            & 0x1ffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = src_addr: u64;
        dw[3], dw[4] = dst_addr: u64;
    });

    packet!(PtepdeRmw {
        @bits
        dw[0] = {
            & 0x7 << 16 = mtype: u8;
            & 0x1 << 19 = gcc: bool;
            & 0x1 << 20 = sys: bool;
            & 0x1 << 22 = snp: bool;
            & 0x1 << 23 = gpa: bool;
            & 0x3 << 24 = l2_policy: u8;
        }
        @join
        dw[1], dw[2] = addr: u64;
        dw[3], dw[4] = mask: u64;
        dw[5], dw[6] = value: u64;
    });

    packet!(WriteIncr {
        @bits
        dw[9] = {
            & 0x7ffff << 0 = count: u32;
        }
        @join
        dw[1], dw[2] = dst_addr: u64;
        dw[3], dw[4] = mask: u64;
        dw[5], dw[6] = init: u64;
        dw[7], dw[8] = incr: u64;
    });

    packet!(Indirect {
        @bits
        dw[0] = {
            & 0xf << 16 = vmid: u8;
            & 0x1 << 31 = r#priv: bool;
        }
        dw[3] = {
            & 0xfffff << 0 = ib_size: u32;
        }
        @join
        dw[1], dw[2] = ib_base: u64;
        dw[4], dw[5] = csa_addr: u64;
    });

    packet!(Semaphore {
        @bits
        dw[0] = {
            & 0x1 << 29 = write_one: bool;
            & 0x1 << 30 = signal: bool;
            & 0x1 << 31 = mailbox: bool;
        }
        @join
        dw[1], dw[2] = addr: u64;
    });

    packet!(Fence {
        @bits
        dw[0] = {
            & 0x7 << 16 = mtype: u8;
            & 0x1 << 19 = gcc: bool;
            & 0x1 << 20 = sys: bool;
            & 0x1 << 22 = snp: bool;
            & 0x1 << 23 = gpa: bool;
            & 0x3 << 24 = l2_policy: u8;
        }
        @full
        dw[3] = data: u32;
        @join
        dw[1], dw[2] = addr: u64;
    });

    packet!(SrbmWrite {
        @bits
        dw[0] = {
            & 0xf << 28 = byte_en: u8;
        }
        dw[1] = {
            & 0x3ffff << 0 = addr: u32;
            & 0xfff << 20 = apertureid: u16;
        }
        @full
        dw[2] = data: u32;
    });

    packet!(PreExe {
        @bits
        dw[0] = {
            & 0xff << 16 = dev_sel: u8;
        }
        dw[1] = {
            & 0x3fff << 0 = exec_count: u16;
        }
    });

    packet!(CondExe {
        @bits
        dw[4] = {
            & 0x3fff << 0 = exec_count: u16;
        }
        @full
        dw[3] = reference: u32;
        @join
        dw[1], dw[2] = addr: u64;
    });

    packet!(ConstantFill {
        @bits
        dw[0] = {
            & 0x3 << 16 = sw: u8;
            & 0x3 << 30 = fillsize: u8;
        }
        dw[4] = {
            & 0x3fffff << 0 = count: u32;
        }
        @full
        dw[3] = src_data_31_0: u32;
        @join
        dw[1], dw[2] = dst_addr: u64;
    });

    packet!(DataFillMulti {
        @bits
        dw[0] = {
            & 0x1 << 31 = memlog_clr: bool;
        }
        dw[5] = {
            & 0x3ffffff << 0 = count: u32;
        }
        @full
        dw[1] = byte_stride: u32;
        dw[2] = dma_count: u32;
        @join
        dw[3], dw[4] = dst_addr: u64;
    });

    packet!(PollRegmem {
        @bits
        dw[0] = {
            & 0x1 << 26 = hdp_flush: bool;
            & 0x7 << 28 = func: u8;
            & 0x1 << 31 = mem_poll: bool;
        }
        dw[5] = {
            & 0xffff << 0 = interval: u16;
            & 0xfff << 16 = retry_count: u16;
        }
        @full
        dw[3] = value: u32;
        dw[4] = mask: u32;
        @join
        dw[1], dw[2] = addr: u64;
    });

    pub use super::v4_4::PollRegWriteMem;

    pub use super::v4_4::PollDbitWriteMem;

    pub use super::v4_4::PollMemVerify;

    packet!(VmInvalidation {
        @bits
        dw[0] = {
            & 0x1f << 16 = gfx_eng_id: u8;
            & 0x1f << 24 = mm_eng_id: u8;
        }
        dw[3] = {
            & 0xffff << 0 = invalidateack: u16;
            & 0x1f << 16 = addressrangehi: u8;
            & 0x1ff << 23 = reserved: u16;
        }
        @full
        dw[1] = invalidatereq: u32;
        dw[2] = addressrangelo: u32;
    });

    pub use super::v4_4::Atomic;

    packet!(TimestampSet {
        @join
        dw[1], dw[2] = init_data: u64;
    });

    packet!(TimestampGet {
        @bits
        dw[1] = {
            & 0x1fffffff << 3 = write_addr_31_3: u32;
        }
        @full
        dw[2] = write_addr_63_32: u32;
    });

    packet!(TimestampGetGlobal {
        @bits
        dw[1] = {
            & 0x1fffffff << 3 = write_addr_31_3: u32;
        }
        @full
        dw[2] = write_addr_63_32: u32;
    });

    packet!(Trap {
        @bits
        dw[1] = {
            & 0xfffffff << 0 = int_context: u32;
        }
    });

    packet!(DummyTrap {
        @bits
        dw[1] = {
            & 0xfffffff << 0 = int_context: u32;
        }
    });

    packet!(GpuvmInv {
        @bits
        dw[1] = {
            & 0xffff << 0 = per_vmid_inv_req: u16;
            & 0x7 << 16 = flush_type: u8;
            & 0x1 << 19 = l2_ptes: bool;
            & 0x1 << 20 = l2_pde0: bool;
            & 0x1 << 21 = l2_pde1: bool;
            & 0x1 << 22 = l2_pde2: bool;
            & 0x1 << 23 = l1_ptes: bool;
            & 0x1 << 24 = clr_protection_fault_status_addr: bool;
            & 0x1 << 25 = log_request: bool;
            & 0x1 << 26 = four_kilobytes: bool;
        }
        dw[2] = {
            & 0x1 << 0 = s: bool;
            & 0x7fffffff << 1 = page_va_42_12: u32;
        }
        dw[3] = {
            & 0x3f << 0 = page_va_47_43: u8;
        }
    });

    packet!(GcrReq {
        @bits
        dw[1] = {
            & 0x1ffffff << 7 = base_va_31_7: u32;
        }
        dw[2] = {
            & 0xffff << 0 = base_va_47_32: u16;
            & 0xffff << 16 = gcr_control_15_0: u16;
        }
        dw[3] = {
            & 0x7 << 0 = gcr_control_18_16: u8;
            & 0x1ffffff << 7 = limit_va_31_7: u32;
        }
        dw[4] = {
            & 0xffff << 0 = limit_va_47_32: u16;
            & 0xf << 24 = vmid: u8;
        }
    });

    packet!(Nop<'pkt> {
        @dyn
        dw[1..] = data: &'pkt [u32],
        dw[0] & 0x3fff << 16 = len
    });

    unify!(Pkt<'pkt> {
        @match_extra op =1, subop = 0, dw[0] >> 27 & 0x1 => {
            0 => CopyLinear
            1 => CopyBroadcastLinear
        }

        // discriminant not confirmed
        // 27 -> broadcast
        // 26 -> videocopy (frame_to_field in umr)
        //
        // Since it uses the same mem layout in umr irrespective of videocopy I'm
        // going to treat it as if only broadcast is a discriminant
        @match_extra op = 1, subop = 1, dw[0] >> 27 & 0x1 => {
            0 => CopyTiled
            1 => CopyL2tBroadcast
        }
        op = 0 => Nop<'pkt>
        op = 1, subop = 3 => CopyStruct
        op = 1, subop = 4 => CopyLinearSubwin
        op = 1, subop = 5 => CopyTiledSubwin
        op = 1, subop = 6 => CopyT2t
        op = 1, subop = 7 => CopyDirtyPage
        op = 1, subop = 8 => CopyPhysicalLinear
        op = 1, subop = 16 => CopyLinearBc
        op = 1, subop = 17 => CopyTiledBc
        op = 1, subop = 20 => CopyLinearSubwinBc
        op = 1, subop = 21 => CopyTiledSubwinBc
        op = 1, subop = 22 => CopyT2tBc
        op = 2, subop = 0 => WriteUntiled<'pkt>
        op = 2, subop = 1 => WriteTiled<'pkt>
        op = 2, subop = 17 => WriteTiledBc<'pkt>
        op = 4 => Indirect
        op = 5, subop = 0 => Fence
        // Only in UMR
        //op = 5, subop = 1 => FenceConditionalInterrupt
        //op = 5, subop = 3 => FenceProtected
        op = 6 => Trap
        op = 7, subop = 0 => Semaphore
        //op = 7, subop = 1 => WriteIncr
        op = 8, subop = 0 => PollRegmem
        op = 8, subop = 1 => PollRegWriteMem
        op = 8, subop = 2 => PollDbitWriteMem
        op = 8, subop = 3 => PollMemVerify
        op = 8, subop = 4 => VmInvalidation
        op = 9 => CondExe
        op = 10 => Atomic
        op = 11, subop = 0 => ConstantFill
        op = 11, subop = 1 => DataFillMulti
        op = 12, subop = 0 => WriteIncr
        op = 12, subop = 1 => PtepdeCopy
        op = 12, subop = 2 => PtepdeRmw
        op = 12, subop = 3 => PtepdeCopyBackwards
        op = 13, subop = 0 => TimestampSet
        op = 13, subop = 1 => TimestampGet
        op = 13, subop = 2 => TimestampGetGlobal
        op = 14, subop = 0 => SrbmWrite
        // only in UMR
        //op = 14, subop = 1 => RMW_REGISTER
        op = 15 => PreExe
        op = 16 => GpuvmInv
        op = 17 => GcrReq
        op = 32 => DummyTrap
    });
}

/// Rdna 2: Sienna Child, Van Gogh, Navy Flounder, Dimgrey Cavefish, Yellow Carp, Beige Goby
///
/// ## Changes
/// - Added `cpv` (Cache Policy Valid)
pub mod v5_2 {}
/// Rdna 3, 3.5
///
/// ## Changes
pub mod v6_0 {}

/// Rdna 4
///
/// ## Changes
pub mod v7_0 {}
