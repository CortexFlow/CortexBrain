use lazy_static::lazy_static;
use prometheus::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};


lazy_static!{
    /* One of the main benefits of using lazy_static is the ability to store thread-safe global variables. 
    Because lazy static values are initialized in a thread-safe manner, they can be safely accessed 
    from multiple threads without the need for additional synchronization. 
    This can be especially useful in cases where you want to avoid the overhead of locking and unlocking shared resources. 

    lazy static documentation: https://blog.logrocket.com/rust-lazy-static-pattern/#how-lazy-static-works
    */

    pub static ref DNS_REQUEST: IntCounterVec = register_int_counter_vec!(
        "total_dns_requests",
        "Total_DNS_Requests",
        &["client_ip"]
    ).unwrap();

    pub  static ref DNS_RESPONSE_TIME: HistogramVec = register_histogram_vec!(
        "dns_response_time",
        "DNS_response_time",
        &["server"]
    ).unwrap();
}
