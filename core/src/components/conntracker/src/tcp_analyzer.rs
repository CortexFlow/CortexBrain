use aya_ebpf::programs::ProbeContext;
use aya_ebpf::helpers::{
    bpf_get_current_comm,
    bpf_get_current_pid_tgid,
    bpf_get_current_cgroup_id,
};

use crate::bindings::{ sk_buff };
use crate::offsets::OFFSETS;
use crate::data_structures::{ PACKET_REGISTRY, TcpPacketRegistry };
use crate::veth_tracer::{ read_linux_inner_struct, read_linux_inner_value };

// docs:
// TODO: add function documentation

// docs:
//
// how skb works? http://oldvger.kernel.org/~davem/skb_data.html
//
// ref: https://elixir.bootlin.com/linux/v6.17.7/source/net/ipv4/tcp_ipv4.c#L2195
//

//in tcp_v4_recv skb->data
pub fn try_tcp_analyzer(ctx: ProbeContext) -> Result<u32, i64> {
    let sk_buff_pointer: *const sk_buff = ctx.arg(0).ok_or(1i64)?;
    // first control: i'm, verifying that the pointer is not null
    if sk_buff_pointer.is_null() {
        return Err(1);
    }

    let skb_data_pointer = read_linux_inner_struct::<u8>(
        sk_buff_pointer as *const u8,
        OFFSETS::SKB_DATA_POINTER
    )?;
    let first_ipv4_byte = read_linux_inner_value::<u8>(skb_data_pointer as *const u8, 0)?;
    let ihl = (first_ipv4_byte & 0x0f) as usize; // 0x0F=00001111 &=AND bit a bit operator to extract the last 4 bit
    let ip_header_len = ihl * 4; //returns the header lenght in bytes

    let proto = read_linux_inner_struct::<u8>(
        skb_data_pointer,
        OFFSETS::IPV4_PROTOCOL_OFFSET
    )? as u8;

    if proto != 6 {
        return Ok(0);
    } else {
        // get the source ip,destination ip and connection id
        let src_ip = read_linux_inner_value::<u32>(skb_data_pointer, OFFSETS::SRC_BYTE_OFFSET)?;
        let dst_ip = read_linux_inner_value::<u32>(skb_data_pointer, OFFSETS::DST_BYTE_OFFSET)?;
        let src_port = u16::from_be(
            read_linux_inner_value(
                skb_data_pointer,
                ip_header_len + OFFSETS::SRC_PORT_OFFSET_FROM_IP_HEADER
            )?
        );
        let dst_port = u16::from_be(
            read_linux_inner_value(
                skb_data_pointer,
                ip_header_len + OFFSETS::DST_PORT_OFFSET_FROM_IP_HEADER
            )?
        );

        let command = bpf_get_current_comm()?;
        let pid = (bpf_get_current_pid_tgid() >> 32) as u32;
        let cgroup_id = unsafe { bpf_get_current_cgroup_id() };

        let log = TcpPacketRegistry {
            proto,
            src_ip,
            dst_ip,
            src_port,
            dst_port,
            pid,
            command,
            cgroup_id,
        };
        unsafe {
            PACKET_REGISTRY.output(&ctx, &log, 0);
        }
    }

    Ok(0)
}
