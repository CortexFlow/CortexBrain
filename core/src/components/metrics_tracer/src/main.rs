#![no_std]
#![no_main]
#![allow(warnings)]

mod bindings;
mod data_structures;

use crate::bindings::net_device;
use aya_ebpf::EbpfContext;
use aya_ebpf::helpers::{bpf_probe_read_kernel, bpf_probe_read_kernel_str_bytes};
use aya_ebpf::macros::{kprobe, map};
use aya_ebpf::maps::{HashMap, PerfEventArray};
use aya_ebpf::programs::ProbeContext;
use aya_ebpf::helpers::bpf_get_current_pid_tgid;
use crate::data_structures::NetworkMetrics;
use crate::data_structures::NET_METRICS;

#[kprobe]
fn metrics_tracer(ctx: ProbeContext) -> u32 {
    match try_metrics_tracer(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret.try_into().unwrap_or(1),
    }
}

fn try_metrics_tracer(ctx: ProbeContext) -> Result<u32, i64> {
    let sk_pointer = ctx.arg::<*const u8>(0).ok_or(1i64)?;

    if sk_pointer.is_null() {
        return Err(1);
    }

    let sk_err_offset = 284;
    let sk_err_soft_offset = 600;
    let sk_backlog_len_offset = 196;
    let sk_write_memory_queued_offset = 376;
    let sk_receive_buffer_size_offset = 244;
    let sk_ack_backlog_offset = 604;
    let sk_drops_offset = 136;

    let sk_err = unsafe { bpf_probe_read_kernel::<i32>(sk_pointer.add(sk_err_offset) as *const i32).map_err(|_| 1)? };
    let sk_err_soft = unsafe { bpf_probe_read_kernel::<i32>(sk_pointer.add(sk_err_soft_offset) as *const i32).map_err(|_| 1)? };
    let sk_backlog_len = unsafe { bpf_probe_read_kernel::<i32>(sk_pointer.add(sk_backlog_len_offset) as *const i32).map_err(|_| 1)? };
    let sk_write_memory_queued = unsafe { bpf_probe_read_kernel::<i32>(sk_pointer.add(sk_write_memory_queued_offset) as *const i32).map_err(|_| 1)? };
    let sk_receive_buffer_size = unsafe { bpf_probe_read_kernel::<i32>(sk_pointer.add(sk_receive_buffer_size_offset) as *const i32).map_err(|_| 1)? };
    let sk_ack_backlog = unsafe { bpf_probe_read_kernel::<u32>(sk_pointer.add(sk_ack_backlog_offset) as *const u32).map_err(|_| 1)? };
    let sk_drops = unsafe { bpf_probe_read_kernel::<i32>(sk_pointer.add(sk_drops_offset) as *const i32).map_err(|_| 1)? };

    let net_metrics = NetworkMetrics {
        sk_err,
        sk_err_soft,
        sk_backlog_len,
        sk_write_memory_queued,
        sk_receive_buffer_size,
        sk_ack_backlog,
        sk_drops,
    };

    unsafe {
        NET_METRICS.output(&ctx, &net_metrics, 0);
    }

    Ok(0)
}

// Monitor on tcp_sendmsg, tcp_v4_connect


// panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
