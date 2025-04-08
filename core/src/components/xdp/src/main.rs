/* contains the code for the kernel xdp manipulation. this code lives in
the kernel space only and needs to be attached to a program in the user space
*/
#![no_std] //no standard library
#![no_main] //no main entrypoint

use aya_ebpf::{bindings::xdp_action, macros::xdp, programs::XdpContext};
use aya_log_ebpf::info;

#[xdp]
//simple hello world to test
pub fn xdp_hello(ctx: XdpContext) -> u32 {
    match unsafe { init_xdp(ctx) } {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

//xdp init
unsafe fn init_xdp(ctx: XdpContext) -> Result<u32, u32> {
    info!(&ctx, "Received a packet");
    Ok(xdp_action::XDP_PASS)
}

#[panic_handler] //panic handler in case of fatal errors
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
