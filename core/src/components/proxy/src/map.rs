/*
    Init a bpf map to save the user space pod data
    to use in the kernel space and user space
*/

use aya::Pod;
use aya_ebpf::{macros::map, maps::HashMap as KernelSpaceMap}; //aya_ebpf is the kernel space libary
use bytemuck::Zeroable;

unsafe impl Zeroable for SVCKey {} //implemente zeroable 
unsafe impl Zeroable for SVCValue {}

#[repr(C)]
/*
    match the C fields alignment. tells the compiler that the rappresentation
    must follow the C rules. disable the rust compiler realignment

    In this case Rust struct and C are unitary equivalent

*/
#[derive(Clone, Copy)]
//struct to
pub struct SVCKey {
    pub port: u32,
}

#[repr(C)] //match the C fields alignment
#[derive(Clone, Copy)]
pub struct SVCValue {
    pub ip: u32,
    pub port: u32,
}

//enable Pod (Plain of data) data type
unsafe impl Pod for SVCKey {}
unsafe impl Pod for SVCValue {}
/*

    Doc:
    POD (Plain Old Data) types are marked with the trait, indicating that they can be
    duplicated simply by copying their memory representation.

    This trait allows the Rust compiler to efficiently handle data creating bit-for-bit copies
    without invoking user-defined methods
    POD types do not involve pointers or complex data structures, they are easier to manage in terms of
    memory allocation and deallocation

*/

#[map(name = "services")] //connect the map name "SERVICES" to the HasMap in the BPF bytecode
//init a BPF_MAP_HASH_TYPE to store the resolved service values
pub static mut SERVICES: KernelSpaceMap<SVCKey, SVCValue> =
    KernelSpaceMap::with_max_entries(1024, 0);
