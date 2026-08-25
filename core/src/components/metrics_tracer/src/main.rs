#![no_std]
#![no_main]
#![allow(warnings)]

mod bindings;
mod cpu;
mod data_structures;
mod memory;
mod network;
mod ssl;

use crate::bindings::net_device;
use crate::cpu::{cpu_idle, per_cpu_bytes_alloc, sched_stat_runtime, sched_stat_wait};
use crate::data_structures::CpuFrequency;
use crate::data_structures::NET_METRICS;
use crate::data_structures::{CPU_FREQUENCY, SchedStatWait};
use crate::data_structures::{
    CPU_IDLE, PacketLossMetrics, TASK_COMM_LEN, TIME_STAMP_EVENTS, TIME_STAMP_START,
    TimeStampEvent, TimeStampStartInfo,
};
use crate::data_structures::{MEM_ALLOC, SCHED_STAT_RUNTIME, SCHED_STAT_WAIT};
use crate::data_structures::{MemAlloc, SchedStatRuntime};
use crate::memory::enter_mmap;
use crate::network::{detect_packet_loss, on_connect, on_rcv_state_process};
use crate::ssl::{try_ssl_event_end, try_ssl_start};
use aya_ebpf::EbpfContext;
use aya_ebpf::helpers::bpf_get_current_pid_tgid;
use aya_ebpf::helpers::generated::{bpf_ktime_get_ns, bpf_perf_event_output};
use aya_ebpf::helpers::{
    bpf_get_current_comm, bpf_probe_read_kernel, bpf_probe_read_kernel_str_bytes,
};
use aya_ebpf::macros::{kprobe, map, tracepoint, uprobe, uretprobe};
use aya_ebpf::maps::{HashMap, PerfEventArray};
use aya_ebpf::programs::{ProbeContext, RetProbeContext, TracePointContext};
use core::{mem, ptr};

const AF_INET: u16 = 2;
const AF_INET6: u16 = 10;
const TCP_SYN_SENT: u8 = 2;

#[kprobe]
fn packet_loss_tracer(ctx: ProbeContext) -> u32 {
    match try_metrics_tracer(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret.try_into().unwrap_or(1),
    }
}

fn try_metrics_tracer(ctx: ProbeContext) -> Result<u32, i64> {
    let net_metrics = detect_packet_loss(&ctx)?;
    unsafe {
        NET_METRICS.output(&ctx, &net_metrics, 0);
    }

    Ok(0)
}

/// Monitor on tcp_sendmsg, tcp_v6_connect
#[kprobe]
fn tcp_v6_connect(ctx: ProbeContext) -> u32 {
    match on_connect(ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

/// Monitor on tcp_sendmsg, tcp_v4_connect
#[kprobe]
fn tcp_v4_connect(ctx: ProbeContext) -> u32 {
    match on_connect(ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

#[kprobe]
fn tcp_latency_monitor(ctx: ProbeContext) -> u32 {
    match on_rcv_state_process(ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

#[tracepoint]
fn trace_cpu_frequency(ctx: TracePointContext) -> u32 {
    match trace_cpu_metrics(&ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

#[tracepoint]
fn trace_cpu_idle(ctx: TracePointContext) -> u32 {
    match cpu_idle(ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

fn trace_cpu_metrics(ctx: &TracePointContext) -> Result<(), i64> {
    let (bytes_alloc, tgid, command) = per_cpu_bytes_alloc(ctx)?;
    //let (cpu_id, cpu_freq) = cpu_frequency(&ctx)?;
    let cpu_metrics = CpuFrequency {
        //    cpu_id,
        //   cpu_freq,
        bytes_alloc,
        tgid,
        command,
    };

    unsafe { CPU_FREQUENCY.output(ctx, &cpu_metrics, 0) };

    Ok(())
}

/// Tracepoint attached to `syscalls:sys_enter_mmap`.
///
/// Emits a `MemAlloc` event for every `mmap` syscall.  No PID/command filter
/// is applied yet (see the next update), so this will generate events for every
/// process in the system.
#[tracepoint]
fn trace_enter_mmap(ctx: TracePointContext) -> u32 {
    match trace_memory_allocation(&ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

fn trace_memory_allocation(ctx: &TracePointContext) -> Result<(), i64> {
    let (tgid, addr, length, command, cgroup_id) = enter_mmap(ctx)?;

    let memory_alloc_metrics = MemAlloc {
        tgid,
        addr,
        length,
        command,
        cgroup_id,
    };

    unsafe { MEM_ALLOC.output(ctx, &memory_alloc_metrics, 0) };

    Ok(())
}

#[tracepoint]
fn trace_sched_stat_wait(ctx: TracePointContext) -> u32 {
    match sched_stat_wait_tracer(&ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

fn sched_stat_wait_tracer(ctx: &TracePointContext) -> Result<(), i64> {
    let (tgid, delay, command, cgroup_id) = sched_stat_wait(ctx)?;

    let sched_stat_wait_data = SchedStatWait {
        tgid,
        delay,
        command,
        cgroup_id,
    };

    unsafe { SCHED_STAT_WAIT.output(ctx, &sched_stat_wait_data, 0) };

    Ok(())
}

#[tracepoint]
fn trace_sched_stat_runtime(ctx: TracePointContext) -> u32 {
    match sched_stat_runtime_tracer(&ctx) {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}

fn sched_stat_runtime_tracer(ctx: &TracePointContext) -> Result<(), i64> {
    let (tgid, runtime, command, cgroup_id) = sched_stat_runtime(ctx)?;

    let sched_stat_runtime_data = SchedStatRuntime {
        tgid,
        runtime,
        command,
        cgroup_id,
    };

    unsafe { SCHED_STAT_RUNTIME.output(ctx, &sched_stat_runtime_data, 0) };

    Ok(())
}

const SSL_READ_DIR: u8 = 0;
const SSL_WRITE_DIR: u8 = 1;

#[uprobe]
fn ssl_read(ctx: ProbeContext) -> u32 {
    match try_ssl_start(&ctx) {
        Ok(_) => 0,
        Err(_) => 0, // fail silently to avoid perturbing the application
    }
}

#[uretprobe]
fn ssl_read_ret(ctx: RetProbeContext) -> u32 {
    match try_ssl_event_end(&ctx, SSL_READ_DIR) {
        Ok(_) => 0,
        Err(_) => 0,
    }
}

// ssl write
#[uprobe]
//uprobe reads input data from the userspace
fn ssl_write(ctx: ProbeContext) -> u32 {
    match try_ssl_start(&ctx) {
        Ok(_) => 0,
        Err(_) => 0,
    }
}

#[uretprobe]
//uretprobe best fits for measuring returning data
fn ssl_write_ret(ctx: RetProbeContext) -> u32 {
    match try_ssl_event_end(&ctx, SSL_WRITE_DIR) {
        Ok(_) => 0,
        Err(_) => 0,
    }
}
// panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
