// observe L5 and L6 connections

use crate::data_structures::{SSL_CTX_MAP, SSL_EVENTS, SslEvent};
use aya_ebpf::helpers::{bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_ktime_get_ns};
use aya_ebpf::programs::{ProbeContext, RetProbeContext};

/// store requested bytes keyed by pid_tgid
/// This is the main entrypoint when working with SSL
pub fn try_ssl_start(ctx: &ProbeContext) -> Result<(), i64> {
    let num_bytes = ctx.arg::<i32>(2).ok_or(1i64)?;
    let pid_tgid = unsafe { bpf_get_current_pid_tgid() };
    let map_ptr = unsafe { &raw mut SSL_CTX_MAP };

    unsafe {
        (*map_ptr)
            .insert(&pid_tgid, &num_bytes, 0)
            .map_err(|_| 1i64)?;
    }

    Ok(())
}

// ssl return: emit event with actual transferred bytes
pub fn try_ssl_event_end(ctx: &RetProbeContext, direction: u8) -> Result<(), i64> {
    let size = ctx.ret::<i32>();
    let pid_tgid = unsafe { bpf_get_current_pid_tgid() }; // extracts pid and tgid
    let tgid = (pid_tgid >> 32) as u32; // read only tgid 

    let map_ptr = unsafe { &raw mut SSL_CTX_MAP };
    let requested = unsafe { (*map_ptr).get(&pid_tgid) }.copied().ok_or(1i64)?;

    let comm = unsafe { bpf_get_current_comm() }.map_err(|_| 1i64)?; // get current command that generates the event
    let ts_us = unsafe { bpf_ktime_get_ns() } / 1_000; // get current time in nanoseconds

    let ev = SslEvent {
        tgid,
        comm,
        ts_us,
        direction,
        size,
        requested,
    };

    unsafe {
        SSL_EVENTS.output(ctx, &ev, 0); // emit the event
        (*map_ptr).remove(&pid_tgid); // remove the emitted event from the MAP
    }

    Ok(())
}
