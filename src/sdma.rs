pub type Word = u32;

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

        impl const crate::sdma::FieldDecode for $name {
            fn decode(val: u32) -> Option<Self> {
                match val {
                    $($value => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}
macro_rules! sdma_packet {
    (
        $(#[$attr:meta])*
        $name:ident : op = $op:literal $(, subop = $subop:literal)? {
            $(
                $(#[$fattr:meta])*
                $field:ident : $ftype:ty = dw[$dw:literal] $(@ shift $shift:literal, mask $mask:literal)?
            ),* $(,)?
            $(;
                $(
                    $combined_field:ident : $combined_type:ty = {
                        lo: dw[$lo_dw:literal],
                        hi: dw[$hi_dw:literal]
                    }
                ),* $(,)?
            )?
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone, Copy, Default)]
        pub struct $name {
            $(
                $(#[$fattr])*
                pub $field: $ftype,
            )*
            $($(
                pub $combined_field: $combined_type,
            )*)?
        }

        impl $name {
            pub const fn encode(self) -> [u32; {
                sdma_packet!(@max_dw 0u32
                    $(, $dw)*
                    $($(, $lo_dw, $hi_dw)*)?
                ) as usize + 1
            }] {
                let mut dwords = [0u32; {
                    sdma_packet!(@max_dw 0u32
                        $(, $dw)*
                        $($(, $lo_dw, $hi_dw)*)?
                    ) as usize + 1
                }];

                dwords[0] |= $op & 0xFF;
                $(dwords[0] |= ($subop & 0xFF) << 8;)?

                $(
                    sdma_packet!(@encode_field dwords, self.$field, $ftype, $dw $(, $shift, $mask)?);
                )*

                $($(
                    dwords[$lo_dw] = self.$combined_field as u32;
                    dwords[$hi_dw] = (self.$combined_field >> 32) as u32;
                )*)?

                dwords
            }

            const fn decode(dwords: &[u32]) -> Option<Self> {
                let min_len = sdma_packet!(@max_dw 0u32
                    $(, $dw)*
                    $($(, $lo_dw, $hi_dw)*)?
                ) as usize + 1;
                if dwords.len() < min_len {
                    return None;
                }

                Some(Self {
                    $(
                        $field: sdma_packet!(@decode_field $ftype, dwords, $dw $(, $shift, $mask)?),
                    )*
                    $($(
                        $combined_field: (dwords[$lo_dw] as $combined_type)
                            | ((dwords[$hi_dw] as $combined_type) << 32),
                    )*)?
                })
            }
        }
    };

    (@encode_field $dwords:ident, $self_field:expr, $ftype:ty, $dw:literal, $shift:literal, $mask:literal) => {
        $dwords[$dw] |= ($self_field as u32 & $mask) << $shift;
    };
    (@encode_field $dwords:ident, $self_field:expr, $ftype:ty, $dw:literal) => {
        $dwords[$dw] = $self_field as u32;
    };

    (@decode_field $ftype:ty, $dwords:ident, $dw:literal, $shift:literal, $mask:literal) => {
        match <$ftype as crate::sdma::FieldDecode>::decode(($dwords[$dw] >> $shift) & $mask) {
            Some(v) => v,
            None => return None,
        }
    };
    (@decode_field $ftype:ty, $dwords:ident, $dw:literal) => {
        match <$ftype as crate::sdma::FieldDecode>::decode($dwords[$dw]) {
            Some(v) => v,
            None => return None,
        }
    };

    (@max_dw $cur:expr) => { $cur };
    (@max_dw $cur:expr, $next:literal $($rest:tt)*) => {
        sdma_packet!(@max_dw {
            let c = $cur;
            let n = $next;
            let gt = (n > c) as u32;
            c * (1 - gt) + n * gt
        } $($rest)*)
    };
}

macro_rules! sdma_packets {
    (
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$attr:meta])*
                $variant:ident : op = $op:literal $(, subop = $subop:literal)? {
                    $(
                        $(#[$fattr:meta])*
                        $field:ident : $ftype:ty = dw[$dw:literal] $(@ shift $shift:literal, mask $mask:literal)?
                    ),* $(,)?
                    $(;
                        $(
                            $combined_field:ident : $combined_type:ty = {
                                lo: dw[$lo_dw:literal],
                                hi: dw[$hi_dw:literal]
                            }
                        ),* $(,)?
                    )?
                }
            ),* $(,)?
        }
    ) => {
        $(
            sdma_packet! {
                $(#[$attr])*
                $variant : op = $op $(, subop = $subop)? {
                    $(
                        $(#[$fattr])*
                        $field : $ftype = dw[$dw] $(@ shift $shift, mask $mask)?,
                    )*
                    $(;
                        $(
                            $combined_field : $combined_type = {
                                lo: dw[$lo_dw],
                                hi: dw[$hi_dw]
                            },
                        )*
                    )?
                }
            }
        )*

        $vis enum $enum_name {
            $($variant($variant),)*
        }

        impl $enum_name {
            pub const fn decode(dwords: &[u32]) -> Option<Self> {
                if dwords.is_empty() {
                    return None;
                }

                let op    = dwords[0] & 0xFF;
                let subop = (dwords[0] >> 8) & 0xFF;

                match (op, subop) {
                    $(
                        sdma_packets!(@op_subop_pat $op $(, $subop)?) => {
                            match $variant::decode(dwords) {
                                Some(pkt) => Some(Self::$variant(pkt)),
                                None      => None,
                            }
                        },
                    )*
                    _ => None,
                }
            }
        }
    };

    (@op_subop_pat $op:literal, $subop:literal) => {
        ($op, $subop)
    };
    (@op_subop_pat $op:literal) => {
        ($op, _)
    };
}

#[allow(non_camel_case_types)]
pub enum Op {
    Noop,

    INDIRECT,   // = 4,
    FENCE,      // = 5,
    TRAP,       // = 6,
    SEM,        // = 7,
    COND_EXE,   // = 9,
    ATOMIC,     // = 10,
    CONST_FILL, // = 11,
    TIMESTAMP,  // = 13,
    SRBM_WRITE, // = 14,
    PRE_EXE,    // = 15,
    GPUVM_INV,  // = 16,
    GCR_REQ,    // = 17,
    DUMMY_TRAP, // = 32,

    // COPY = 1
    COPY_LINEAR,                //  0
    COPY_LINEAR_SUB_WIND,       //  4
    COPY_LINEAR_PHY,            //  8
    COPY_LINEAR_BC,             //  16
    COPY_LINEAR_SUB_WIND_BC,    //  20
    COPY_LINEAR_SUB_WIND_LARGE, //  36
    COPY_TILED,                 //  1
    COPY_TILED_SUB_WIND,        //  5
    COPY_TILED_BC,              //  17
    COPY_TILED_SUB_WIND_BC,     //  21
    COPY_T2T_SUB_WIND,          //  6
    COPY_T2T_SUB_WIND_BC,       //  22
    COPY_SOA,                   //  3
    COPY_DIRTY_PAGE,            //  7

    // WRITE = 2
    WRITE_LINEAR,   //  0
    WRITE_TILED,    //  1
    WRITE_TILED_BC, //  17

    // PTEPDE = 12
    PTEPDE_GEN,            //  0
    PTEPDE_COPY,           //  1
    PTEPDE_RMW,            //  2
    PTEPDE_COPY_BACKWARDS, //  3

    // TIMESTAMP = 13
    TIMESTAMP_SET,        //  0
    TIMESTAMP_GET,        //  1
    TIMESTAMP_GET_GLOBAL, //  2

    MEM_INCR, //  1

    DATA_FILL_MULTI, //  1

    POLL_REGMEM,         // = 8,
    POLL_REG_WRITE_MEM,  //  1
    POLL_DBIT_WRITE_MEM, //  2
    POLL_MEM_VERIFY,     //  3

    VM_INVALIDATION, //  4
}

/// Also known as Iceland (GCN 3)
pub mod v2 {}
/// Also known as Tonga (GCN3, GCN 4)
pub mod v3 {}
/// Also known as Vega (GCN 5)
pub mod v4 {}
/// Also known as Navi (Rdna 1)
///
/// See `kernel/drivers/gpu/drm/amd/amdgpu/navi10_sdma_pkt_open.h`
/// and `kernel/drivers/gpu/drm/amd/include/navi10_enum.h`
pub mod v5 {
    use crate::kfd::ioctl::VirtualAddress;

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

    sdma_packets! {
        pub enum Packet {
            Fence: op = 5 {
                mtype:     Mtype         = dw[0] @ shift 16, mask 0x7,
                gcc:       bool          = dw[0] @ shift 19, mask 0x1,
                sys:       bool          = dw[0] @ shift 20, mask 0x1,
                snp:       bool          = dw[0] @ shift 22, mask 0x1,
                gpa:       bool          = dw[0] @ shift 23, mask 0x1,
                l2_policy: L2Policy      = dw[0] @ shift 24, mask 0x3,
                data:      u32           = dw[3],
                ;
                addr: VirtualAddress = { lo: dw[1], hi: dw[2] },
            },
            Atomic: op = 10 {
                loop_en:       bool = dw[0] @ shift 16, mask 0x1,
                tmz:           bool = dw[0] @ shift 18, mask 0x1,
                atomic_op:     u32  = dw[0] @ shift 25, mask 0x7F,
                loop_interval: u32  = dw[7] @ shift  0, mask 0x1FFF,
                ;
                addr:     u64 = { lo: dw[1], hi: dw[2] },
                src_data: u64 = { lo: dw[3], hi: dw[4] },
                cmp_data: u64 = { lo: dw[5], hi: dw[6] },
            },
            CopyLinear: op = 1, subop = 0 {
                encrypt:   bool = dw[0] @ shift 16, mask 0x1,
                tmz:       bool = dw[0] @ shift 18, mask 0x1,
                backwards: bool = dw[0] @ shift 25, mask 0x1,
                broadcast: bool = dw[0] @ shift 27, mask 0x1,
                count:     u32  = dw[1] @ shift  0, mask 0x003FFFFF,
                dst_sw:    u32  = dw[2] @ shift 16, mask 0x3,
                src_sw:    u32  = dw[2] @ shift 24, mask 0x3,
                ;
                src: u64 = { lo: dw[3], hi: dw[4] },
                dst: u64 = { lo: dw[5], hi: dw[6] },
            },
            Trap: op = 6 {
                int_context: u32 = dw[1] @ shift 0, mask 0x0FFFFFFF,
            },
        }
    }
}
/// (Rdna 3)
pub mod v6 {}
