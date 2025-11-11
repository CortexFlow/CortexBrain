// docs:
// TODO: write docs about the traffic control features

use core::net::Ipv4Addr;

use aya_ebpf::{
    helpers::{bpf_get_current_pid_tgid},
    programs::{TcContext},
};
use aya_log_ebpf::info;

use crate::data_structures::{ ConnArray, PacketLog };
use crate::data_structures::{ EVENTS,BLOCKLIST };
use crate::offsets::OFFSETS;

pub fn try_identity_classifier(ctx: TcContext) -> Result<(), i64> {
    let eth_proto = u16::from_be(ctx.load::<u16>(12).map_err(|_| 1)?);

    //only ipv4 protcol allowed
    if eth_proto != OFFSETS::IPV4_ETHERTYPE {
        return Ok(());
    }

    //read if the packets has Options
    let first_ipv4_byte = u8::from_be(ctx.load::<u8>(OFFSETS::ETH_STACK_BYTES).map_err(|_| 1)?);
    let ihl = (first_ipv4_byte &
        0x0f) as usize; /* 0x0F=00001111 &=AND bit a bit operator to extract the last 4 bit*/
    let ip_header_len = ihl * 4; //returns the header lenght in bytes

    //get the source ip,destination ip and connection id
    let src_ip = ctx.load::<u32>(OFFSETS::SRC_T0TAL_BYTES_OFFSET).map_err(|_| 1)?; // ETH+SOURCE_ADDRESS
    let src_port = u16::from_be(
        ctx
            .load::<u16>(
                OFFSETS::ETH_STACK_BYTES + ip_header_len + OFFSETS::SRC_PORT_OFFSET_FROM_IP_HEADER
            )
            .map_err(|_| 1)?
    ); //14+IHL-Lenght+0
    let dst_ip = ctx.load::<u32>(OFFSETS::DST_T0TAL_BYTES_OFFSET).map_err(|_| 1)?; // ETH+ DESTINATION_ADDRESS
    let dst_port = u16::from_be(
        ctx
            .load::<u16>(
                OFFSETS::ETH_STACK_BYTES + ip_header_len + OFFSETS::DST_PORT_OFFSET_FROM_IP_HEADER
            )
            .map_err(|_| 1)?
    ); //14+IHL-Lenght+0
    let proto = u8::from_be(ctx.load::<u8>(OFFSETS::PROTOCOL_T0TAL_BYTES_OFFSET).map_err(|_| 1)?);

    let pid: u32 = bpf_get_current_pid_tgid() as u32;

    // check if the address is in the blocklist
    let src_ip_be_bytes: [u8; 4] = src_ip.to_be_bytes(); //transforming the src_ip in big endian bytes

    // ** blocklist logic
    if unsafe { BLOCKLIST.get(&src_ip_be_bytes).is_some() } {
        info!(
            &ctx,
            "Blocking address: {}. Reason: Address is in a BLOCKLIST",
            Ipv4Addr::from(src_ip_be_bytes)
        );
        return Err(1);
    } else {
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
        unsafe {
            EVENTS.output(&ctx, &log, 0); //output to userspace
        }
    }
    Ok(())
}
