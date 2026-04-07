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
        #[derive(Default, Clone, Copy)]
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
pub mod v2_4;

/// GCN 3, 4: Tonga, Corrizo, Fiji, Stoney, Polaris 10, Polaris 12, Vegam
///
/// ## Changes
/// - added ATOMIC
pub mod v3_0;

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
pub mod v4_0;

/// GCN 5 (v4.4.2, v4.4.3, v4.4.4)
///
/// ## Changes
/// - COPY_LINEAR: dst and src cache policy
/// - COPY_LINEAR_BROADCAST: dst, dst2, src cache policy
pub mod v4_4;

/// Rdna 1: Navi 14, Navi 12, Navi 10, Cyan Skillfish, Cyan Skillfish 2
///
/// See `kernel/drivers/gpu/drm/amd/amdgpu/navi10_sdma_pkt_open.h`
///
/// ## Changes
pub mod v5_0;

/// Rdna 2: Sienna Child, Van Gogh, Navy Flounder, Dimgrey Cavefish, Yellow Carp, Beige Goby
///
/// ## Changes
/// - Added `cpv` (Cache Policy Valid)
/// - COPY_LINEAR,
/// COPY_LINEAR_SUBWIN:
///     * added `cpv`
///     * added `dst_cache_policy`
///     * added `src_cache_policy`
///     * increased `count` size
/// - Added COPY_LINEAR_SUBWIN_LARGE
/// - COPY_PHYSICAL_LINEAR:
///     * added `cpv`
///     * moved `dst_sw`
///     * added `dst_llc`
///     * added `src_llc`
/// - COPY_STRUCT:
///     * added `cpv`
///     * added `linear_cache_policy`
///     * added `struct_cache_policy`
/// - COPY_T2T:
///     * added `cpv`
///     * added `dst_cache_policy`
///     * added `src_cache_policy`
///     * added `meta_llc`
/// - COPY_TILED:
///     * added `cpv`
///     * removed `linear_cc`
///     * added `linear_cache_policy`
///     * added `tile_cache_policy`
///     * increased `count` size
/// - COPY_TILED_SUBWIN:
///     * added `cpv`
///     * added `linear_cache_policy`
///     * added `tile_cache_policy`
///     * added `meta_llc`
/// - DATA_FILL_MULTI,
/// POLL_DBIT_WRITE_MEM,
/// POLL_MEM_VERIFY,
/// POLL_REGMEM,
/// POLL_REG_WRITE_MEM,
/// WRITE_TILED,
/// WRITE_UNTILED,
/// WRITE_INCR:
///     * added `cpv`
///     * added `cache_policy`
/// - FENCE:
///     * added `cpv`
///     * added `llc_policy`
/// - MEM_INCR,
/// TIMESTAMP_GET,
/// TIMESTAMP_GET_GLOBAL:
///     * added `cpv`
///     * added `llc_policy`
///     * added `l2_policy`
pub mod v5_2;
/// Rdna 3, 3.5
///
/// ## Changes
pub mod v6_0 {}

/// Rdna 4
///
/// ## Changes
pub mod v7_0 {}
