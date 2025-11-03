/*
 * IpProtocols enum to reconstruct the packet protocol based on the
 * IPV4 Header Protocol code
 */
#[cfg(feature="enums")]
#[derive(Debug)]
#[repr(u8)]
pub enum IpProtocols {
    ICMP = 1,
    TCP = 6,
    UDP = 17,
}