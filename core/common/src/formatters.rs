use std::net::Ipv4Addr;

pub fn format_ipv4(ip: u32) -> String {
    Ipv4Addr::from(u32::from_be(ip)).to_string()
}

pub fn format_ipv6(ip: &[u32; 4]) -> String {
    format!(
        "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
        (ip[0] >> 16) & 0xFFFF, ip[0] & 0xFFFF,
        (ip[1] >> 16) & 0xFFFF, ip[1] & 0xFFFF,
        (ip[2] >> 16) & 0xFFFF, ip[2] & 0xFFFF,
        (ip[3] >> 16) & 0xFFFF, ip[3] & 0xFFFF
    )
}
