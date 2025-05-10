#![no_std]

use bytemuck::{Pod, Zeroable};

/// Chiave per la mappa dei servizi
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable,Debug)]
pub struct SVCKey {
    pub service_name: [u8; 64],
}

/// Valore per la mappa dei servizi
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable,Debug)]
pub struct SVCValue {
    pub ip: [u8; 4],
    pub port: u32,
}

/// Converte una stringa in un array di byte di lunghezza 64
pub fn str_to_u8_64(s: &str) -> [u8; 64] {
    let mut buf = [0u8; 64];
    let bytes = s.as_bytes();
    let len = bytes.len().min(64);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}