pub use super::v2_4::{
    CondExe, ConstFill, CopyLinear, CopyLinearBroadcast, CopyLinearSubWindow,
    CopyLinearToTiledBroadcast, CopyStruct, CopyTiled, CopyTiledSubWindow, CopyTiledToTiled, Fence,
    GenPtepde, IndirectBuffer, Nop, PollRegmem, PreExe, Semaphore, SrbmWrite, TimestampGet,
    TimestampGetGlobal, TimestampSet, Trap, WriteTiled, WriteUntiled,
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
