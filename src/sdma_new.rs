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

/// Helpfull macro to define sdma packet enum and respective structs for each packet type.
///
/// The generated types can define a generic lifetyme necessary to have variable packet length.
///
/// Packets are encoded as dwords.
///
/// Packets are defined with a header which distinguishes the packet by op, subop and extra bits in
/// first dword.
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
/// # Example
/// ```
/// sdma_packets!(Pkt<'pkt>{
///     @match_extra mod_name:family op = 2, subop = 5, dw[0] >> 17 & 0x1 => {
///         0 => VariantAFalse {}
///         1 => VariantATrue<'pkt> {
///             @bits
///             dw[0] = {
///                 /// A bit flag
///                 & 0x1 << 16 = bit_flag: bool;
///             }
///             @full
///             /// All bits used
///             dw[4] = free_real_estate: i32;
///             @join
///             /// A joined u64 address, `addr.lo = dw[2], addr.hi = dw[3]`
///             dw[2], dw[3] = addr: u64;
///             @dyn
///             /// A variable length array
///             dw[5 ..] = data: &'pkt [u32],
///             dw[1] & 0xfff << 1 = len
///         }
///     }
///     @match_extra mod_name:enemy op = 2, subop = 6, dw[0] >> 17 & 0x1 => {
///         1 => VariantBTrue<'pkt> {
///             @bits
///             dw[0] = {
///                 /// A bit flag
///                 & 0x1 << 16 = bit_flag: bool;
///             }
///             @full
///             /// All bits used
///             dw[4] = free_real_estate: i32;
///             @join
///             /// A joined u64 address, `addr.lo = dw[2], addr.hi = dw[3]`
///             dw[2], dw[3] = addr: u64;
///             @dyn
///             /// A variable length array
///             dw[5 ..] = data: &'pkt [u32],
///             dw[1] & 0xfff << 1 = len
///         }
///     }
///     /// VariantC docs
///     op = 2 => VariantC {}
/// });
/// ```
macro_rules! sdma_packets {
    (
        $enum_name:ident $(<$life:lifetime>)? {
    $(@match_extra mod_name:$ex_scope:ident  op = $ex_op:literal $(, subop = $ex_subop:literal)?
    , dw[0] >> $exshift:literal & $exmask:literal => { $(
        $(#[$ex_attr:meta])*
        $ex:literal => $ex_variant:ident $(<$ex_vlife:lifetime>)?
                {
                    $(@bits
                        $(
                            dw[$ex_bits_dword:literal] = {
                                $(
                                    $(#[$ex_bits_attr:meta])*
                                    & $ex_bits_mask:literal << $ex_bits_shift:literal
    = $ex_bits_ident:ident : $ex_bits_type:ty ;
                                )+
                            }
                        )+
                    )?
                    $(@full
                        $(
                            $(#[$ex_full_attr:meta])*
                            dw[$ex_full_dword:literal] = $ex_full_ident:ident: $ex_full_type:ty ;
                        )+
                    )?
                    $(@join
                        $(
                            $(#[$ex_join_attr:meta])*
                            dw[$ex_join_lo:literal], dw[$ex_join_hi:literal] = $ex_join_ident:ident: $ex_join_type:ty ;
                        )+
                    )?
                    $(@dyn
                        $(#[$ex_dyn_attr:meta])*
                        dw[$ex_dyn_dword:literal ..] = $ex_dyn_ident:ident : & $ex_dyn_lifetime:lifetime [u32],
                        dw[$ex_dyn_len:literal] $(& $ex_dyn_mask:literal << $ex_dyn_shift:literal)? = len
                    )?
                }

    )+ } )*
    $(
    $(#[$attr:meta])*
    op = $op:literal $(, subop = $subop:literal)? =>
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
    )*

        }
    ) => {
        pub enum $enum_name $(<$life>)? {
        $($($ex_variant($ex_scope::$ex_variant $(<$ex_vlife>)?),)*)*
        $($variant($variant $(<$vlife>)?),)*
        }



        $(
        pub mod $ex_scope {
        pub const OP: u8 = $ex_op;
        pub const SUB_OP: u8 = 0 $(| $ex_subop)?;
        pub const HEADER_MASK: u32 = super::super::HEADER_MASK | $exmask << $exshift;


        const _: () = assert!(matches!(u32::overflowing_shl($exmask, $exshift), (mask, false) if mask & $crate::sdma_new::HEADER_MASK == 0) , concat!("extra bits are limited to upper 2 bytes of header: ", stringify!($exmask), " << ", stringify!($exshift)));

        $(
        $(#[$ex_attr])*
        #[derive(Clone, Copy, Default)]
        pub struct $ex_variant $(<$ex_vlife>)? {
        $($($(
            $(#[$ex_bits_attr])*
            pub $ex_bits_ident: $ex_bits_type,
        )*)*)?
        $($(
            $(#[$ex_full_attr])*
            pub $ex_full_ident: $ex_full_type,
        )*)?
        $($(
            $(#[$ex_join_attr])*
            pub $ex_join_ident: $ex_join_type,
        )*)?
        $(
            $(#[$ex_dyn_attr])*
            pub $ex_dyn_ident: & $ex_dyn_lifetime [u32]
        )?
        }

        impl $(<$ex_vlife>)? $ex_variant $(<$ex_vlife>)? {
            pub const STATIC_DWORDS: usize = 1 + sdma_packets!(@max_dw 0usize $($(, $ex_bits_dword)*)?
        $($(, $ex_full_dword)*)? $($(, $ex_join_lo, $ex_join_hi)*)? $(, $ex_dyn_len)?);
            pub const HEADER: u32 = u32::from(OP) | u32::from(SUB_OP) << 8 | ($ex & $exmask) << $exshift;

            pub const fn encode_linear(&self, buff: &mut [u32]) -> usize {
                let actual_dwords: usize = Self::STATIC_DWORDS $(+ self.$ex_dyn_ident.len())?;
                if buff.len() < actual_dwords { panic!("Buffer too small to copy data to") }

                buff[0] &= !HEADER_MASK;
                buff[0] |= Self::HEADER;

                $($(
                let mut mask: u32 = 0;
                $(mask |= $ex_bits_mask << $ex_bits_shift;)*

                let mut value: u32 = 0;
                $(value |= (self.$ex_bits_ident as u32) << $ex_bits_shift;)*

                buff[$ex_bits_dword] &= !mask;
                buff[$ex_bits_dword] |= value;
                )*)?

                $($(buff[$ex_full_dword] = self.$ex_full_ident as u32;)*)?

                $($(
                buff[$ex_join_lo] = self.$ex_join_ident as u32;
                buff[$ex_join_hi] = (self.$ex_join_ident as u64 >> 32) as u32;
                )*)?

                $(
                const _: () = {assert!($ex_dyn_dword == $ex_variant::STATIC_DWORDS, "Dynamic part must start at the end");};

                let extra_length = self.$ex_dyn_ident.len();
                let len_u32 = match u32::try_from(extra_length) {
                    Ok(x) => x,
                    Err(_) => panic!("Too many extra dwords, u32 limit")
                };

                $(
                if len_u32 > $ex_dyn_mask {
                    panic!(concat!("Too many extra dwords, max: ",stringify!($ex_len_mask)))
                }

                let mask: u32 = $ex_dyn_mask << $ex_dyn_shift;
                buff[$ex_dyn_len] &= !mask;
                )?

                buff[$ex_dyn_len] |= len_u32 $(<< $ex_dyn_shift)?;
                buff[$ex_dyn_dword..($ex_dyn_dword + extra_length)].copy_from_slice(self.$ex_dyn_ident);
                )?
                actual_dwords
            }
        }
)*
        }
        )*

        $(

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
            pub const STATIC_DWORDS: usize = 1 + sdma_packets!(@max_dw 0usize $($(, $dw)*)? $($(, $fdw)*)?
        $($(, $lo_dw, $hi_dw)*)? $(, $dyn_len)?);
            pub const OP: u8 = $op;
            pub const SUB_OP: u8 = 0 $(| $subop)?;
            pub const HEADER: u32 = u32::from(Self::OP) | u32::from(Self::SUB_OP) << 8;

            pub const fn encode_linear(&self, buff: &mut [u32]) -> usize {
                let actual_dwords: usize = Self::STATIC_DWORDS $(+ self.$dyn_field.len())?;
                if buff.len() < actual_dwords { panic!("Buffer too small to copy data to") }

                buff[0] &= !super::HEADER_MASK;
                buff[0] |= Self::HEADER;

                $($(
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
        }
        )*


        impl $(<$life>)? $enum_name $(<$life>)? {
            pub const fn decode_linear(buff: & $($life)? [u32]) -> Option<(usize,$enum_name $(<$life>)?)> {
                if buff.is_empty() {
                    panic!("Not even 1dw to read");
                }
                let header = buff[0];
                let op = header as u8;
                let subop = (header >> 8) as u8;
                let res = match (op, subop) {
        $(
        sdma_packets!(@op_match $ex_op $($ex_subop)?) => {
        let extra = header >> $exshift & $exmask;
        match extra {
        $(
        $ex => {
                        let actual_size = $ex_scope::$ex_variant::STATIC_DWORDS $(+ {
                            if buff.len() < ($ex_dyn_len + 1) {
                                panic!(concat!("Not enough dwords for ", stringify!($ex_scope::$ex_variant)));
                            }
                            let len = buff[$ex_dyn_len];
                            $(let len = (len >> $ex_dyn_shift) & $ex_dyn_mask;)?
                            len as usize
                        })?;

                        if buff.len() < actual_size {
                            panic!(concat!("Not enough dwords for ", stringify!($ex_scope::$ex_variant)));
                        }

                        $($(
                        let dw = buff[$ex_bits_dword];
                        $(let Some($ex_bits_ident) = <$ex_bits_type as super::FieldDecode>::decode((dw >> $ex_bits_shift) & $ex_bits_mask) else {panic!("Field")};)*
                        )*)?

                        $($(
                        let Some($ex_full_ident) = <$ex_full_type as super::FieldDecode>::decode(buff[$ex_full_dword]) else {panic!("Field")};
                        )*)?

                        $($(
                        let $ex_join_ident = <$ex_join_type>::from((u64::from(buff[$ex_join_hi]) << 32) | u64::from(buff[$ex_join_lo]));
                        )*)?

                        $(
                        let dyn_len = buff[$ex_dyn_len] as usize;
                        let $ex_dyn_ident = &buff[$ex_dyn_dword..($ex_dyn_dword + dyn_len)];
                        )?

                        let pkt = $ex_scope::$ex_variant {
                        $($($($ex_bits_ident),*,)*)?
                        $($($ex_full_ident),*,)?
                        $($($ex_join_ident),*,)?
                        $($ex_dyn_ident)?
                        };
                        (actual_size, $enum_name::$ex_variant(pkt))

        },
        )*
        _ => panic!("Unhandled extra driscriminant"),
        }
        }
        )*

                    $(
                    sdma_packets!(@op_match $op $($subop)?)
                     => {
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
                        (actual_size, $enum_name::$variant(pkt))
                    },
                    )*
                    (_, _) => panic!("Unused combination")
                };
                Some(res)
            }
        }
    };
    (@max_dw $cur:expr) => { $cur };
    (@max_dw $cur:expr, $next:literal $($rest:tt)*) => {
        sdma_packets!(@max_dw {
            let c = $cur;
            let n = $next;
            let gt = (n > c) as usize;
            c * (1usize - gt) + n * gt
        } $($rest)*)
    };
    (@op_match $op:literal $subop:literal) => {($op, $subop)};
    (@op_match $op:literal) => {($op, _)};
}

/// GCN 3: Topaz
pub mod v2_4 {
    sdma_packets!(Pkt<'pkt> {

        @match_extra mod_name:copy_linear op =1, subop = 0, dw[0] >> 27 & 0x1 => {
            0 => CopyLinear {
                @bits
                dw[0] = {
                    & 0x1 << 25 = backwards: bool;
                    & 0x1 << 27 = broadcast: bool;
                }
                dw[2] = {
                    & 0x3 << 16 = dst_sw: u8;
                    & 0x1 << 22 = dst_ha: bool;
                    & 0x3 << 24 = src_sw: u8;
                    & 0x1 << 30 = src_ha: bool;
                }
                @full
                dw[1] = copy_count: u32;
                @join
                dw[3], dw[4] = src_addr: u64;
                dw[5], dw[6] = dst_addr: u64;
            }
            1 => CopyLinearBroadcast {
                @bits
                dw[0] = {
                    & 0x1 << 16 = encrypt: bool;
                    & 0x1 << 18 = tmz: bool;
                    & 0x1 << 27 = broadcast: bool;
                }
                dw[2] = {
                    & 0x3 << 8  = dst2_sw: u8;
                    & 0x3 << 16 = dst_sw: u8;
                    & 0x3 << 24 = src_sw: u8;
                }
                @full
                dw[1] = copy_count: u32;
                @join
                dw[3], dw[4] = src_addr: u64;
                dw[5], dw[6] = dst_addr: u64;
                dw[7], dw[8] = dst2_addr: u64;
            }
        }

        /// NOP (variable length)
        op = 0 => Nop<'pkt> {
            @dyn
            dw[1..] = data: &'pkt [u32],
            dw[0] & 0xffff << 16 = len
        }

        // ── op 1: COPY ───────────────────────────────────────────────────────


        /// COPY TILED (non-broadcast, non-l2t)
        op = 1, subop = 1 => CopyTiled {
            @bits
            dw[0] = {
                & 0x1 << 16 = encrypt: bool;
                & 0x1 << 18 = tmz: bool;
                & 0x1 << 26 = videocopy: bool;
                & 0x1 << 27 = broadcast: bool;
                & 0x1 << 31 = detile: bool;
            }
            dw[3] = {
                & 0x7ff << 0  = pitch_in_tile: u16;
                & 0x3fff << 16 = height: u16;
            }
            dw[4] = {
                & 0x3fffff << 0 = slice_pitch: u32;
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
                & 0x1fff << 0 = z: u16;
            }
            dw[8] = {
                & 0x3 << 16 = linear_sw: u8;
                & 0x3 << 24 = tile_sw: u8;
            }
            dw[11] = {
                & 0x7ffff << 0 = linear_pitch: u32;
            }
            dw[12] = {
                & 0xfffff << 0 = count: u32;
            }
            @join
            dw[1], dw[2] = tiled_addr: u64;
            dw[9], dw[10] = linear_addr: u64;
        }

        /// COPY STRUCT (SOA)
        op = 1, subop = 3 => CopyStruct {
            @bits
            dw[0] = {
                & 0x1 << 18 = tmz: bool;
                & 0x1 << 31 = detile: bool;
            }
            dw[5] = {
                & 0x7ff << 0  = stride: u16;
                & 0x3  << 16  = linear_sw: u8;
                & 0x3  << 24  = struct_sw: u8;
            }
            @full
            dw[3] = start_index: u32;
            dw[4] = count: u32;
            @join
            dw[1], dw[2] = sb_addr: u64;
            dw[6], dw[7] = linear_addr: u64;
        }

        op = 1, subop = 4 => CopyLinearSubWindow {
            @bits
            dw[0] = {
                & 0x1 << 18 = tmz: bool;
                & 0x7 << 29 = elementsize: u8;
            }
            dw[3] = {
                & 0x3fff << 0  = src_x: u16;
                & 0x3fff << 16 = src_y: u16;
            }
            dw[4] = {
                // z_mask is version-dependent; use 0x1fff (NV value) for representability
                & 0x1fff << 0 = src_z: u16;
            }
            dw[8] = {
                & 0x3fff << 0  = dst_x: u16;
                & 0x3fff << 16 = dst_y: u16;
            }
            dw[9] = {
                & 0x1fff << 0 = dst_z: u16;
            }
            dw[11] = {
                & 0x3fff << 0  = rect_x: u16;
                & 0x3fff << 16 = rect_y: u16;
            }
            dw[12] = {
                & 0x1fff << 0  = rect_z: u16;
                & 0x3   << 16  = dst_sw: u8;
                & 0x1   << 22  = dst_ha: bool;
                & 0x3   << 24  = src_sw: u8;
                & 0x1   << 30  = src_ha: bool;
            }
            @full
            dw[5] = src_slice_pitch: u32;
            dw[10] = dst_slice_pitch: u32;
            @join
            dw[1], dw[2] = src_addr: u64;
            dw[6], dw[7] = dst_addr: u64;
        }

        op = 1, subop = 5 => CopyTiledSubWindow {
            @bits
            dw[0] = {
                & 0x1 << 18 = tmz: bool;
                & 0x1 << 31 = detile: bool;
            }
            dw[3] = {
                & 0x3fff << 0  = tiled_x: u16;
                & 0x3fff << 16 = tiled_y: u16;
            }
            dw[4] = {
                & 0x1fff << 0  = tiled_z: u16;
                & 0x3fff << 16 = tiled_pitch: u16;
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
                & 0x1fff << 0  = linear_z: u16;
                & 0x3fff << 16 = linear_pitch: u16;
            }
            dw[12] = {
                & 0x3fff << 0  = rect_x: u16;
                & 0x3fff << 16 = rect_y: u16;
            }
            dw[13] = {
                & 0x1fff << 0  = rect_z: u16;
                & 0x3   << 16  = linear_sw: u8;
                & 0x3   << 22  = tile_sw: u8;
            }
            @full
            dw[5] = pitch_in_tile: u32;
            dw[11] = linear_slice_pitch: u32;
            @join
            dw[1], dw[2] = tiled_addr: u64;
            dw[7], dw[8] = linear_addr: u64;
        }
        op = 1, subop = 6 => CopyT2tSubWindow {
            @bits
            dw[0] = {
                & 0x1 << 18 = tmz: bool;
            }
            dw[3] = {
                & 0x3fff << 0  = src_x: u16;
                & 0x3fff << 16 = src_y: u16;
            }
            dw[4] = {
                & 0x1fff << 0  = src_z: u16;
                & 0x3fff << 16 = src_pitch: u16;
            }
            dw[6] = {
                & 0x7  << 0  = src_element_size: u8;
                & 0xf  << 3  = src_array_mode: u8;
                & 0x7  << 8  = src_mit_mode: u8;
                & 0x7  << 11 = src_tilesplit_size: u8;
                & 0x3  << 15 = src_bank_w: u8;
                & 0x3  << 18 = src_bank_h: u8;
                & 0x3  << 21 = src_num_banks: u8;
                & 0x3  << 24 = src_mat_aspt: u8;
                & 0x1f << 26 = src_pipe_config: u8;
            }
            dw[9] = {
                & 0x3fff << 0  = dst_x: u16;
                & 0x3fff << 16 = dst_y: u16;
            }
            dw[10] = {
                & 0x1fff << 0  = dst_z: u16;
                & 0x3fff << 16 = dst_pitch: u16;
            }
            dw[12] = {
                & 0x7  << 0  = dst_element_size: u8;
                & 0xf  << 3  = dst_array_mode: u8;
                & 0x7  << 8  = dst_mit_mode: u8;
                & 0x7  << 11 = dst_tilesplit_size: u8;
                & 0x3  << 15 = dst_bank_w: u8;
                & 0x3  << 18 = dst_bank_h: u8;
                & 0x3  << 21 = dst_num_banks: u8;
                & 0x3  << 24 = dst_mat_aspt: u8;
                & 0x1f << 26 = dst_pipe_config: u8;
            }
            dw[13] = {
                & 0x3fff << 0  = rect_x: u16;
                & 0x3fff << 16 = rect_y: u16;
            }
            dw[14] = {
                & 0x1fff << 0  = rect_z: u16;
                & 0x3   << 16  = dst_sw: u8;
                & 0x3   << 22  = src_sw: u8;
            }
            @full
            dw[5] = src_slice_pitch: u32;
            dw[11] = dst_slice_pitch: u32;
            @join
            dw[1], dw[2] = src_addr: u64;
            dw[7], dw[8] = dst_addr: u64;
        }
        op = 1, subop = 7 => CopyDirtyPage {
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
        }
        // ── op 2: WRITE ──────────────────────────────────────────────────────
        /// WRITE LINEAR (with variable data payload)
        op = 2, subop = 0 => WriteLinear<'pkt> {
            @bits
            dw[0] = {
                & 0x1 << 16 = encrypt: bool;
                & 0x1 << 18 = tmz: bool;
            }
            dw[3] = {
                & 0x3      << 24 = swap: u8;
            }
            @join
            dw[1], dw[2] = dst_addr: u64;
            @dyn
            dw[4..] = data: &'pkt [u32],
            dw[3] & 0xffffff << 0 = len
        }

        /// WRITE TILED (with variable data payload)
        op = 2, subop = 1 => WriteTiled<'pkt> {
            @bits
            dw[3] = {
                & 0x7ff  << 0  = pitch_in_tile: u16;
                & 0x3fff << 16 = height: u16;
            }
            dw[4] = {
                & 0x3fffff << 0 = slice_pitch: u32;
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
                & 0x7ff  << 0  = z: u16;
                & 0x3    << 24 = sw: u8;
            }
            @join
            dw[1], dw[2] = dst_addr: u64;
            @dyn
            dw[9..] = data: &'pkt [u32],
            dw[8] & 0xfffff << 0 = len
        }
        op = 4 => IndirectBuffer {
            @bits
            dw[0] = {
                & 0xf << 16 = vmid: u8;
                & 0x1 << 31 = priv_flag: bool;
            }
            @full
            dw[3] = ib_size: u32;
            @join
            dw[1], dw[2] = ib_base: u64;
            dw[4], dw[5] = ib_csa_addr: u64;
        }
        op = 5 => Fence {
            @full
            dw[3] = fence_data: u32;
            @join
            dw[1], dw[2] = fence_addr: u64;
        }
        op = 6 => Trap {
            @bits
            dw[1] = {
                & 0xffffff << 0 = trap_int_context: u32;
            }
        }
        op=7, subop=0 => Sem {
            @bits
            dw[0] = {
                & 0x1 << 29 = write_one: bool;
                & 0x1 << 30 = signal: bool;
                & 0x1 << 31 = mailbox: bool;
            }
            @join
            dw[1], dw[2] = semaphore_addr: u64;
        }

        op = 7, subop = 1 => SemMemIncr {
            @join
            dw[1], dw[2] = addr: u64;
        }
        op = 8, subop = 0 => PollRegmem {
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
        }

        op = 8, subop = 1 => PollRegWriteMem {
            @full
            dw[1] = src_addr: u32;
            @join
            dw[2], dw[3] = dst_addr: u64;
        }

        op = 8, subop = 2 => PollDbitWriteMem {
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
        }

        op = 8, subop = 3 => MemVerify {
            @bits
            dw[0] = {
                & 0x1 << 31 = mode: bool;
            }
            @full
            dw[1] = pattern: u32;
            dw[12] = reserved: u32;
            @join
            dw[2], dw[3] = cmp0_addr_start: u64;
            dw[4], dw[5] = cmp0_addr_end: u64;
            dw[6], dw[7] = cmp1_addr_start: u64;
            dw[8], dw[9] = cmp1_addr_end: u64;
            dw[10], dw[11] = rec_addr: u64;
        }

        op = 9 => CondExe {
            @full
            dw[3] = reference: u32;
            dw[4] = exec_count: u32;
            @join
            dw[1], dw[2] = addr: u64;
        }
        op = 10 => Atomic {
            @bits
            dw[0] = {
                & 0x1  << 16 = loop_flag: bool;
                & 0x7f << 25 = op_code: u8;
            }
            dw[7] = {
                & 0x1fff << 0 = loop_interval: u16;
            }
            @full
            dw[3] = src_data_lo: u32;
            dw[4] = src_data_hi: u32;
            dw[5] = cmp_data_lo: u32;
            dw[6] = cmp_data_hi: u32;
            @join
            dw[1], dw[2] = addr: u64;
        }
        op = 11, subop = 0 => ConstFill {
            @bits
            dw[0] = {
                & 0x3 << 16 = swap: u8;
                & 0x3 << 30 = fill_size: u8;
            }
            @full
            dw[3] = data: u32;
            dw[4] = byte_count: u32;
            @join
            dw[1], dw[2] = dst_addr: u64;
        }

        op = 11, subop = 1 => DataFillMulti {
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
        }
        op = 12, subop = 0 => GenPtepde {
            @bits
            dw[9] = {
                & 0x7ffff << 0 = count: u32;
            }
            @join
            dw[1], dw[2] = dst_addr: u64;
            dw[3], dw[4] = mask: u64;
            dw[5], dw[6] = init: u64;
            dw[7], dw[8] = incr: u64;
        }

        op = 12, subop = 1 => GenPtepdeCopy {
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
        }

        op = 12, subop = 2 => GenPtepdeRmw {
            @bits
            dw[0] = {
                & 0x7 << 16 = mtype: u8;
                & 0x1 << 19 = gcc: bool;
                & 0x1 << 20 = sys: bool;
                & 0x1 << 22 = snp: bool;
                & 0x1 << 23 = gpa: bool;
            }
            @full
            dw[7] = num_of_pte: u32;
            @join
            dw[1], dw[2] = addr: u64;
            dw[3], dw[4] = mask: u64;
            dw[5], dw[6] = value: u64;
        }
        op = 13, subop = 0 => TimestampSet {
            @join
            dw[1], dw[2] = init_data: u64;
        }

        op = 13, subop = 1 => TimestampGet {
            @join
            dw[1], dw[2] = write_addr: u64;
        }

        op = 13, subop = 2 => TimestampGetGlobal {
            @join
            dw[1], dw[2] = write_addr: u64;
        }
        op = 14, subop = 0 => SrbmWrite {
            @bits
            dw[0] = {
                & 0xf << 28 = byte_enable: u8;
            }
            dw[1] = {
                & 0xffff << 0 = addr: u16;
            }
            @full
            dw[2] = data: u32;
        }

        op = 15 => PreExe {
            @bits
            dw[0] = {
                & 0xff << 16 = dev_sel: u8;
            }
            dw[1] = {
                & 0x3fff << 0 = exec_count: u16;
            }
        }

        op = 16 => GpuvmInv {
            @bits
            dw[1] = {
                & 0xffff << 0  = per_vmid_inv_req: u16;
                & 0x7    << 16 = flush_type: u8;
                & 0x1    << 19 = inv_l2_ptes: bool;
                & 0x1    << 20 = inv_l2_pde0: bool;
                & 0x1    << 21 = inv_l2_pde1: bool;
                & 0x1    << 22 = inv_l2_pde2: bool;
                & 0x1    << 23 = inv_l1_ptes: bool;
                & 0x1    << 24 = clear_protection_fault_status_addr: bool;
                & 0x1    << 25 = log_request: bool;
                & 0x1    << 26 = four_kb: bool;
            }
            dw[2] = {
                & 0x1  << 0 = s_bit: bool;
            }
            dw[3] = {
                & 0x3f << 0 = page_vm_addr_hi: u8;
            }
        }

        op = 17 => Gcr {
            @bits
            dw[3] = {
                & 0x3 << 0 = seq: u8;
                & 0x1 << 2 = range_is_pa: bool;
            }
            @join
            dw[1], dw[2] = base_va: u64;
            dw[3], dw[4] = limit_va: u64;
        }

        op = 32 => DummyTrap {
            @bits
            dw[1] = {
                & 0xfffffff << 0 = int_context: u32;
            }
        }
    });
}

/// GCN 3, 4: Tonga, Corrizo, Fiji, Stoney, Polaris 10, Polaris 12, Vegam
pub mod v3_0 {
    // Packet layout is identical to v2_4.
    pub use super::v2_4::*;
}

/// GCN 5 (v4.0 - v4.4.0): Vega 10, Raven, Vega 12, Raven 2, Picasso, Renoir, Vega 20, Arcturus, Aldebaran
///
/// ## Changes
/// - COPY LINEAR: adds ENCRYPT, TMZ; drops DST_HA/SRC_HA in favour of
///    DST_SW/SRC_SW (kept), optional DST_CACHE_POLICY/SRC_CACHE_POLICY
/// - COPY TILED: replaces ARRAY_MODE+MIT_MODE+… with SWIZZLE_MODE/DIMENSION/
///    EPITCH; adds MIP_MAX; changes Z mask to 0x1FFF and linear pitch to
///    0x7FFFF; count 0x3FFFFFFF; adds L2_POLICY / LLC_POLICY cache fields
/// - COPY TILED_SUB_WINDOW: SWIZZLE_MODE/DIMENSION/EPITCH; MIP_MAX/MIP_ID
/// - WRITE LINEAR/TILED: TMZ/ENCRYPT; SWIZZLE_MODE instead of tiling params
/// - FENCE: now a sub-opcode 0 packet with MTYPE/GCC/SYS/SNP/GPA fields
/// - FENCE COND INTR: subop 1
/// - SEM MEM_INCR: L2_POLICY/LLC_POLICY
/// - SRBM_WRITE: addr 0x3FFFF-wide
pub mod v4_0 {}

/// GCN 5 (v4.4.2, v4.4.3, v4.4.4)
pub mod v4_4 {}

/// Rdna 1: Navi 14, Navi 12, Navi 10, Cyan Skillfish, Cyan Skillfish 2
pub mod v5_0 {}

/// Rdna 2: Sienna Child, Van Gogh, Navy Flounder, Dimgrey Cavefish, Yellow Carp, Beige Goby
pub mod v5_2 {}
/// Rdna 3, 3.5
pub mod v6_0 {}

/// Rdna 4
pub mod v7_0 {}
