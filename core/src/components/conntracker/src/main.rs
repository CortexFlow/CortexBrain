/* 
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

use bytemuck::{Pod,Zeroable};
use aya_ebpf::{
    bindings::{TC_ACT_OK,TC_ACT_SHOT},
    macros::{ classifier, map },
    maps::PerfEventArray,
    maps::LruPerCpuHashMap,
    programs::TcContext,
};
use aya_log_ebpf::info;
use core::mem;


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
    * https://datatracker.ietf.org/doc/html/rfc791
    *
    *
    * Ipv4 header datagram

    0                   1                   2                   3                       TOT BYTES OFFSET (full length)     
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1      32 bit           
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |Version|  IHL  |Type of Service|          Total Length         |     4 bytes            4             
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |         Identification        |Flags|      Fragment Offset    |     4 bytes            8            
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+    
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
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ConnArray{
    pub event_id: u16,
    pub connection_id: u16
}

#[map(name="EventsMap")]
static mut EVENTS: PerfEventArray<PacketLog> = PerfEventArray::new(0);
#[map(name = "ConnectionMap")]
pub static mut CONNMAP: LruPerCpuHashMap<u8,ConnArray> = LruPerCpuHashMap::with_max_entries(1024,0); //TODO: modify this to a LRU HASHMAP

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


static ETH_STACK_BYTES :usize = SRC_MAC+DST_MAC+ETHERTYPE_BYTES;
static DST_T0TAL_BYTES_OFFSET :usize = ETH_STACK_BYTES + DST_BYTE_OFFSET;
static SRC_T0TAL_BYTES_OFFSET :usize = ETH_STACK_BYTES + SRC_BYTE_OFFSET;
static PROTOCOL_T0TAL_BYTES_OFFSET :usize = ETH_STACK_BYTES + IPV4_PROTOCOL_OFFSET;




#[classifier]
pub fn identity_classifier(ctx: TcContext) -> i32 {
    match try_identity_classifier(ctx) {
        Ok(_) => TC_ACT_OK,
        Err(_) => TC_ACT_SHOT,//block packets that returns errors
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
    let ihl = (first_ipv4_byte & 0x0F) as usize; /* 0x0F=00001111 &=AND bit a bit operator to extract the last 4 bit*/
    let ip_header_len= ihl*4; //returns the header lenght in bytes

    //get the source ip,destination ip and connection id
    let src_ip = u32::from_be(ctx.load::<u32>(SRC_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+SOURCE_ADDRESS
    let src_port = u16::from_be(ctx.load::<u16>(ETH_STACK_BYTES+ip_header_len+SRC_PORT_OFFSET_FROM_IP_HEADER).map_err(|_| 1)?); //14+IHL-Lenght+0
    let dst_ip = u32::from_be(ctx.load::<u32>(DST_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+ DESTINATION_ADDRESS
    let dst_port = u16::from_be(ctx.load::<u16>(ETH_STACK_BYTES+ip_header_len+DST_PORT_OFFSET_FROM_IP_HEADER).map_err(|_| 1)?); //14+IHL-Lenght+0
    let proto = u8::from_be(ctx.load::<u8>(PROTOCOL_T0TAL_BYTES_OFFSET).map_err(|_| 1)?);
    

    //not logging internal communication packets
    //TODO: do not log internal communications such as minikube dashboard packets or kubectl api packets
    let ip_to_block = u32::from_be_bytes([192,168,49,1]); //inverted requence
    let dst_ip_to_block = u32::from_be_bytes([192,168,49,2]);    
    
    
    // XOR to generate the hash id for the given connection
    let event_id = (src_ip ^ dst_ip ^ (src_port as u32) ^ (dst_port as u32) ^ (proto as u32)) as u16; //generate one for every event using a 'byte XOR' operation 

    let connection_id = (src_ip ^ dst_ip ^(proto as u32)) as u16; //added host_id to track the host to count every all the different connections

    if src_ip == ip_to_block && dst_ip == dst_ip_to_block {
        return Ok(());
    }
    else{
        //log all other packets
        let log = PacketLog {
            proto,
            src_ip,
            src_port,
            dst_ip,
            dst_port,
            event_id,
        };
        let connections = ConnArray{
            event_id,
            connection_id
        };
        unsafe {
            EVENTS.output(&ctx, &log, 0); //output to userspace
            //TODO: add more parameters to better identify the active connection (maybe timestamp?)
            CONNMAP.insert(&proto,&connections, 0) //save hash_id to kernel space lru per cpu hashmap
        };
    }
        
    Ok(())
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
    }
}
