/* 
    *
    * This file contains the code for the identity service  
    *
    
    * Functionalities:
    *   1. Creates a PacketLog structure to track incoming packets
    *   2. Tracking Parameters: SRC_IP.SRC_PORT,DST_IP,DST_PORT,PROTOCOL,HASH
    *   3. Store HASH_ID in a BPF HASHMAP
*/

// Imports
#![no_std]
#![no_main]

use bytemuck::{Pod,Zeroable};
use aya_ebpf::{
    bindings::TC_ACT_OK,
    macros::{ classifier, map },
    maps::PerfEventArray,
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
    pub src_port: u32,
    pub dst_ip: u32,
    pub dst_port: u32,
    pub hash_id: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ConnArray{
    pub hash_id: u16
}

#[map]
static mut EVENTS: PerfEventArray<PacketLog> = PerfEventArray::new(0);
#[map(name = "ConnectionArray")]
pub static mut CONNARRAY: PerfEventArray<ConnArray> = PerfEventArray::new(0);

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
const SRC_PORT: usize = 34;
const DST_PORT: usize = 36;

static ETH_STACK_BYTES :usize = SRC_MAC+DST_MAC+ETHERTYPE_BYTES;
static DST_T0TAL_BYTES_OFFSET :usize = ETH_STACK_BYTES + DST_BYTE_OFFSET;
static SRC_T0TAL_BYTES_OFFSET :usize = ETH_STACK_BYTES + SRC_BYTE_OFFSET;
static PROTOCOL_T0TAL_BYTES_OFFSET :usize = ETH_STACK_BYTES + IPV4_PROTOCOL_OFFSET;




#[classifier]
pub fn identity_classifier(ctx: TcContext) -> i32 {
    match try_identity_classifier(ctx) {
        Ok(_) => TC_ACT_OK,
        Err(_) => TC_ACT_OK,
    }
}

fn try_identity_classifier(ctx: TcContext) -> Result<(), i64> {
    let eth_proto = u16::from_be(ctx.load::<u16>(12).map_err(|_| 1)?);

    //only ipv4 protcol allowed
    if eth_proto != IPV4_ETHERTYPE {
        return Ok(());
    } else {

        //get the source ip,destination ip and connection id
        let src_ip = u32::from_be(ctx.load::<u32>(SRC_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+SOURCE_ADDRESS
        let src_port = u32::from_be(ctx.load::<u32>(SRC_PORT).map_err(|_| 1)?);
        let dst_ip = u32::from_be(ctx.load::<u32>(DST_T0TAL_BYTES_OFFSET).map_err(|_| 1)?); // ETH+ DESTINATION_ADDRESS
        let dst_port = u32::from_be(ctx.load::<u32>(DST_PORT).map_err(|_| 1)?);
        let proto = u8::from_be(ctx.load::<u8>(PROTOCOL_T0TAL_BYTES_OFFSET).map_err(|_| 1)?);
        
        // XOR to generate the hash id for the given connection
        let hash_id = (src_ip ^ dst_ip ^ (src_port as u32) ^ (dst_port as u32) ^ (proto as u32)) as u16;

        let log = PacketLog {
            proto,
            src_ip,
            src_port,
            dst_ip,
            dst_port,
            hash_id,
        };
        let connections = ConnArray{
            hash_id
        };
        unsafe {
            EVENTS.output(&ctx, &log, 0); //output to userspace
            CONNARRAY.output(&ctx,&connections, 0) //save hash_id to kernel space array
        };
    }

    Ok(())
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
    }
}
