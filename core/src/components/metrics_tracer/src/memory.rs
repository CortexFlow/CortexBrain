use aya_ebpf::{EbpfContext, programs::TracePointContext};

/// Read the fields of the `syscalls:sys_enter_mmap` tracepoint.
pub fn enter_mmap(ctx: &TracePointContext) -> Result<((u32, u64, u64, [u8; 16])), i64> {
    // For syscall tracepoints `common_pid` is the TGID of the calling thread.
    let tgid_offset = 4;
    let addr_offset = 16;
    let len_offset = 24;

    let tgid: u32 = unsafe { ctx.read_at(tgid_offset) }?;
    let addr: u64 = unsafe { ctx.read_at(addr_offset) }?;
    let len: u64 = unsafe { ctx.read_at(len_offset) }?;
    let command = ctx.command()?;

    Ok((tgid, addr, len, command))
}
