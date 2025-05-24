/*
 * Init a bpf map to save the user space pod data
 * to use in the kernel space and user space
*/
#![no_std]
#![no_main]

use aya_ebpf::maps::PerfEventArray;
use aya_ebpf::{macros::map, maps::HashMap as KernelSpaceMap}; //aya_ebpf is the kernel space libary
use bytemuck::{Pod, Zeroable};

/* unsafe impl Zeroable for SVCKey {} //implemente zeroable
unsafe impl Zeroable for SVCValue {} */

#[repr(C)]
/*
 * match the C fields alignment. tells the compiler that the rappresentation
 * must follow the C rules. disable the rust compiler realignment
 *
 *In this case Rust struct and C are unitary equivalent
*/
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct SVCKey {
    pub service_name: [u8; 64],
}

#[repr(C)] //match the C fields alignment
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct SVCValue {
    pub ip: [u8; 4],
    pub port: u32,
}

#[repr(C)]
#[derive(Clone, Debug, Pod, Zeroable, Copy)]
pub struct BackendPorts {
    pub ports: [u16; 4],
    pub index: usize,
}

//enable Pod (Plain of data) data type
/* unsafe impl Pod for SVCKey {}
unsafe impl Pod for SVCValue {} */
/*
    * Doc:
    * POD (Plain Old Data) types are marked with the trait, indicating that they can be
    * duplicated simply by copying their memory representation.
    *
    * This trait allows the Rust compiler to efficiently handle data creating bit-for-bit copies
    * without invoking user-defined methods
    * POD types do not involve pointers or complex data structures, they are easier to manage in terms of
    * memory allocation and deallocation

*/

/*
 * Maps
*/

#[map(name = "services")]
/*
 * connect the map name "SERVICES" to the HasMap in the BPF bytecode
 * init a BPF_MAP_HASH_TYPE to store the resolved service values
*/
pub static mut SERVICES: KernelSpaceMap<SVCKey, SVCValue> =
    KernelSpaceMap::with_max_entries(1024, 0);

#[map(name = "Backend_ports")]
/*
 * connect the map name "BACKEND_PORTS" to the HasMap in the BPF bytecode
 * init a BPF_MAP_HASH_TYPE to store the resolved service values
*/
pub static mut BACKEND_PORTS: KernelSpaceMap<u16, BackendPorts> =
    KernelSpaceMap::with_max_entries(10, 0);

/*Aux Functions */

//perform &str types to &[u8;64]
pub fn str_to_u8_64(s: &str) -> [u8; 64] {
    let mut buf = [0u8; 64];
    let bytes = s.as_bytes();
    let len = bytes.len().min(64);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}
pub fn u32_to_u8_4(s: u32) -> [u8; 4] {
    //32 bit ---> 4 bytes
    let mut buf = [0u8; 4];
    let bytes = s.to_le_bytes(); // this produce [u8,4]
    buf[..4].copy_from_slice(&bytes); // Only copy the first 4 bytes
    buf
}

pub fn u32_to_u8_64(s: u32) -> [u8; 64] {
    //32 bit ---> 4 bytes
    let mut buf = [0u8; 64];
    let bytes = s.to_le_bytes(); //this produce [u8,4]
    buf[..4].copy_from_slice(&bytes); // Only copy the first 4 bytes
    buf
}
