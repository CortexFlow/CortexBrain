/*
    UTILITIES: support functions used in the EdgeDNS crate

*/
#[warn(unused_imports)]
use std::net::Ipv4Addr;
use itertools::Itertools;

fn is_valid_ip(ip: &str) -> bool {
    /* check if an ip address is valid or not*/
    ip.parse::<Ipv4Addr>().is_ok()
}
fn is_valid_port(port: &str) -> bool {
    /* 
        Workflow:
        - convert the port from string to i32 integer
        - check if the port is between 0 and 1 
            - OK: return true
            - Err: return false + error status
     */
    let port_enum = port.parse::<i32>().unwrap();
    if 0 < port_enum && port_enum < 65536 {
        return true;
    }
    else {
        return false;
    }
}
pub fn remove_duplicates(ss: Vec<String>)->Vec<String> {
    /* 
    Workflow: 
    - into_iter():
            Turns ss into an iterator of owned elements.
    - unique():
            Filters out duplicates while maintaining the original order.
    - collect():
            Collects the filtered elements into a new container. 
    
    */
    ss.into_iter().unique().collect()
}
