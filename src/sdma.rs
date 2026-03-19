pub type Word = u32;
const OP_FENCE: Word = 5;
const OP_TRAP: Word = 6;

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
    #[repr(u32)]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Default)]
    pub enum Mtype {
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

    #[repr(u32)]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy, Default)]
    pub enum L2Policy {
        #[default]
        LRU = 0x00000000,
        STREAM = 0x00000001,
        NOA = 0x00000002,
        BYPASS = 0x00000003,
    }

    const FENCE_MTYPE_SHIFT: super::Word = 16;
    const FENCE_GCC_SHIFT: super::Word = 19;
    const FENCE_SYS_SHIFT: super::Word = 20;
    const FENCE_SNP_SHIFT: super::Word = 22;
    const FENCE_GPA_SHIFT: super::Word = 23;
    const FENCE_L2_POLICY_SHIFT: super::Word = 24;

    #[derive(Clone, Copy, Default)]
    pub struct Fence {
        pub addr: VirtualAddress,
        pub value: u32,
        pub mtype: Mtype,
        pub gcc: bool,
        pub sys: bool,
        pub snp: bool,
        pub gpa: bool,
        pub l2_policy: L2Policy,
    }
    impl Fence {
        pub const fn enc(self) -> [super::Word; 4] {
            [
                super::OP_FENCE
                    | (self.mtype as u32) << FENCE_MTYPE_SHIFT
                    | (self.gcc as u32) << FENCE_GCC_SHIFT
                    | (self.sys as u32) << FENCE_SYS_SHIFT
                    | (self.snp as u32) << FENCE_SNP_SHIFT
                    | (self.gpa as u32) << FENCE_GPA_SHIFT
                    | (self.l2_policy as u32) << FENCE_L2_POLICY_SHIFT,
                self.addr as u32,
                (self.addr >> 32) as u32,
                self.value,
            ]
        }
    }

    pub struct Trap {
        pub context_id: u32,
    }
    const TRAP_CTX_MASK: super::Word = 0x0FFFFFFF;

    impl Trap {
        pub const fn enc(self) -> [super::Word; 2] {
            [super::OP_TRAP, self.context_id & TRAP_CTX_MASK]
        }
    }
}
/// (Rdna 3)
pub mod v6 {}
