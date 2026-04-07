pub use super::v5_0::*;

packet!(CopyLinear {
    @bits
    dw[0] = {
        & 0x1 << 16 = encrypt: bool;
        & 0x1 << 18 = tmz: bool;
        // added
        & 0x1 << 20 = cache_policy_valid: bool;
        & 0x1 << 25 = backwards: bool;
        & 0x1 << 27 = broadcast: bool;
    }
    dw[1] = {
        // Increased size
        & 0x3fffffff << 0 = count: u32;
    }
    dw[2] = {
        & 0x3 << 16 = dst_sw: u8;
        & 0x7 << 18 = dst_cache_policy: u8;
        & 0x3 << 24 = src_sw: u8;
        & 0x7 << 26 = src_cache_policy: u8;
    }
    @join
    dw[3], dw[4] = src_addr: u64;
    dw[5], dw[6] = dst_addr: u64;
});

packet!(CopyLinearSubwin {
    @bits
    dw[0] = {
        & 0x1 << 18 = tmz: bool;
        // added
        & 0x1 << 19 = cache_policy_valid: bool;
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
        & 0x0fff_ffff << 0 = src_slice_pitch: u32;
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
        & 0x0fffffff << 0 = dst_slice_pitch: u32;
    }
    dw[11] = {
        & 0x3fff << 0 = rect_x: u16;
        & 0x3fff << 16 = rect_y: u16;
    }
    dw[12] = {
        & 0x1fff << 0 = rect_z: u16;
        & 0x3 << 16 = dst_sw: u8;
        & 0x7 << 18 = dst_cache_policy: u8;
        & 0x3 << 24 = src_sw: u8;
        & 0x7 << 26 = src_cache_policy: u8;
    }
    @join
    dw[1], dw[2] = src_addr: u64;
    dw[6], dw[7] = dst_addr: u64;
});

// added
packet!(CopyLinearSubwinLarge {
    @bits
    dw[0] = {
        & 0x1 << 18 = tmz: bool;
        & 0x1 << 19 = cache_policy_valid: bool;
    }
    dw[16] = {
        & 0xffff << 0 = dst_slice_pitch_47_32: u32;
        & 0x3 << 16 = dst_sw: u8;
        & 0x7 << 18 = dst_cache_policy: u8;
        & 0x3 << 24 = src_sw: u8;
        & 0x7 << 26 = src_cache_policy: u8;
    }
    @full
    dw[3] = src_x: u32;
    dw[4] = src_y: u32;
    dw[5] = src_z: u32;
    dw[6] = src_pitch: u32;
    dw[11] = dst_x: u32;
    dw[12] = dst_y: u32;
    dw[13] = dst_z: u32;
    dw[14] = dst_pitch: u32;
    dw[15] = dst_slice_pitch_31_0: u32;
    dw[17] = rect_x: u32;
    dw[18] = rect_y: u32;
    dw[19] = rect_z: u32;
    @join
    dw[1], dw[2] = src_addr: u64;
    /// Only 48 bits
    dw[7], dw[8] = src_slice_pitch: u64;
    dw[9], dw[10] = dst_addr: u64;
});

packet!(CopyPhysicalLinear {
    @bits
    dw[0] = {
        & 0x1 << 18 = tmz: bool;
        // added
        & 0x1 << 19 = cache_policy_valid: bool;
    }
    dw[1] = {
        & 0x3fffff << 0 = count: u32;
        // In umr and pal
        & 0xff << 24 = addr_pair_num: u8;
    }
    dw[2] = {
        & 0x7 << 3 = dst_mtype: u8;
        & 0x3 << 6 = dst_l2_policy: u8;
        // added
        & 0x1 << 8 = dst_llc: bool;
        & 0x7 << 11 = src_mtype: u8;
        & 0x3 << 14 = src_l2_policy: u8;
        // added
        & 0x1 << 16 = src_llc: bool;
        // moved
        & 0x3 << 17 = dst_sw: u8;
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

packet!(CopyStruct {
    @bits
    dw[0] = {
        & 0x1 << 18 = tmz: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
        & 0x1 << 31 = detile: bool;
    }
    dw[5] = {
        & 0x7ff << 0 = stride: u16;
        & 0x3 << 16 = linear_sw: u8;
        // added
        & 0x7 << 18 = linear_cache_policy: u8;
        & 0x3 << 24 = struct_sw: u8;
        // added
        & 0x7 << 26 = struct_cache_policy: u8;
    }
    @full
    dw[3] = start_index: u32;
    dw[4] = count: u32;
    @join
    dw[1], dw[2] = sb_addr: u64;
    dw[6], dw[7] = linear_addr: u64;
});

packet!(CopyT2t {
    @bits
    dw[0] = {
        & 0x1 << 18 = tmz: bool;
        & 0x1 << 19 = dcc: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
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
        // added
        & 0x7 << 18 = dst_cache_policy: u8;
        & 0x3 << 24 = src_sw: u8;
        // added
        & 0x7 << 26 = src_cache_policy: u8;
    }
    dw[17] = {
        & 0x7f << 0 = data_format: u8;
        & 0x1 << 7 = color_transform_disable: bool;
        & 0x1 << 8 = alpha_is_on_msb: bool;
        & 0x7 << 9 = number_type: u8;
        & 0x3 << 12 = surface_type: u8;
        // added
        & 0x1 << 14 = meta_llc: bool;
        & 0x3 << 24 = max_comp_block_size: u8;
        & 0x3 << 26 = max_uncomp_block_size: u8;
        & 0x1 << 28 = write_compress_enable: bool;
        & 0x1 << 29 = meta_tmz: bool;
        & 0x1 << 31 = pipe_aligned: bool;
    }
    @join
    dw[1], dw[2] = src_addr: u64;
    dw[7], dw[8] = dst_addr: u64;
    dw[15], dw[16] = meta_addr: u64;
});

packet!(CopyTiled {
    @bits
    dw[0] = {
        & 0x1 << 16 = encrypt: bool;
        & 0x1 << 18 = tmz: bool;
        // added
        & 0x1 << 19 = cache_policy_valid: bool;
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
        // added
        & 0x7 << 18 = linear_cache_policy: u8;
        // removed
        // & 0x1 << 20 = linear_cc: bool;
        & 0x3 << 24 = tile_sw: u8;
        // added
        & 0x7 << 26 = tile_cache_policy: u8;
    }
    dw[10] = {
        & 0x7ffff << 0 = linear_pitch: u32;
    }
    dw[12] = {
        // increased size
        & 0x3fff_ffff << 0 = count: u32;
    }
    @full
    dw[11] = linear_slice_pitch: u32;
    @join
    dw[1], dw[2] = tiled_addr: u64;
    dw[8], dw[9] = linear_addr: u64;
});

packet!(CopyTiledSubwin {
    @bits
    dw[0] = {
        & 0x1 << 18 = tmz: bool;
        & 0x1 << 19 = dcc: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
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
        // added
        & 0x7 << 18 = linear_cache_policy: u8;
        & 0x3 << 24 = tile_sw: u8;
        // added
        & 0x7 << 26 = tile_cache_policy: u8;
    }
    dw[16] = {
        & 0x7f << 0 = data_format: u8;
        & 0x1 << 7 = color_transform_disable: bool;
        & 0x1 << 8 = alpha_is_on_msb: bool;
        & 0x7 << 9 = number_type: u8;
        & 0x3 << 12 = surface_type: u8;
        // added
        & 0x1 << 14 = meta_llc: bool;
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

packet!(DataFillMulti {
    @bits
    dw[0] = {
        // added
        & 0x7 << 24 = cache_policy: u8;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
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

packet!(Fence {
    @bits
    dw[0] = {
        & 0x7 << 16 = mtype: Mtype;
        & 0x1 << 19 = gcc: bool;
        & 0x1 << 20 = sys: bool;
        & 0x1 << 22 = snp: bool;
        & 0x1 << 23 = gpa: bool;
        & 0x3 << 24 = l2_policy: L2Policy;
        // added
        & 0x1 << 26 = llc_policy: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
    }
    @full
    dw[3] = data: u32;
    @join
    dw[1], dw[2] = addr: u64;
});

packet!(MemIncr {
    @bits
    dw[0] = {
        // added
        & 0x3 << 24 = l2_policy: u8;
        // added
        & 0x1 << 26 = llc_policy: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
    }
    @join
    dw[1], dw[2] = addr: u64;
});

packet!(PollDbitWriteMem {
    @bits
    dw[0] = {
        & 0x3 << 16 = ea: u8;
        // added
        & 0x7 << 24 = cache_policy: u8;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
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
        // added
        & 0x7 << 24 = cache_policy: u8;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
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

packet!(PollRegmem {
    @bits
    dw[0] = {
        // added
        & 0x7 << 20 = cache_policy: u8;
        // added
        & 0x1 << 24 = cache_policy_valid: bool;
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

packet!(PollRegWriteMem {
    @bits
    dw[0] = {
        // added
        & 0x7 << 24 = cache_policy: u8;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
    }
    dw[1] = {
        & 0x3fff_ffff << 2 = src_addr: u32;
    }
    @join
    dw[2], dw[3] = dst_addr: u64;
});

packet!(TimestampGet {
    @bits
    dw[0] = {
        // added
        & 0x3 << 24 = l2_policy: u8;
        // added
        & 0x1 << 26 = llc_policy: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
    }
    dw[1] = {
        & 0x1fffffff << 3 = write_addr_31_3: u32;
    }
    @full
    dw[2] = write_addr_63_32: u32;
});

packet!(TimestampGetGlobal {
    @bits
    dw[0] = {
        // added
        & 0x3 << 24 = l2_policy: u8;
        // added
        & 0x1 << 26 = llc_policy: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
    }
    dw[1] = {
        & 0x1fffffff << 3 = write_addr_31_3: u32;
    }
    @full
    dw[2] = write_addr_63_32: u32;
});

packet!(WriteIncr {
    @bits
    dw[0] = {
        // added
        & 0x7 << 24 = cache_policy: u8;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
    }
    dw[9] = {
        & 0x7ffff << 0 = count: u32;
    }
    @join
    dw[1], dw[2] = dst_addr: u64;
    dw[3], dw[4] = mask: u64;
    dw[5], dw[6] = init: u64;
    dw[7], dw[8] = incr: u64;
});

packet!(WriteTiled<'a> {
    @bits
    dw[0] = {
        & 0x1 << 16 = encrypt: bool;
        & 0x1 << 18 = tmz: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
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
        // added
        & 0x7 << 26 = cache_policy: u8;
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

packet!(WriteUntiled<'a> {
    @bits
    dw[0] = {
        & 0x1 << 16 = encrypt: bool;
        & 0x1 << 18 = tmz: bool;
        // added
        & 0x1 << 28 = cache_policy_valid: bool;
    }
    dw[3] = {
        & 0x3 << 24 = sw: u8;
        // added
        & 0x7 << 26 = cache_policy: u8;
    }
    @join
    dw[1], dw[2] = dst_addr: u64;
    @dyn
    dw[4..] = data0: &'a [u32],
    dw[3] & 0x000fffff << 0 = len
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
    // added
    op = 1, subop = 36 => CopyLinearSubwinLarge
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
    op = 7, subop = 1 => MemIncr
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
    op = 14, subop = 1 => RegisterRmw
    op = 15 => PreExe
    op = 16 => GpuvmInv
    op = 17 => GcrReq
    op = 32 => DummyTrap
});
