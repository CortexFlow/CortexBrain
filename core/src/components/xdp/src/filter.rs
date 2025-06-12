/*
 * Contains the code for the kernel xdp manipulation. this code lives in
 * the kernel space only and needs to be attached to a "main" program that lives in the user space
*/

#![no_std] // * no standard library
#![no_main] // * no main entrypoint

use aya_ebpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_log_ebpf::{debug, error, info};

use core::mem;
use maps::map::{SVCKey, SVCValue, SERVICES};
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    tcp::TcpHdr,
    udp::UdpHdr,
};

/*
* init xdp program
*/
#[xdp]
pub fn xdp_hello(ctx: XdpContext) -> u32 {
    match unsafe { xdp_firewall(&ctx) } {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

unsafe fn init_xdp(ctx: &XdpContext) -> Result<u32, u32> {
    info!(ctx, "Received a packet");
    Ok(xdp_action::XDP_PASS)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// * getting packet data from raw packets
#[inline(always)] //inline
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }
    Ok((start + offset) as *const T)
}

//TODO:safe the result of the firewall into a bpf hash map and perform a redirect
/*
* XDP firewall
* Usage:
*   1. Drop packets from the 443 port (only fo development test )
*   2. Log TCP and UDP traffic
*   3. //TODO: rebuild firewall policy to efficiently filter traffic
*   4. //TODO: use ConnArray declared in conntracker program to discover services and implement policies
*/
fn xdp_firewall(ctx: &XdpContext) -> Result<u32, ()> {
    let ethhdr: *const EthHdr = ptr_at(ctx, 0)?;
    match unsafe { (*ethhdr).ether_type } {
        EtherType::Ipv4 => {}
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = ptr_at(ctx, EthHdr::LEN)?;
    let source_addr = u32::from_be_bytes(unsafe { (*ipv4hdr).src_addr });

    // handle protocols
    match unsafe { (*ipv4hdr).proto } {
        IpProto::Tcp => {
            let tcphdr: *const TcpHdr = ptr_at(ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            let port = u16::from_be(unsafe { (*tcphdr).source });
            if port == 443 {
                return Ok(xdp_action::XDP_PASS);
            } else {
                info!(
                    ctx,
                    "Received TCP packet from IP: {:i} PORT: {}", source_addr, port
                );
            }
        }
        IpProto::Udp => {
            let udphdr: *const UdpHdr = ptr_at(ctx, EthHdr::LEN + Ipv4Hdr::LEN)?;
            let port = u16::from_be_bytes(unsafe { (*udphdr).source });
            if port == 443 {
                return Ok(xdp_action::XDP_PASS);
            } else {
                info!(
                    ctx,
                    "Received UDP packet from IP: {:i} PORT: {}", source_addr, port
                );
                //TODO: saving the packet address here will only store the ip and port
                // of the gateway (in this case the minikube node ip)--> i need to save the ip in the user space
                debug!(
                    ctx,
                    "Inserting key: {:i} and value {:i} into the services bpf map",
                    source_addr,
                    port as u32
                );
                let key = SVCKey {
                    service_name: maps::map::u32_to_u8_64(source_addr),
                };
                let value = SVCValue {
                    ip: maps::map::u32_to_u8_4(source_addr.into()),
                    port: port as u32,
                };
                let res = unsafe { SERVICES.insert(&key, &value, 0) };
                match res {
                    Ok(_) => {
                        return Ok(xdp_action::XDP_PASS);
                    }
                    Err(_) => {
                        error!(ctx, "Error inserting element into bpf map");
                        return Err(());
                    }
                }
            }
        }
        _ => return Ok(xdp_action::XDP_DROP),
    };

    Ok(xdp_action::XDP_PASS)
}
