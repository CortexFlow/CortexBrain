use aya_ebpf::helpers::bpf_probe_read_kernel;
use aya_ebpf::{helpers::bpf_get_current_pid_tgid, programs::ProbeContext};

use crate::bindings::net;
use crate::bindings::net_device;
use crate::data_structures::VethLog;
use crate::data_structures::VETH_EVENTS;

// docs:
//
// This is the main function to trace the creation or teh deletion of a virtual ethernet interface
// Takes a ProbeContext and a mode to call an action (creation/deletion)
// mode selection:
//      - 1 -> veth_creation_tracer
//      - 2 -> veth_deletion_tracer
//
// Returns a Result type with the value as u32 or an error code as i64

pub fn try_veth_tracer(ctx: ProbeContext, mode: u8) -> Result<u32, i64> {
    let net_device_pointer: *const net_device = ctx.arg(0).ok_or(1i64)?;

    // first control: i'm, verifying that the pointer is not null
    if net_device_pointer.is_null() {
        return Err(1);
    }

    let mut name_buf = [0u8; 16];
    let mut dev_addr_buf = [0u32; 8];

    // name field
    let name_field_offset = 304; // reading the name field offset

    let name_array: [u8; 16] =
        read_linux_inner_value::<[u8; 16]>(net_device_pointer as *const u8, name_field_offset)?;

    // state field
    let state_offset = 168;
    let state: u8 = read_linux_inner_value::<u8>(net_device_pointer as *const u8, state_offset)?;

    // dev_addr
    let dev_addr_offset = 1080;
    let dev_addr_array: [u32; 8] =
        read_linux_inner_value::<[u32; 8]>(net_device_pointer as *const u8, dev_addr_offset)?;

    let inum: u32 = extract_netns_inum(net_device_pointer as *const u8)?;
    let pid: u32 = bpf_get_current_pid_tgid() as u32; // extracting lower 32 bit corresponding to the PID

    // buffer copying for array types
    name_buf.copy_from_slice(&name_array);
    dev_addr_buf.copy_from_slice(&dev_addr_array);

    // compose the structure
    let veth_data = VethLog {
        name: name_buf,
        state: state.into(),
        dev_addr: dev_addr_buf,
        event_type: mode,
        netns: inum,
        pid,
    };

    // send the data to the userspace
    unsafe {
        VETH_EVENTS.output(&ctx, &veth_data, 0);
    }

    Ok(0)
}

// docs:
//
// This is an helper function to read an inner structure in the kernel
// Takes a pointer to the kernel structure and an offset
//
// ex: 
//      - let net = read_linux_inner_struct::<net>(net_device_pointer, possible_net_t_offset)?;
//
// Returns a Result type with a const pointer to an inner field or an error code as i64

pub fn read_linux_inner_struct<T>(ptr: *const u8, offset: usize) -> Result<*const T, i64> {
    if ptr.is_null() {
        return Err(1);
    } else {
        let inner_ptr = unsafe { (ptr as *const u8).add(offset) };

        let inner_field: *const T = unsafe {
            match bpf_probe_read_kernel(inner_ptr as *const *const T) {
                Ok(inner_field) => inner_field,
                Err(e) => {
                    return Err(e);
                }
            }
        };
        Ok(inner_field)
    }
}

// docs:
//
// This is an helper function to read an value from a kernel structure
// Takes a pointer to the kernel structure and an offset
//
// ex:
//      - let inum_ptr = read_linux_inner_value::<u32>(net as *const u8, ns_common_offset + inum_offset)?;
//
// Returns a Result type with the value or an error code as i64

pub fn read_linux_inner_value<T: Copy>(ptr: *const u8, offset: usize) -> Result<T, i64> {
    if ptr.is_null() {
        return Err(1);
    }

    let inner_ptr = unsafe { (ptr as *const u8).add(offset) };

    let inner_value = unsafe {
        match bpf_probe_read_kernel::<T>(inner_ptr as *const T) {
            Ok(inner_field) => inner_field,
            Err(e) => {
                return Err(e);
            }
        }
    };

    Ok(inner_value)
}

// docs:
//
// This is an helper function to read the network namespace inum (nets inum ) from a net_device pointer
// Takes a pointer to the net_device kernel structure
//
// Returns a Result type with the value as u32 or an error code as i64

fn extract_netns_inum(net_device_pointer: *const u8) -> Result<u32, i64> {
    let possible_net_t_offset = 280;

    let net = read_linux_inner_struct::<net>(net_device_pointer, possible_net_t_offset)?;

    let ns_common_offset = 120;

    let inum_offset = 16;
    let inum_ptr = read_linux_inner_value::<u32>(net as *const u8, ns_common_offset + inum_offset)?;
    Ok(inum_ptr)
}
