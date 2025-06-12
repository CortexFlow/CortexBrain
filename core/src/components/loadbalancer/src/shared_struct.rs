#![no_std]

use bytemuck::{Pod, Zeroable};

//ref: /maps/map.rs/SVCKey
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable,Debug)]
pub struct SVCKey {
    pub service_name: [u8; 64],
}

//ref: /maps/map.rs/SVCValue
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable,Debug)]
pub struct SVCValue {
    pub ip: [u8; 4],
    pub port: u32,
}
//ref: /map/maps.rs/BackendPorts
#[repr(C)]
#[derive(Clone,Debug,Pod,Zeroable,Copy)]
pub struct BackendPorts{
    pub ports: [u16;4],
    pub index: usize
}

//ref: /map/map.rs/str_to_u8_64
pub fn str_to_u8_64(s: &str) -> [u8; 64] {
    let mut buf = [0u8; 64];
    let bytes = s.as_bytes();
    let len = bytes.len().min(64);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}
pub fn str_to_u8_4(s: &str) -> [u8; 4] {
    let mut buf = [0u8; 4];
    let bytes = s.as_bytes();
    let len = bytes.len().min(4);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}