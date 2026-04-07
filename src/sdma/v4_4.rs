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
