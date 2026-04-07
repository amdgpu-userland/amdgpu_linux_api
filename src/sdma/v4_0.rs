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
        & 0x3fff_ffff << 2 = src_addr: u32;
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
        & 0x0fff_ffff << 4 = start_page: u32;
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
