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
///     VariantA<'pkt>: op = 2, subop = 5, extra & 0x1 << 17 = 1 {
///         @bits
///         dw[0] = {
///             /// A bit flag
///             & 0x1 << 16 = bit_flag: bool;
///         }
///         @full
///         /// All bits used
///         dw[4] = free_real_estate: i32;
///         @join
///         #[doc = "A joined u64 address, `addr.lo = dw[2], addr.hi = dw[3]`"]
///         dw[2], dw[3] = addr: u64;
///         @dyn
///         /// A variable length array
///         dw[5 ..] = data: &'pkt [u32],
///         dw[1] & 0xfff << 1 = len
///     },
/// });
/// ```
macro_rules! sdma_packets {
    (
        $enum_name:ident $(<$life:lifetime>)? {
            $(
                $(#[$attr:meta])*
                $variant:ident $(<$vlife:lifetime>)? :
                    op = $op:literal
                    $(, subop = $subop:literal)?
                    $(, extra & $exmask:literal << $exshift:literal = $ex:literal)?
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
            ),* $(,)?
        }
    ) => {
        $(
        $(#[$attr])*
        #[derive(Clone, Copy, Default)]
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
            $(pub $dyn_field: & $dyn_life [u32])?
        }

        impl $(<$vlife>)? $variant $(<$vlife>)? {
            pub const STATIC_DWORDS: usize = 1 + sdma_packets!(@max_dw 0usize $($(, $dw)*)? $($(, $fdw)*)? $($(, $lo_dw, $hi_dw)*)? $(, $dyn_len)?);

            pub const fn encode_linear(&self, buff: &mut [u32]) -> usize {
                let actual_dwords: usize = Self::STATIC_DWORDS $(+ self.$dyn_field.len())?;
                if buff.len() < actual_dwords { panic!("Buffer too small to copy data to") }

                const _: () = assert!($op <= 0xff, "OP is limited to 0xFF");
                $(const _: () = assert!($subop <= 0xff, "SUBOP is limited to 0xFF");)?
                $(const _: () = assert!($ex <= $exmask, concat!("extra is limited to ", stringify!($exmask)));)?
                $(const _: () = assert!($exshift >= 16, concat!("extra should not use OP or SUBOP bits", stringify!($exmask)));)?

                let header = $op $(| $subop << 8)? $(| ($ex & $exmask) << $exshift)?;
                let mask = 0xffff $(| $exmask << $exshift)?;
                buff[0] &= !mask;
                buff[0] |= header;


                $($(
                    let mut mask: u32 = 0;
                    $(mask |= $mask << $shift;)*

        //             $(
        //             const _: () = assert!($mask >= <$ftype as super::FieldDecode>::MASK,
        // concat!("Provided type: ", stringify!($ftype), " doesn't fit mask: ", stringify!($mask)));
        //             )*

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

        pub enum $enum_name $(<$life>)? {
        $($variant ($variant $(<$vlife>)?),)*
        }

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
                    sdma_packets!(@op_match $op $($subop)? $(header [$exmask << $exshift] == $ex)?) => {
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
    (@op_match $op:literal $subop:literal $hd:ident[$msk:literal << $shf:literal] == $ex:literal) => {($op, $subop) if $ex == ($hd >> $shf ) & $msk };
    (@op_match $op:literal $subop:literal) => {($op, $subop)};
    (@op_match $op:literal) => {($op, _)};
}

/// GCN 3: Topaz
pub mod v2_4 {}

/// GCN 3, 4: Tonga, Corrizo, Fiji, Stoney, Polaris 10, Polaris 12, Vegam
pub mod v3_0 {}

/// GCN 5 (v4.0 - v4.4.0): Vega 10, Raven, Vega 12, Raven 2, Picasso, Renoir, Vega 20, Arcturus, Aldebaran
pub mod v4_0 {}

/// GCN 5 (v4.4.2, v4.4.3, v4.4.4)
pub mod v4_4 {}

/// Rdna 1: Navi 14, Navi 12, Navi 10, Cyan Skillfish, Cyan Skillfish 2
pub mod v5_0 {}

/// Rdna 2: Sienna Child, Van Gogh, Navy Flounder, Dimgrey Cavefish, Yellow Carp, Beige Goby
pub mod v5_2 {

    field_enum![
        Mtype: 0x7 {
            #[default]
            C_RW_US = 0x00000000,
            RESERVED_1 = 0x00000001,
            C_RO_S = 0x00000002,
            Uncached = 3,
            C_RW_S = 0x00000004,
            RESERVED_5 = 0x00000005,
            C_RO_US = 0x00000006,
            RESERVED_7 = 0x00000007,
        }
    ];

    field_enum!(
        L2Policy: 0x3 {
            #[default]
            LRU = 0x00000000,
            STREAM = 0x00000001,
            NOA = 0x00000002,
            BYPASS = 0x00000003,
        }
    );

    sdma_packets!(Pkt<'pkt>{
        VariantA<'pkt>: op = 2, subop = 5, extra & 0x1 << 17 = 1 {
            @bits
            dw[0] = {
                /// A bit flag
                & 0x1 << 16 = bit_flag: bool;
            }
            @full
            /// All bits used
            dw[4] = free_real_estate: i32;
            @join
            /// A joined u64 address, `addr.lo = dw[2], addr.hi = dw[3]`
            dw[2], dw[3] = addr: u64;
            @dyn
            /// A variable length array
            dw[5 ..] = data: &'pkt [u32],
            dw[1] & 0xfff << 1 = len
        },
        VariantB: op = 2 {}
    });
}
/// Rdna 3, 3.5
pub mod v6_0 {}

/// Rdna 4
pub mod v7_0 {}
