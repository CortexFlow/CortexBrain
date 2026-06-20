//tracepoint:power:cpu_frequency
//tracepoint:power:cpu_frequency_limits
//tracepoint:power:cpu_idle
//tracepoint:power:cpu_idle_miss
use aya_ebpf::{EbpfContext, programs::TracePointContext};
use aya_log_ebpf::info;

use crate::data_structures::{CPU_FREQUENCY, CpuFrequency};

pub fn cpu_idle(ctx: TracePointContext) -> Result<(), i64> {
    let state_offset = 8;
    let cpu_id_offset = 12;
    let state: u32 = unsafe { ctx.read_at(state_offset) }?;
    let cpu_id: u32 = unsafe { ctx.read_at(cpu_id_offset) }?;

    info!(&ctx, "CPU idle: State: {} cpu_id: {}", state, cpu_id);
    Ok(())
}

pub fn per_cpu_bytes_alloc(ctx: &TracePointContext) -> Result<((u32, u32, [u8; 16])), i64> {
    let bytes_alloc_offset = 64;
    let pid_offset = 4;
    let bytes_alloc = unsafe { ctx.read_at(bytes_alloc_offset) }?;
    let pid = unsafe { ctx.read_at(pid_offset) }?;
    let command = ctx.command()?;

    //let cpu_freq_data = CpuFrequency {
    //    cpu_id,
    //    cpu_freq: state,
    //};

    //CPU_FREQUENCY.output(&ctx, &cpu_freq_data, 0);

    Ok((bytes_alloc, pid, command))
}
