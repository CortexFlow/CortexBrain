use aya_ebpf::{
    EbpfContext,
    helpers::{bpf_get_current_cgroup_id, bpf_get_current_pid_tgid},
    programs::TracePointContext,
};

/// Read the fields of the `syscalls:sys_enter_mmap` tracepoint.
pub fn enter_mmap(ctx: &TracePointContext) -> Result<((u32, u64, u64, [u8; 16], u64)), i64> {
    // For syscall tracepoints `common_pid` is the TGID of the calling thread.
    let tgid_offset = 4;
    let addr_offset = 16;
    let len_offset = 24;

    //let tgid: u32 = unsafe { ctx.read_at(tgid_offset) }?;
    let pid_tgid: u64 = bpf_get_current_pid_tgid();
    let tgid: u32 = (pid_tgid >> 32) as u32;
    let cgroup_id: u64 = unsafe { bpf_get_current_cgroup_id() };
    let addr: u64 = unsafe { ctx.read_at(addr_offset) }?;
    let len: u64 = unsafe { ctx.read_at(len_offset) }?;
    let command = ctx.command()?;

    Ok((tgid, addr, len, command, cgroup_id))
}
