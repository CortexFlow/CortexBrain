//tracepoint:power:cpu_frequency
//tracepoint:power:cpu_frequency_limits
//tracepoint:power:cpu_idle
//tracepoint:power:cpu_idle_miss
use aya_ebpf::{EbpfContext, programs::TracePointContext};
use aya_log_ebpf::info;

use crate::data_structures::{CPU_FREQUENCY, CPU_IDLE, CPU_IDLE_LAST_STATE, CpuFrequency, CpuIdle};

pub fn cpu_idle(ctx: TracePointContext) -> Result<(), i64> {
    let state_offset = 8;
    let cpu_id_offset = 12;
    let state: u32 = unsafe { ctx.read_at(state_offset) }?;
    let cpu_id: u32 = unsafe { ctx.read_at(cpu_id_offset) }?;

    let map_ptr = unsafe { &raw mut CPU_IDLE_LAST_STATE };

    // skip the data when:
    //      - last_state is equal to the current state
    //      - last_state is equal to 4294967295 or -1. This codes means that the cpu is exiting from the current state and entering a new state
    let emit = match unsafe { (*map_ptr).get(&cpu_id) } {
        Some(last_state) if (*last_state == state) || (*last_state == 4294967295) => false,
        _ => true,
    };

    if emit {
        let _ = unsafe { (*map_ptr).insert(&cpu_id, &state, 0) };
        let event = CpuIdle { cpu_id, state };
        unsafe { CPU_IDLE.output(&ctx, &event, 0) };
    }

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

pub fn sched_stat_wait(ctx: &TracePointContext) -> Result<((u32, u64, [u8; 16])), i64> {
    let pid_offset = 4;
    let delay_offset = 16;

    let pid = unsafe { ctx.read_at(pid_offset) }?;

    let delay = unsafe { ctx.read_at(delay_offset) }?;
    let command = ctx.command()?;

    Ok((pid, delay, command))
}

pub fn sched_stat_runtime(ctx: &TracePointContext) -> Result<((u32, u64, [u8; 16])), i64> {
    let pid_offset = 4;
    let runtime_offset = 16;

    let pid = unsafe { ctx.read_at(pid_offset) }?;

    let runtime = unsafe { ctx.read_at(runtime_offset) }?;
    let command = ctx.command()?;

    Ok((pid, runtime, command))
}
