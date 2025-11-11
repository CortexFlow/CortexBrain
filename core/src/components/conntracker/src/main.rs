// docs:
// This file contains the code for the identity service
//  Functionalities:
//      1. Creates a PacketLog structure to track incoming packets
//      2. Creates a VethLog structure to track veth creation and veth eletetion events
//      3. VethLog Tracking Parameters: NAME,STATE,DEVICE_ADDRESS,EVENT_TYPE,NETNS INUM.
//      4. PacketLog Tracking Parameters: SRC_IP.SRC_PORT,DST_IP,DST_PORT,PROTOCOL,PID(HOOK)
//      5. Store CONNECTION_ID in a BPF LRU HASHMAP and pass PID to the user space to identify ACTIVE CONNECTIONS
//

#![no_std]
#![no_main]
#![allow(warnings)]

// Imports

mod bindings;
mod data_structures;
mod offsets;
mod tc;
mod tcp_analyzer;
mod veth_tracer;

use aya_ebpf::{
    bindings::{TC_ACT_OK, TC_ACT_SHOT},
    macros::{classifier, kprobe},
    programs::{ProbeContext, TcContext},
};

use crate::tc::try_identity_classifier;
use crate::veth_tracer::try_veth_tracer;
use crate::tcp_analyzer::try_tcp_analyzer;

#[kprobe]
pub fn veth_creation_trace(ctx: ProbeContext) -> u32 {
    match try_veth_tracer(ctx, 1) {
        Ok(ret_val) => ret_val,
        Err(ret_val) => ret_val.try_into().unwrap_or(1),
    }
}
#[kprobe]
pub fn veth_deletion_trace(ctx: ProbeContext) -> u32 {
    match try_veth_tracer(ctx, 2) {
        Ok(ret_val) => ret_val,
        Err(ret_val) => ret_val.try_into().unwrap_or(1),
    }
}

// docs;
// this kprobe retrieves pid data and task id of an incoming packet

#[kprobe]
pub fn tcp_message_tracer(ctx: ProbeContext) -> u32 {
    match try_tcp_analyzer(ctx) {
        Ok(ret_val) => ret_val,
        Err(ret_val) => ret_val.try_into().unwrap_or(1),
    }
}

// docs: this classifier acts in the very first step when a packet is logged

// Linux hooks stack:
//
//  6.Socket Layer
//         |
//  5.TCP Stack
//         |
//  4.Netfilter
//         |
//  3.Traffic control (TC)
//         |
//  2.XDP
//         |
//  1.Network interface
//         |
// Incoming Packet

// so we also need to extract the data from a second source in a kprobe context and correlate the data to catch
// most of the value, without losing the ability to block a packet from the very early stages

#[classifier]
pub fn identity_classifier(ctx: TcContext) -> i32 {
    match try_identity_classifier(ctx) {
        Ok(_) => TC_ACT_OK,
        Err(_) => TC_ACT_SHOT, //block packets that returns errors
    }
}

//ref:https://elixir.bootlin.com/linux/v6.15.1/source/include/uapi/linux/ethtool.h#L536
//https://elixir.bootlin.com/linux/v6.15.1/source/drivers/net/veth.c#L268
//https://eunomia.dev/tutorials/3-fentry-unlink/

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
