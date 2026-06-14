//tracepoint:power:cpu_frequency
//tracepoint:power:cpu_frequency_limits
//tracepoint:power:cpu_idle
//tracepoint:power:cpu_idle_miss
use aya_ebpf::programs::TracePointContext;
use aya_log_ebpf::info;

use crate::data_structures::{CPU_FREQUENCY, CpuFrequency};

//sys/kernel/tracing/events/power/cpu_frequency

pub fn cpu_frequency(ctx: TracePointContext) -> Result<(), i64> {
    let state_offset = 8;
    let cpu_id_offset = 12;
    let state: u32 = unsafe { ctx.read_at(state_offset) }?;
    let cpu_id: u32 = unsafe { ctx.read_at(cpu_id_offset) }?;

    let cpu_freq_data = CpuFrequency {
        cpu_id,
        cpu_freq: state,
    };

    CPU_FREQUENCY.output(&ctx, &cpu_freq_data, 0);
    Ok(())
}
//sys/kernel/tracing/events/power/cpu_idle

pub fn cpu_idle(ctx: TracePointContext) -> Result<(), i64> {
    let state_offset = 8;
    let cpu_id_offset = 12;
    let state: u32 = unsafe { ctx.read_at(state_offset) }?;
    let cpu_id: u32 = unsafe { ctx.read_at(cpu_id_offset) }?;

    info!(&ctx, "CPU idle: State: {} cpu_id: {}", state, cpu_id);
    Ok(())
}
