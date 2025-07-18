/*  TODO: this part needs an update
    *
    * This file contains the code for the identity service  
    *
    
    * Functionalities:
    *   1. Creates a PacketLog structure to track incoming packets
    *   2. Tracking Parameters: SRC_IP.SRC_PORT,DST_IP,DST_PORT,PROTOCOL,HASH
    *   3. Compute the EVENT_ID and CONNECTION_ID using a byte XOR 
    *   4. Store CONNECTION_ID in a BPF LRU HASHMAP and pass EVENT_ID to the user space
*/

// Imports
#![no_std]
#![no_main]
#![allow(warnings)]

//mod skbuff;
//mod veth_trace;
mod bindings;

use bytemuck::{ Pod, Zeroable };
use aya_ebpf::{
    bindings::{ TC_ACT_OK, TC_ACT_SHOT },
    helpers::{ bpf_ktime_get_ns, bpf_probe_read_kernel, bpf_probe_read_kernel_str_bytes },
    macros::{ classifier, kprobe, map, tracepoint },
    maps::{ LruPerCpuHashMap, PerfEventArray },
    programs::{ ProbeContext, TcContext, TracePointContext },
};
use aya_ebpf::EbpfContext;
//use crate::skbuff::{ sock, sock_common };
use aya_log_ebpf::info;
use core::{ mem, ptr };
//use crate::skbuff::proto;
//use crate::skbuff::{ iphdr };
//use crate::skbuff::sk_buff;
use network_types::{
    eth::{ EthHdr, EtherType },
    ip::{ IpProto, Ipv4Hdr },
    tcp::TcpHdr,
    udp::UdpHdr,
};
use core::ptr::addr_of;

use crate::bindings::net_device;
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PacketLog {
    pub proto: u8,
    pub src_ip: u32,
    pub src_port: u16,
    pub dst_ip: u32,
    pub dst_port: u16,
    pub event_id: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnArray {
    pub src_ip: u32,
    pub dst_ip: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: u8,
}

#[map(name = "EventsMap")]
static mut EVENTS: PerfEventArray<PacketLog> = PerfEventArray::new(0);
//TODO: ConnectionMap needs a rework after implementing issue #105
#[map(name = "ConnectionMap")]
pub static mut ACTIVE_CONNECTIONS: LruPerCpuHashMap<
    u16,
    ConnArray
> = LruPerCpuHashMap::with_max_entries(65536, 0);

#[map(name = "ConnectionTrackerMap")]
pub static mut CONNTRACKER: LruPerCpuHashMap<ConnArray, u8> = LruPerCpuHashMap::with_max_entries(
    65536,
    0
);

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Debug)]
struct VethLog {
    name: [u8; 16],
    state: u64, //state var type: long unsigned int
    dev_addr: [u32; 8],
    event_type: u8, //i choose 1 for veth creation or 2 for veth destruction
}

#[map(name = "veth_identity_map")]
pub static mut VETH_CREATION_EVENTS: PerfEventArray<VethLog> = PerfEventArray::new(0);

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

/* constants */
//FIXME: this will be deprecated after solving issue #105
const HOST_NETNS_INUM: u32 = 4026531993;
const KUBE_POD_CIDR: u32 = 0x0af40000; // 10.244.0.0/16
/* Helper Functions */
#[inline]
unsafe fn is_kube_internal(ip: u32) -> bool {
    (ip & 0xffff0000) == KUBE_POD_CIDR
}

#[kprobe]
pub fn veth_creation_trace(ctx: ProbeContext) -> u32 {
    match try_veth_creation_trace(ctx) {
        Ok(ret_val) => ret_val,
        Err(ret_val) => ret_val.try_into().unwrap_or(1),
    }
}
#[kprobe]
pub fn veth_deletion_trace(ctx: ProbeContext) -> u32 {
    match try_veth_deletion_trace(ctx) {
        Ok(ret_val) => ret_val,
        Err(ret_val) => ret_val.try_into().unwrap_or(1),
    }
}


pub fn try_veth_creation_trace(ctx: ProbeContext) -> Result<u32, i64> {
    let net_device_pointer: *const net_device = ctx.arg(0).ok_or(1i64)?;

    // first control: i'm, verifying that the pointer is not null
    if net_device_pointer.is_null() {
        return Err(1);
    }

    let name_field_offset = 304; // reading the name field offset
    //pahole commands:
    //syntax pahole -C <struct name>
    // pahole -C net_device | grep name

    let dev_addr_offset = 1080;
    let state_offset = 168;

    let name_pointer = unsafe { (net_device_pointer as *const u8).add(name_field_offset) };
    let dev_addr_pointer = unsafe { (net_device_pointer as *const u8).add(dev_addr_offset) };
    let state_pointer = unsafe { (net_device_pointer as *const u8).add(state_offset) };

    let mut name_buf = [0u8; 16];
    let mut dev_addr_buf = [0u32; 8];

    let name_array_ptr = name_pointer as *const [u8; 16];
    let dev_addr_ptr_array = dev_addr_pointer as *const [u32; 8];

    let name_array = unsafe {
        match bpf_probe_read_kernel(name_array_ptr) {
            Ok(arr) => arr,
            Err(ret) => {
                return Err(ret);
            }
        }
    };

    let state=unsafe {
        match bpf_probe_read_kernel(state_pointer) {
            Ok(s)=>s,
            Err(ret)=>{
                return Err(ret);
            }
        }
    };

    let dev_addr_array = unsafe {
        match bpf_probe_read_kernel(dev_addr_ptr_array) {
            Ok(arr) => arr,
            Err(ret) => {
                return Err(ret);
            }
        }
    };

    name_buf.copy_from_slice(&name_array);
    dev_addr_buf.copy_from_slice(&dev_addr_array);

    let veth_data = VethLog {
        name: name_buf,
        state: state.into(),
        dev_addr: dev_addr_buf,
        event_type: 1
    };

    //send the data to the userspace
    unsafe {
        VETH_CREATION_EVENTS.output(&ctx, &veth_data, 0);
    }

    Ok(0)
}

pub fn try_veth_deletion_trace(ctx: ProbeContext) -> Result<u32, i64> {
    let net_device_pointer: *const net_device = ctx.arg(0).ok_or(1i64)?;

    // first control: i'm, verifying that the pointer is not null
    if net_device_pointer.is_null() {
        return Err(1);
    }

    let name_field_offset = 304; // reading the name field offset
    //pahole commands:
    //syntax pahole -C <struct name>
    // pahole -C net_device | grep name

    let dev_addr_offset = 1080;
    let state_offset = 168;

    let name_pointer = unsafe { (net_device_pointer as *const u8).add(name_field_offset) };
    let dev_addr_pointer = unsafe { (net_device_pointer as *const u8).add(dev_addr_offset) };
    let state_pointer = unsafe { (net_device_pointer as *const u8).add(state_offset) };

    let mut name_buf = [0u8; 16];
    let mut dev_addr_buf = [0u32; 8];

    let name_array_ptr = name_pointer as *const [u8; 16];
    let dev_addr_ptr_array = dev_addr_pointer as *const [u32; 8];

    let name_array = unsafe {
        match bpf_probe_read_kernel(name_array_ptr) {
            Ok(arr) => arr,
            Err(ret) => {
                return Err(ret);
            }
        }
    };

    let state=unsafe {
        match bpf_probe_read_kernel(state_pointer) {
            Ok(s)=>s,
            Err(ret)=>{
                return Err(ret);
            }
        }
    };

    let dev_addr_array = unsafe {
        match bpf_probe_read_kernel(dev_addr_ptr_array) {
            Ok(arr) => arr,
            Err(ret) => {
                return Err(ret);
            }
        }
    };

    name_buf.copy_from_slice(&name_array);
    dev_addr_buf.copy_from_slice(&dev_addr_array);

    let veth_data = VethLog {
        name: name_buf,
        state: state.into(),
        dev_addr: dev_addr_buf,
        event_type: 2
    };

    //send the data to the userspace
    unsafe {
        VETH_CREATION_EVENTS.output(&ctx, &veth_data, 0);
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
    let ihl = (first_ipv4_byte &
        0x0f) as usize; /* 0x0F=00001111 &=AND bit a bit operator to extract the last 4 bit*/
    let ip_header_len = ihl * 4; //returns the header lenght in bytes

    //get the source ip,destination ip and connection id
    let src_ip = u32::from_be(ctx.load::<u32>(SRC_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+SOURCE_ADDRESS
    let src_port = u16::from_be(
        ctx
            .load::<u16>(ETH_STACK_BYTES + ip_header_len + SRC_PORT_OFFSET_FROM_IP_HEADER)
            .map_err(|_| 1)?
    ); //14+IHL-Lenght+0
    let dst_ip = u32::from_be(ctx.load::<u32>(DST_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+ DESTINATION_ADDRESS
    let dst_port = u16::from_be(
        ctx
            .load::<u16>(ETH_STACK_BYTES + ip_header_len + DST_PORT_OFFSET_FROM_IP_HEADER)
            .map_err(|_| 1)?
    ); //14+IHL-Lenght+0
    let proto = u8::from_be(ctx.load::<u8>(PROTOCOL_T0TAL_BYTES_OFFSET).map_err(|_| 1)?);

    //not logging internal communication packets
    //TODO: do not log internal communications such as minikube dashboard packets or kubectl api packets
    //FIXME: this part is not working properly because the ip associated in the k8s environment constantly changes every restart
    let ip_to_block = u32::from_be_bytes([90, 120, 244, 10]); // kubernetes-dashboard internal ip
    let ip_to_block_2 = u32::from_be_bytes([87, 120, 244, 10]); // cert manager internal ip
    let ip_to_block_3 = u32::from_be_bytes([89, 120, 244, 10]); // kube-system internal ip
    let ip_to_block_4 = u32::from_be_bytes([88, 120, 244, 10]); // other kuber-system internal ip

    let key = ConnArray {
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        proto,
    };

    // XOR to generate the hash id for the given connection
    let event_id = (src_ip ^
        dst_ip ^
        (src_port as u32) ^
        (dst_port as u32) ^
        (proto as u32)) as u16; //generate one for every event using a 'byte XOR' operation

    //let connection_id = (src_ip ^ dst_ip ^(proto as u32)) as u16; //added host_id to track the host to count every all the different connections

    if
        src_ip == ip_to_block ||
        src_ip == ip_to_block_2 ||
        src_ip == ip_to_block_3 ||
        src_ip == ip_to_block_4
    {
        return Ok(());
    } else {
        //log all other packets
        let log = PacketLog {
            proto,
            src_ip,
            src_port,
            dst_ip,
            dst_port,
            event_id,
        };
        //let connections = ConnArray{
        //  event_id,
        //connection_id
        //};
        unsafe {
            EVENTS.output(&ctx, &log, 0); //output to userspace
            //TODO: add more parameters to better identify the active connection (maybe timestamp?)
            ACTIVE_CONNECTIONS.insert(&event_id, &key, 0);
        }
    }

    Ok(())
}

//ref:https://elixir.bootlin.com/linux/v6.15.1/source/include/uapi/linux/ethtool.h#L536
//https://elixir.bootlin.com/linux/v6.15.1/source/drivers/net/veth.c#L268
//https://eunomia.dev/tutorials/3-fentry-unlink/

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
    }
}