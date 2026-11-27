use aya_ebpf::{
    EbpfContext,
    helpers::{bpf_get_current_pid_tgid, generated::bpf_get_current_cgroup_id},
    programs::TracePointContext,
};

use crate::data_structures::{CPU_FREQUENCY, CPU_IDLE, CPU_IDLE_LAST_STATE, CpuFrequency, CpuIdle};

pub fn cpu_idle(ctx: TracePointContext) -> Result<(), i64> {
    let state_offset = 8;
    let cpu_id_offset = 12;
    let state: u32 = unsafe { ctx.read_at(state_offset) }?;
    let cpu_id: u32 = unsafe { ctx.read_at(cpu_id_offset) }?;
    let cgroup_id = unsafe { bpf_get_current_cgroup_id() };

    // power:cpu_idle fires twice per idle period: once with the C-state being entered
    // and once with PWR_EVENT_EXIT ((u32)-1) when the CPU wakes up.
    // Only C-state changes are reported, so skip:
    //      - exit events (state == PWR_EVENT_EXIT)
    //      - entries into the same state as the last reported one
    // The check used to compare `last_state` against PWR_EVENT_EXIT instead of `state`:
    // the first exit event was stored as the last state, after which every later
    // event on that CPU matched and nothing was emitted again.
    if state == PWR_EVENT_EXIT {
        return Ok(());
    }

    let map_ptr = unsafe { &raw mut CPU_IDLE_LAST_STATE };
    let emit = match unsafe { (*map_ptr).get(&cpu_id) } {
        Some(last_state) => *last_state != state,
        None => true,
    };

    if emit {
        let _ = unsafe { (*map_ptr).insert(&cpu_id, &state, 0) };
        let event = CpuIdle { cpu_id, state };
        unsafe { CPU_IDLE.output(&ctx, &event, 0) };
    }

    Ok(())
}

/// `state` value of power:cpu_idle when a CPU leaves its idle state.
const PWR_EVENT_EXIT: u32 = u32::MAX;

pub fn per_cpu_bytes_alloc(ctx: &TracePointContext) -> Result<((u32, u32, [u8; 16])), i64> {
    let bytes_alloc_offset = 64;
    let pid_offset = 4;
    let bytes_alloc = unsafe { ctx.read_at(bytes_alloc_offset) }?;
    //let tgid: u32 = unsafe { ctx.read_at(tgid_offset) }?;
    let pid_tgid: u64 = bpf_get_current_pid_tgid();
    let tgid: u32 = (pid_tgid >> 32) as u32;
    let command = ctx.command()?;

    //let cpu_freq_data = CpuFrequency {
    //    cpu_id,
    //    cpu_freq: state,
    //};

    //CPU_FREQUENCY.output(&ctx, &cpu_freq_data, 0);

    Ok((bytes_alloc, tgid, command))
}
// sometimes this tracepoint does not work. A possible fix is to enable the tracepoint using
// sudo sysctl kernel.sched_schedstats=1
pub fn sched_stat_wait(ctx: &TracePointContext) -> Result<((u32, u64, [u8; 16], u64)), i64> {
    let delay_offset = 16;
    let command_offset = 8;

    //let tgid: u32 = unsafe { ctx.read_at(tgid_offset) }?;
    let pid_tgid: u64 = bpf_get_current_pid_tgid();
    let tgid: u32 = (pid_tgid >> 32) as u32;
    let cgroup_id: u64 = unsafe { bpf_get_current_cgroup_id() };

    let delay = unsafe { ctx.read_at(delay_offset) }?;
    let command = unsafe{ctx.read_at(command_offset)}?;

    Ok((tgid, delay, command, cgroup_id))
}

pub fn sched_stat_runtime(ctx: &TracePointContext) -> Result<((u32, u64, [u8; 16], u64)), i64> {
    let runtime_offset = 16;

    //let tgid: u32 = unsafe { ctx.read_at(tgid_offset) }?;
    let pid_tgid: u64 = bpf_get_current_pid_tgid();
    let tgid: u32 = (pid_tgid >> 32) as u32;
    let cgroup_id: u64 = unsafe { bpf_get_current_cgroup_id() };

    let runtime = unsafe { ctx.read_at(runtime_offset) }?;
    let command = ctx.command()?;

    Ok((tgid, runtime, command, cgroup_id))
}
