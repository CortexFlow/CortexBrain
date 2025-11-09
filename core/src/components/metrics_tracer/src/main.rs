#![no_std]
#![no_main]
#![allow(warnings)]

mod bindings;
mod data_structures;

use core::{mem, ptr};

use crate::bindings::net_device;
use aya_ebpf::helpers::generated::{bpf_ktime_get_ns, bpf_perf_event_output};
use aya_ebpf::EbpfContext;
use aya_ebpf::helpers::{bpf_get_current_comm, bpf_probe_read_kernel, bpf_probe_read_kernel_str_bytes};
use aya_ebpf::macros::{kprobe, map};
use aya_ebpf::maps::{HashMap, PerfEventArray};
use aya_ebpf::programs::ProbeContext;
use aya_ebpf::helpers::bpf_get_current_pid_tgid;
use crate::data_structures::{NetworkMetrics, TimeStampEvent, TimeStampStartInfo, TASK_COMM_LEN, TIME_STAMP_EVENTS, TIME_STAMP_START};
use crate::data_structures::NET_METRICS;

const AF_INET: u16  = 2;
const AF_INET6: u16 = 10;
const TCP_SYN_SENT: u8 = 2;

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

    let tgid = (unsafe { bpf_get_current_pid_tgid() } >> 32) as u32;
    let comm = unsafe { bpf_get_current_comm() }.map_err(|_| 1i64)?;
    let ts_us: u64 = unsafe { bpf_ktime_get_ns() } / 1_000;
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
        tgid,
        comm,
        ts_us,
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
#[kprobe]
fn tcp_connect(ctx: ProbeContext) -> u32 {
    match on_connect(ctx) { Ok(_) => 0, Err(e) => e as u32 }
}


fn on_connect(ctx: ProbeContext) -> Result<(), i64> {
    let sk = ctx.arg::<*mut bindings::sock>(0).ok_or(1i64)?;
    if sk.is_null() {
        return Err(1);
    }

    let tgid = (unsafe { bpf_get_current_pid_tgid() } >> 32) as u32;
    let mut start = TimeStampStartInfo {
        comm: [0; TASK_COMM_LEN],
        ts_ns: unsafe { bpf_ktime_get_ns() },
        tgid,
    };
    unsafe {
        let comm_result = bpf_get_current_comm();
        if let Ok(comm) = comm_result {
            start.comm.copy_from_slice(&comm);
        }
        let map_ptr = &raw mut TIME_STAMP_START;
        (*map_ptr).insert(&(sk as *mut core::ffi::c_void), &start, 0).map_err(|_| 1)?;
    }
    Ok(())
}

#[kprobe]
fn tcp_rcv_state_process(ctx: ProbeContext) -> u32 {
    match on_rcv_state_process(ctx) { Ok(_) => 0, Err(e) => e as u32 }
}

fn on_rcv_state_process(ctx: ProbeContext) -> Result<(), i64> {
    // On some kernels, kprobe wrapper puts `sk` at arg0; on others arg1.
    let sk = ctx.arg::<*mut bindings::sock>(0).unwrap_or(ptr::null_mut());
    let sk = if sk.is_null() {
        ctx.arg::<*mut bindings::sock>(1).ok_or(1i64)?
    } else { sk };

    if sk.is_null() {
        return Err(1);
    }

    let skc_daddr_off = 0;
    let skc_rcv_saddr_off  = 4;
    let skc_dport_off = 12;
    let skc_num_off = 14;
    let skc_family_off = 16;
    let skc_state_off = 18;
    let skc_v6_daddr_off = 56; 
    let skc_v6_rcv_saddr_off = 72; 

    let state = unsafe {
        bpf_probe_read_kernel::<u8>((sk as usize + skc_state_off) as *const u8)
    }.map_err(|_| 1)?;


    if state != TCP_SYN_SENT {
        return Ok(());
    }

    let start = unsafe {
        let map_ptr = &raw const TIME_STAMP_START;
        (*map_ptr).get(&((sk as usize) as *mut core::ffi::c_void))
    }.ok_or(1i64)?;
    let now = unsafe { bpf_ktime_get_ns() };
    let delta = now as i64 - start.ts_ns as i64;
    if delta <= 0 {
        unsafe {
            let map_ptr = &raw mut TIME_STAMP_START;
            let _ = (*map_ptr).remove(&((sk as usize) as *mut core::ffi::c_void));
        }
        return Ok(());
    }

    let mut ev = TimeStampEvent {
        delta_us: (delta as u64) / 1_000,
        ts_us: now / 1_000,
        tgid: start.tgid,
        comm: start.comm,
        lport: 0,
        dport_be: 0,
        af: 0,
        saddr_v4: 0,
        daddr_v4: 0,
        saddr_v6: [0; 4],
        daddr_v6: [0; 4],
    };

    // family, ports
    ev.af = unsafe {
        bpf_probe_read_kernel::<u16>((sk as usize + skc_family_off) as *const u16)
            .map_err(|_| 1)?
    };
    ev.lport = unsafe {
        bpf_probe_read_kernel::<u16>((sk as usize + skc_num_off) as *const u16)
            .map_err(|_| 1)?
    };
    ev.dport_be = unsafe {
        bpf_probe_read_kernel::<u16>((sk as usize + skc_dport_off) as *const u16)
            .map_err(|_| 1)?
    };

    if ev.af == AF_INET {
        ev.saddr_v4 = unsafe {
            bpf_probe_read_kernel::<u32>((sk as usize + skc_rcv_saddr_off) as *const u32)
                .map_err(|_| 1)?
        };
        ev.daddr_v4 = unsafe {
            bpf_probe_read_kernel::<u32>((sk as usize + skc_daddr_off) as *const u32)
                .map_err(|_| 1)?
        };
    } else {
        // read 16 bytes as four u32 words
        for i in 0..4 {
            ev.saddr_v6[i] = unsafe {
                bpf_probe_read_kernel::<u32>(
                    (sk as usize + skc_v6_rcv_saddr_off + i * 4) as *const u32
                ).map_err(|_| 1)?
            };
            ev.daddr_v6[i] = unsafe {
                bpf_probe_read_kernel::<u32>(
                    (sk as usize + skc_v6_daddr_off + i * 4) as *const u32
                ).map_err(|_| 1)?
            };
        }
    }

    // emit + cleanup
    unsafe {
        bpf_perf_event_output(
            ctx.as_ptr(),
            &raw const TIME_STAMP_EVENTS as *const _ as *mut _,
            0, // BPF_F_CURRENT_CPU
            &ev as *const _ as *mut _,
            (mem::size_of::<TimeStampEvent>() as u32).into(),
        );
        let map_ptr = &raw mut TIME_STAMP_START;
        let _ = (*map_ptr).remove(&((sk as usize) as *mut core::ffi::c_void));
    }


    Ok(())
}



// panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
