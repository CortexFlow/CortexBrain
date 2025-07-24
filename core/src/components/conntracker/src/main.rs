/*
    * This file contains the code for the identity service
    *

    * Functionalities:
    *   1. Creates a PacketLog structure to track incoming packets
    *   2. Creates a VethLog structure to track veth creation and veth eletetion events
    *   3. VethLog Tracking Parameters: NAME,STATE,DEVICE_ADDRESS,EVENT_TYPE,NETNS INUM.
    *   4. PacketLog Tracking Parameters: SRC_IP.SRC_PORT,DST_IP,DST_PORT,PROTOCOL,PID(HOOK)
    *   5. Store CONNECTION_ID in a BPF LRU HASHMAP and pass PID to the user space to identify ACTIVE CONNECTIONS
*/

// Imports
#![no_std]
#![no_main]
#![allow(warnings)]

mod bindings;
mod data_structures;

use aya_ebpf::{
    bindings::{TC_ACT_OK, TC_ACT_SHOT},
    bpf_printk,
    helpers::{bpf_get_current_pid_tgid, bpf_probe_read_kernel},
    macros::{classifier, kprobe},
    programs::{ProbeContext, TcContext},
};
use aya_log_ebpf::info;

use crate::bindings::{net, net_device, ns_common, possible_net_t};
use crate::data_structures::{ConnArray, PacketLog, VethLog};
use crate::data_structures::{ACTIVE_CONNECTIONS, CONNTRACKER, EVENTS, VETH_EVENTS};
/*
    * ETHERNET TYPE II FRAME:
    * Reference: https://it.wikipedia.org/wiki/Frame_Ethernet
    *
    * 6 bytes for the destination mac address;
    * 6 bytes for the source mac address;
    * 2 bytes for the ethertype;
    * from 46 to 1500 bytes for the payload;
    * 4 bytes for the FCS (Checksum)

*/

/*
    * Ipv4 stack reference:
    * https://en.wikipedia.org/wiki/IPv4#Header
    *
    * Original reference:

   |  Time to Live |    Protocol   |         Header Checksum       |     4 bytes            12
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                       Source Address                          |     4 bytes            16
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Destination Address                        |     4 bytes            20
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Options                    |    Padding    |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    *                    Internet Datagram Header
    *
    *
    *
    *
    *
    *                        TCP/UDP header datagram:
    *
    *   reference: https://en.wikipedia.org/wiki/User_Datagram_Protocol
    *
    *    src port: 34 byte
    *    dst port: 36 byte

*/

const IPV4_ETHERTYPE: u16 = 0x0800;

//IPV4 STACK
const SRC_BYTE_OFFSET: usize = 12;
const DST_BYTE_OFFSET: usize = 16;
const IPV4_PROTOCOL_OFFSET: usize = 9;

//ETHERNET STACK
const SRC_MAC: usize = 6;
const DST_MAC: usize = 6;
const ETHERTYPE_BYTES: usize = 2;

//TCP UDP Stack
const SRC_PORT_OFFSET_FROM_IP_HEADER: usize = 0;
const DST_PORT_OFFSET_FROM_IP_HEADER: usize = 2;

static ETH_STACK_BYTES: usize = SRC_MAC + DST_MAC + ETHERTYPE_BYTES;
static DST_T0TAL_BYTES_OFFSET: usize = ETH_STACK_BYTES + DST_BYTE_OFFSET;
static SRC_T0TAL_BYTES_OFFSET: usize = ETH_STACK_BYTES + SRC_BYTE_OFFSET;
static PROTOCOL_T0TAL_BYTES_OFFSET: usize = ETH_STACK_BYTES + IPV4_PROTOCOL_OFFSET;

const AF_INET: u16 = 2; //ipv4
const AF_INET6: u16 = 10; //ipv6

const IPPROTO_UDP: u8 = 17;
const IPPROTO_TCP: u8 = 6;

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

//read linux inner struct. takes a ptr to the structure and an offset
fn read_linux_inner_struct<T>(ptr: *const u8, offset: usize) -> Result<*const T, i64> {
    if ptr.is_null() {
        return Err(1);
    } else {
        let inner_ptr = unsafe { (ptr as *const u8).add(offset) };

        let inner_field: *const T = unsafe {
            match bpf_probe_read_kernel(inner_ptr as *const *const T) {
                Ok(inner_field) => inner_field,
                Err(e) => return Err(e),
            }
        };
        Ok(inner_field)
    }
}

//T= type of return
fn read_linux_inner_value<T: Copy>(ptr: *const u8, offset: usize) -> Result<T, i64> {
    if ptr.is_null() {
        return Err(1);
    }

    let inner_ptr = unsafe { (ptr as *const u8).add(offset) };

    let inner_value = unsafe {
        match bpf_probe_read_kernel::<T>(inner_ptr as *const T) {
            Ok(inner_field) => inner_field,
            Err(e) => return Err(e),
        }
    };

    Ok(inner_value)
}

fn extract_netns_inum(net_device_pointer: *const u8) -> Result<u32, i64> {
    let possible_net_t_offset = 280;

    let net = read_linux_inner_struct::<net>(net_device_pointer, possible_net_t_offset)?;

    let ns_common_offset = 120;

    let inum_offset = 16;
    let inum_ptr = read_linux_inner_value::<u32>(net as *const u8, ns_common_offset + inum_offset)?;
    Ok(inum_ptr)
}

//mode selection:
//1->veth_creation_tracer
//2->veth_deletion_tracer
pub fn try_veth_tracer(ctx: ProbeContext, mode: u8) -> Result<u32, i64> {
    let net_device_pointer: *const net_device = ctx.arg(0).ok_or(1i64)?;

    // first control: i'm, verifying that the pointer is not null
    if net_device_pointer.is_null() {
        return Err(1);
    }

    let mut name_buf = [0u8; 16];
    let mut dev_addr_buf = [0u32; 8];

    //name field
    let name_field_offset = 304; // reading the name field offset

    let name_array: [u8; 16] =
        read_linux_inner_value::<[u8; 16]>(net_device_pointer as *const u8, name_field_offset)?;

    //state field
    let state_offset = 168;
    let state: u8 = read_linux_inner_value::<u8>(net_device_pointer as *const u8, state_offset)?;

    //dev_addr
    let dev_addr_offset = 1080;
    let dev_addr_array: [u32; 8] =
        read_linux_inner_value::<[u32; 8]>(net_device_pointer as *const u8, dev_addr_offset)?;

    let inum: u32 = extract_netns_inum(net_device_pointer as *const u8)?;
    let pid: u32 = (bpf_get_current_pid_tgid() << 32) as u32; //extracting lower 32 bit corresponding to the PID

    //buffer copying for array types
    name_buf.copy_from_slice(&name_array);
    dev_addr_buf.copy_from_slice(&dev_addr_array);

    //compose the structure
    let veth_data = VethLog {
        name: name_buf,
        state: state.into(),
        dev_addr: dev_addr_buf,
        event_type: mode,
        netns: inum,
        pid,
    };

    //send the data to the userspace
    unsafe {
        VETH_EVENTS.output(&ctx, &veth_data, 0);
    }

    Ok(0)
}

#[classifier]
pub fn identity_classifier(ctx: TcContext) -> i32 {
    match try_identity_classifier(ctx) {
        Ok(_) => TC_ACT_OK,
        Err(_) => TC_ACT_SHOT, //block packets that returns errors
    }
}

fn try_identity_classifier(ctx: TcContext) -> Result<(), i64> {
    let eth_proto = u16::from_be(ctx.load::<u16>(12).map_err(|_| 1)?);

    //only ipv4 protcol allowed
    if eth_proto != IPV4_ETHERTYPE {
        return Ok(());
    }

    //read if the packets has Options
    let first_ipv4_byte = u8::from_be(ctx.load::<u8>(ETH_STACK_BYTES).map_err(|_| 1)?);
    let ihl = (first_ipv4_byte & 0x0f) as usize; /* 0x0F=00001111 &=AND bit a bit operator to extract the last 4 bit*/
    let ip_header_len = ihl * 4; //returns the header lenght in bytes

    //get the source ip,destination ip and connection id
    let src_ip = u32::from_be(ctx.load::<u32>(SRC_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+SOURCE_ADDRESS
    let src_port = u16::from_be(
        ctx.load::<u16>(ETH_STACK_BYTES + ip_header_len + SRC_PORT_OFFSET_FROM_IP_HEADER)
            .map_err(|_| 1)?,
    ); //14+IHL-Lenght+0
    let dst_ip = u32::from_be(ctx.load::<u32>(DST_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+ DESTINATION_ADDRESS
    let dst_port = u16::from_be(
        ctx.load::<u16>(ETH_STACK_BYTES + ip_header_len + DST_PORT_OFFSET_FROM_IP_HEADER)
            .map_err(|_| 1)?,
    ); //14+IHL-Lenght+0
    let proto = u8::from_be(ctx.load::<u8>(PROTOCOL_T0TAL_BYTES_OFFSET).map_err(|_| 1)?);

    let pid = bpf_get_current_pid_tgid() & 0xFFFFFFFF;

    //not logging internal communication packets
    //TODO: do not log internal communications such as minikube dashboard packets or kubectl api packets

    let key = ConnArray {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        proto,
    };

    let log = PacketLog {
        proto,
        src_ip,
        src_port,
        dst_ip,
        dst_port,
        pid,
    };
    //let connections = ConnArray{
    //  event_id,
    //connection_id
    //};
    unsafe {
        EVENTS.output(&ctx, &log, 0); //output to userspace
    }
    Ok(())
}

//ref:https://elixir.bootlin.com/linux/v6.15.1/source/include/uapi/linux/ethtool.h#L536
//https://elixir.bootlin.com/linux/v6.15.1/source/drivers/net/veth.c#L268
//https://eunomia.dev/tutorials/3-fentry-unlink/

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
