#![no_std]
#![no_main]
#![allow(warnings)]

mod bindings;

use crate::bindings::net_device;
use aya_ebpf::EbpfContext;
use aya_ebpf::helpers::{bpf_probe_read_kernel, bpf_probe_read_kernel_str_bytes};
use aya_ebpf::macros::{kprobe, map};
use aya_ebpf::maps::PerfEventArray;
use aya_ebpf::programs::ProbeContext;

struct NetworkMetrics {
    src_addr: [u32; 8],
}

#[map(name = "net_metrics")]
pub static NET_METRICS: PerfEventArray<NetworkMetrics> = PerfEventArray::new(0);

#[kprobe]
fn metrics_tracer(ctx: ProbeContext) -> u32 {
    match try_metrics_tracer(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret.try_into().unwrap_or(1),
    }
}

fn try_metrics_tracer(ctx: ProbeContext) -> Result<u32, i64> {
    let net_device_pointer: *const net_device = ctx.arg(0).ok_or(1i64)?;

    if net_device_pointer.is_null() {
        return Err(1);
    }

    //let sk_pacing_status_offset = 428;
    //let sk_pacing_status_pointer =
    //  unsafe { (sk_pacing_status_offset as *const u8).add(sk_pacing_status_offset) };

    //let sk_pacing_status = unsafe {
    //match bpf_probe_read_kernel(sk_pacing_status_pointer) {
    //Ok(value) => value,
    //Err(ret) => {
    //      return Err(ret);
    //    }
    //  }
    //};

    let dev_addr_offset = 1080;

    let dev_addr_pointer = unsafe { (net_device_pointer as *const u8).add(dev_addr_offset) };

    let mut dev_addr_buf = [0u32; 8];

    let dev_addr_ptr_array = dev_addr_pointer as *const [u32; 8];
    let dev_addr_array = unsafe {
        match bpf_probe_read_kernel(dev_addr_ptr_array) {
            Ok(arr) => arr,
            Err(ret) => {
                return Err(ret);
            }
        }
    };

    dev_addr_buf.copy_from_slice(&dev_addr_array);

    let net_metrics = NetworkMetrics {
        src_addr: dev_addr_buf,
    };

    unsafe {
        NET_METRICS.output(&ctx, &net_metrics, 0);
    }

    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
