//! Perf-buffer consumers for eBPF events.
//!
//! This module provides the [`Consumer`] enum and its associated `read` methods
//! that parse raw bytes from [`aya::maps::perf::PerfEventArrayBuffer`] into
//! strongly-typed eBPF structs and forward them to the OpenTelemetry metrics
//! pipeline via [`crate::otel_metrics::Metrics`].
//!
//! Each consumer method:
//! 1. Validates the raw byte buffer length against the expected struct size.
//! 2. Performs an unaligned read into the `#[repr(C, packed)]` struct.
//! 3. Builds [`crate::metadata::Metadata`] (with optional Docker/K8s enrichment).
//! 4. Records the observation through [`Metrics::record_*`].

#[cfg(feature = "monitoring-structs")]
use crate::buffer_type::{
    CpuFrequency, CpuIdle, MemAlloc, PacketLossMetrics, SchedStatRuntime, SchedStatWait,
    TimeStampMetrics,
};
#[cfg(feature = "network-structs")]
use crate::buffer_type::{PacketLog, TcpPacketRegistry, VethLog};
#[cfg(feature = "monitoring-structs")]
use crate::metadata::Metadata;
#[cfg(feature = "monitoring-structs")]
use crate::otel_metrics::Metrics;
use crate::service_cache::ServiceCache;
use bytes::BytesMut;
#[cfg(feature = "monitoring-structs")]
use std::sync::Arc;
#[cfg(feature = "buffer-reader")]
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Discriminator for perf-buffer event types consumed by the collector.
///
/// Each variant maps to an eBPF program output struct and a dedicated
/// `read_*` method that knows how to parse it.
#[cfg(feature = "buffer-reader")]
pub enum Consumer {
    #[cfg(feature = "network-structs")]
    PacketLog,
    #[cfg(feature = "network-structs")]
    TcpPacketRegistry,
    #[cfg(feature = "network-structs")]
    VethLog,
    #[cfg(feature = "monitoring-structs")]
    PacketLossMetrics,
    #[cfg(feature = "monitoring-structs")]
    TimeStampMetrics,
    #[cfg(feature = "monitoring-structs")]
    CpuFrequency,
    #[cfg(feature = "monitoring-structs")]
    MemAlloc,
    #[cfg(feature = "monitoring-structs")]
    SchedStatWait,
    #[cfg(feature = "monitoring-structs")]
    SchedStatRuntime,
    #[cfg(feature = "monitoring-structs")]
    CpuIdle,
    #[cfg(feature = "monitoring-structs")]
    SslEvents,
}

#[cfg(feature = "buffer-reader")]
impl Consumer {
    /// Read and log [`PacketLog`] events from the perf buffer.
    ///
    /// Parses IPv4 addresses, ports and L4 protocol from raw eBPF bytes and
    /// emits human-readable `tracing::info!` lines.
    #[cfg(feature = "network-structs")]
    pub async fn read_packet_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        use crate::buffer_type::{IpProtocols, reverse_be_addr};

        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<PacketLog>() {
                error!(
                    "Corrupted Packet log data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<PacketLog>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<PacketLog>() {
                let pl: PacketLog =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                let src_ip = reverse_be_addr(pl.src_ip);
                let dst_ip = reverse_be_addr(pl.dst_ip);
                let src_port = u16::from_be(pl.src_port);
                let dst_port = u16::from_be(pl.dst_port);
                let event_id = pl.pid;
                let protocol = pl.proto;

                match IpProtocols::try_from(protocol) {
                    Ok(proto) => {
                        info!(
                            "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{}",
                            event_id, proto, src_ip, src_port, dst_ip, dst_port
                        );
                    }
                    Err(e) => {
                        error!("Unknown protocol. Data maybe corrupted. Reason:{:?}", e);
                    }
                }
            }
        }
    }

    /// Read and log [`TcpPacketRegistry`] events from the perf buffer.
    ///
    /// Similar to [`read_packet_log`] but additionally prints the command name
    /// and cgroup ID extracted from the eBPF struct.
    #[cfg(feature = "network-structs")]
    pub async fn read_tcp_registry_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        use crate::buffer_type::{IpProtocols, reverse_be_addr};

        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<TcpPacketRegistry>() {
                error!(
                    "Corrupted data Tcp Registry data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<TcpPacketRegistry>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<TcpPacketRegistry>() {
                let pl: TcpPacketRegistry =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                let src = reverse_be_addr(pl.src_ip);
                let dst = reverse_be_addr(pl.dst_ip);
                let src_port = u16::from_be(pl.src_port);
                let dst_port = u16::from_be(pl.dst_port);
                let event_id = pl.pid;
                let command = pl.command.to_vec();
                let end = command
                    .iter()
                    .position(|&x| x == 0)
                    .unwrap_or(command.len());
                let command_str = String::from_utf8_lossy(&command[..end]).to_string();
                let cgroup_id = pl.cgroup_id;
                let protocol = pl.proto;

                match IpProtocols::try_from(protocol) {
                    Ok(proto) => {
                        info!(
                            "Event Id: {} Protocol: {:?} SRC: {}:{} -> DST: {}:{} Command: {} Cgroup_id: {}",
                            event_id, proto, src, src_port, dst, dst_port, command_str, cgroup_id
                        );
                    }
                    Err(e) => {
                        error!("Unknown protocol. Data maybe corrupted. Reason:{:?}", e);
                    }
                }
            }
        }
    }

    /// Read and log [`VethLog`] events from the perf buffer.
    ///
    /// Distinguishes between veth interface creation (event_type == 1) and
    /// deletion (event_type == 2) and logs the interface name, MAC address and
    /// netns.
    #[cfg(feature = "network-structs")]
    pub async fn read_and_handle_veth_log(buffers: &mut [BytesMut], tot_events: i32, offset: i32) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<VethLog>() {
                error!(
                    "Corrupted data VethLog data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<VethLog>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<VethLog>() {
                let vthl: VethLog =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                let name_bytes = vthl.name;
                let dev_addr_bytes = vthl.dev_addr;
                let name = std::str::from_utf8(&name_bytes);
                let state = vthl.state;
                let dev_addr = dev_addr_bytes;
                let netns = vthl.netns;
                let mut event_type = String::new();

                match vthl.event_type {
                    1 => {
                        event_type = "creation".to_string();
                        match name {
                            Ok(veth_name) => {
                                info!(
                                    "[{}] Veth Event: Type: {} Name: {} Dev_addr: {:x?} State: {}",
                                    netns,
                                    event_type,
                                    veth_name.trim_end_matches("\0"),
                                    dev_addr,
                                    state
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to extract veth name during event_type = creation (1).Reason:{}",
                                    e
                                );
                            }
                        }
                    }
                    2 => {
                        event_type = "deletion".to_string();
                        match name {
                            Ok(veth_name) => {
                                info!(
                                    "[{}] Veth Event: Type: {} Name: {} Dev_addr: {:x?} State: {}",
                                    netns,
                                    event_type,
                                    veth_name.trim_end_matches("\0"),
                                    dev_addr,
                                    state
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to extract veth name during event_type = deletion (2).Reason:{}",
                                    e
                                );
                            }
                        }
                    }
                    _ => {
                        warn!("Unknown event type")
                    }
                }
            }
        }
    }

    /// Read [`PacketLossMetrics`] events and record OpenTelemetry observations.
    ///
    /// # Arguments
    /// - `buffers` — raw byte buffers populated by `PerfEventArrayBuffer::read_events`.
    /// - `tot_events` — number of events to process.
    /// - `offset` — start index in `buffers`.
    /// - `exporter` — `"otlp"` forwards to [`Metrics`], any other value is skipped.
    /// - `metrics` — shared [`Metrics`] handle.
    ///
    /// # Safety
    /// Uses `std::ptr::read_unaligned` on `#[repr(C, packed)]` structs that implement [`aya::Pod`].
    ///
    /// # Metadata enrichment
    /// If `exporter == "otlp"`, constructs [`Metadata`] and calls [`Metadata::enrich()`] to resolve
    /// Docker container name from `/proc/<tgid>/cgroup` when available.
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_packet_loss_metrics(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<PacketLossMetrics>() {
                error!(
                    "Corrupted Network Metrics data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<PacketLossMetrics>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<PacketLossMetrics>() {
                let packet_loss: PacketLossMetrics =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                match exporter {
                    "otlp" => {
                        let mut metadata = Metadata::from_ebpf(
                            Some(packet_loss.tgid),
                            Some(packet_loss.cgroup_id),
                            &packet_loss.comm,
                        );
                        metadata.enrich(&cache).await;
                        metrics.record_packet_loss_metrics(&packet_loss, &metadata);
                    }
                    _ => continue,
                }

                let tgid = packet_loss.tgid;
                let comm = String::from_utf8_lossy(&packet_loss.comm);
                let ts_us = packet_loss.ts_us;
                let sk_drop_count = packet_loss.sk_drops;
                let sk_err = packet_loss.sk_err;
                let sk_err_soft = packet_loss.sk_err_soft;
                let sk_backlog_len = packet_loss.sk_backlog_len;
                let sk_write_memory_queued = packet_loss.sk_write_memory_queued;
                let sk_ack_backlog = packet_loss.sk_ack_backlog;
                let sk_receive_buffer_size = packet_loss.sk_receive_buffer_size;

                info!(
                    "tgid: {}, comm: {}, ts_us: {}, sk_drops: {}, sk_err: {}, sk_err_soft: {}, sk_backlog_len: {}, 
                    sk_write_memory_queued: {}, sk_ack_backlog: {}, sk_receive_buffer_size: {}",
                    tgid,
                    comm,
                    ts_us,
                    sk_drop_count,
                    sk_err,
                    sk_err_soft,
                    sk_backlog_len,
                    sk_write_memory_queued,
                    sk_ack_backlog,
                    sk_receive_buffer_size,
                );
            }
        }
    }

    /// Read [`TimeStampMetrics`] events and record OpenTelemetry observations.
    ///
    /// Counterpart to [`read_packet_loss_metrics`] for the `time_stamp_events` map.
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_timestamp_metrics(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<TimeStampMetrics>() {
                error!(
                    "Corrupted Network Metrics data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<TimeStampMetrics>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<TimeStampMetrics>() {
                let time_stamp_event: TimeStampMetrics =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                match exporter {
                    "otlp" => {
                        let mut metadata = Metadata::from_ebpf(
                            Some(time_stamp_event.tgid),
                            Some(time_stamp_event.cgroup_id),
                            &time_stamp_event.comm,
                        );
                        metadata.enrich(&cache).await;
                        metrics.record_timestamp_metrics(&time_stamp_event, &metadata);
                    }
                    _ => continue,
                }

                let delta_us = time_stamp_event.delta_us;
                let ts_us = time_stamp_event.ts_us;
                let tgid = time_stamp_event.tgid;
                let comm = String::from_utf8_lossy(&time_stamp_event.comm);
                let lport = time_stamp_event.lport;
                let dport_be = time_stamp_event.dport_be;
                let af = time_stamp_event.af;
                info!(
                    "TimeStampEvent - delta_us: {}, ts_us: {}, tgid: {}, comm: {}, lport: {}, dport_be: {}, af: {}",
                    delta_us, ts_us, tgid, comm, lport, dport_be, af
                );
            }
        }
    }

    /// Read [`CpuFrequency`] events and record OpenTelemetry observations.
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_cpu_frequency(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<CpuFrequency>() {
                error!(
                    "Corrupted Cpu Frequency Metrics data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<CpuFrequency>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<CpuFrequency>() {
                let cpu_freq_metrics: CpuFrequency =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                match exporter {
                    "otlp" => {
                        let mut metadata = Metadata::from_ebpf(
                            Some(cpu_freq_metrics.pid),
                            None,
                            &cpu_freq_metrics.command,
                        );
                        metadata.enrich(&cache).await;
                        metrics.record_cpu_bytes_alloc(&cpu_freq_metrics, &metadata);
                    }
                    _ => continue,
                }

                let bytes_alloc = cpu_freq_metrics.bytes_alloc;
                let pid = cpu_freq_metrics.pid;
                let command = cpu_freq_metrics.command;
                info!(
                    "Cpu Bytes alloc: {} pid : {} command: {:?}",
                    bytes_alloc, pid, command
                );
            }
        }
    }

    /// Read [`MemAlloc`] events and record OpenTelemetry observations.
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_mem_alloc(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<MemAlloc>() {
                error!(
                    "Corrupted MemAlloc data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<MemAlloc>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<MemAlloc>() {
                let mem_alloc: MemAlloc =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                match exporter {
                    "otlp" => {
                        let mut metadata = Metadata::from_ebpf(
                            Some(mem_alloc.tgid),
                            Some(mem_alloc.cgroup_id),
                            &mem_alloc.command,
                        );
                        metadata.enrich(&cache).await;
                        metrics.record_enter_mem_alloc(&mem_alloc, &metadata);
                    }
                    _ => continue,
                }

                let tgid = mem_alloc.tgid;
                let command = String::from_utf8_lossy(&mem_alloc.command);
                let addr = mem_alloc.addr;
                let length = mem_alloc.length;

                info!(
                    "MemAlloc - tgid: {}, command: {}, addr: {}, length: {}",
                    tgid, command, addr, length
                );
            }
        }
    }

    /// Read [`SchedStatWait`] events and record OpenTelemetry observations.
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_sched_stat_wait(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<SchedStatWait>() {
                error!(
                    "Corrupted SchedStatWait data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<SchedStatWait>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<SchedStatWait>() {
                let sched_stat_wait: SchedStatWait =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                match exporter {
                    "otlp" => {
                        let mut metadata = Metadata::from_ebpf(
                            Some(sched_stat_wait.tgid),
                            Some(sched_stat_wait.cgroup_id),
                            &sched_stat_wait.command,
                        );
                        metadata.enrich(&cache).await;
                        metrics.record_sched_stat_wait(&sched_stat_wait, &metadata);
                    }
                    _ => continue,
                }

                let tgid = sched_stat_wait.tgid;
                let command = String::from_utf8_lossy(&sched_stat_wait.command);
                let delay = sched_stat_wait.delay;

                info!(
                    "SchedStatWait - tgid: {}, command: {}, delay: {}",
                    tgid, command, delay
                );
            }
        }
    }

    /// Read [`SchedStatRuntime`] events and record OpenTelemetry observations.
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_sched_stat_runtime(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<SchedStatRuntime>() {
                error!(
                    "Corrupted SchedStatRuntime data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<SchedStatRuntime>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<SchedStatRuntime>() {
                let sched_stat_runtime: SchedStatRuntime =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                match exporter {
                    "otlp" => {
                        let mut metadata = Metadata::from_ebpf(
                            Some(sched_stat_runtime.tgid),
                            Some(sched_stat_runtime.cgroup_id),
                            &sched_stat_runtime.command,
                        );
                        metadata.enrich(&cache).await;
                        metrics.record_sched_stat_runtime(&sched_stat_runtime, &metadata);
                    }
                    _ => continue,
                }

                let tgid = sched_stat_runtime.tgid;
                let command = String::from_utf8_lossy(&sched_stat_runtime.command);
                let runtime = sched_stat_runtime.runtime;

                info!(
                    "SchedStatRuntime - tgid: {}, command: {}, runtime: {}",
                    tgid, command, runtime
                );
            }
        }
    }

    /// Read [`CpuIdle`] events and record OpenTelemetry observations.
    #[cfg(feature = "monitoring-structs")]
    pub async fn read_cpu_idle(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<CpuIdle>() {
                error!(
                    "Corrupted CpuIdle data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<CpuIdle>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<CpuIdle>() {
                let cpu_idle: CpuIdle =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                match exporter {
                    "otlp" => {
                        let metadata = Metadata::from_ebpf(None, None, &[]);
                        metrics.record_cpu_idle(&cpu_idle, &metadata);
                    }
                    _ => continue,
                }

                let cpu_id = cpu_idle.cpu_id;
                let state = cpu_idle.state;

                info!(
                    "CpuIdle state changed - cpu_id: {}, state: {}",
                    cpu_id, state
                );
            }
        }
    }

    #[cfg(feature = "monitoring-structs")]
    pub async fn read_ssl_events(
        buffers: &mut [BytesMut],
        tot_events: i32,
        offset: i32,
        exporter: &str,
        metrics: Arc<Metrics>,
        cache: Arc<RwLock<ServiceCache>>,
    ) {
        for i in offset..tot_events {
            use crate::buffer_type::SslEvent;

            let vec_bytes = &buffers[i as usize];
            if vec_bytes.len() < std::mem::size_of::<SslEvent>() {
                error!(
                    "Corrupted SslEvent data. Raw data: {}. Readed {} bytes expected {} bytes",
                    vec_bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                    vec_bytes.len(),
                    std::mem::size_of::<SslEvent>()
                );
                continue;
            }
            if vec_bytes.len() >= std::mem::size_of::<SslEvent>() {
                let ssl_event: SslEvent =
                    unsafe { std::ptr::read_unaligned(vec_bytes.as_ptr() as *const _) };

                let direction = ssl_event.direction;

                match direction {
                    // 0 = read // 1= write
                    0 => match exporter {
                        "otlp" => {
                            let mut metadata = Metadata::from_ebpf(
                                Some(ssl_event.tgid),
                                Some(ssl_event.cgroup_id),
                                &ssl_event.comm,
                            );
                            metadata.enrich(&cache).await;
                            metrics.record_ssl_read_bytes(&ssl_event, &metadata);
                            let tgid = ssl_event.tgid;
                            let command = String::from_utf8_lossy(&ssl_event.comm);
                            let size = ssl_event.size;
                            let requested = ssl_event.requested;

                            info!(
                                "SSL event: - tgid: {},- command : {}, - direction: {}, - size: {} , - requested : {}",
                                tgid, command, direction, size, requested
                            );
                        }
                        _ => continue,
                    },
                    1 => match exporter {
                        "otlp" => {
                            let mut metadata = Metadata::from_ebpf(
                                Some(ssl_event.tgid),
                                Some(ssl_event.cgroup_id),
                                &ssl_event.comm,
                            );
                            metadata.enrich(&cache).await;
                            metrics.record_ssl_write_bytes(&ssl_event, &metadata);
                            let tgid = ssl_event.tgid;
                            let command = String::from_utf8_lossy(&ssl_event.comm);
                            let size = ssl_event.size;
                            let requested = ssl_event.requested;

                            info!(
                                "SSL event: - tgid: {},- command : {}, - direction: {}, - size: {} , - requested : {}",
                                tgid, command, direction, size, requested
                            );
                        }
                        _ => continue,
                    },
                    _ => continue, // direction data not logged or recorded
                }
            }
        }
    }
}

/// Read perf-buffer events in a loop and dispatch to the appropriate [`Consumer`] handler.
///
/// This function runs indefinitely (or until the process receives `SIGINT`).
/// It polls every CPU buffer every 100 ms, reads available events, and routes
/// them to the matching `Consumer::read_*` method.
///
/// # Arguments
/// - `array_buffers` — per-CPU `PerfEventArrayBuffer` handles opened by [`fill_buffers`].
/// - `buffers` — pre-allocated `BytesMut` scratch space sized by [`BufferSize::set_buffer`].
/// - `consumer` — discriminator that selects which `read_*` method to invoke.
/// - `metrics` — optional [`Metrics`] handle; required when `consumer` is a monitoring variant.
#[cfg(feature = "buffer-reader")]
pub async fn read_perf_buffer<T: std::borrow::BorrowMut<aya::maps::MapData>>(
    mut array_buffers: Vec<aya::maps::perf::PerfEventArrayBuffer<T>>,
    mut buffers: Vec<bytes::BytesMut>,
    consumer: Consumer,
    #[cfg(feature = "monitoring-structs")] metrics: Option<Arc<Metrics>>,
    cache: Option<Arc<RwLock<ServiceCache>>>,
) {
    loop {
        for buf in array_buffers.iter_mut() {
            match buf.read_events(&mut buffers) {
                Ok(events) => {
                    if events.lost > 0 {
                        tracing::debug!("Lost events: {} ", events.lost);
                    }
                    if events.read > 0 {
                        tracing::debug!("Readed events: {}", events.read);
                        let offset = 0;
                        let tot_events = events.read as i32;

                        match consumer {
                            #[cfg(feature = "network-structs")]
                            Consumer::PacketLog => {
                                Consumer::read_packet_log(&mut buffers, tot_events, offset).await
                            }
                            #[cfg(feature = "network-structs")]
                            Consumer::TcpPacketRegistry => {
                                Consumer::read_tcp_registry_log(&mut buffers, tot_events, offset)
                                    .await
                            }
                            #[cfg(feature = "network-structs")]
                            Consumer::VethLog => {
                                Consumer::read_and_handle_veth_log(&mut buffers, tot_events, offset)
                                    .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::PacketLossMetrics => {
                                Consumer::read_packet_loss_metrics(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics
                                        .clone()
                                        .expect("Metrics required for PacketLossMetrics"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::TimeStampMetrics => {
                                Consumer::read_timestamp_metrics(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics
                                        .clone()
                                        .expect("Metric required for TimeStampMetrics"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::CpuFrequency => {
                                Consumer::read_cpu_frequency(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics.clone().expect("Metric required for CpuFrequency"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::MemAlloc => {
                                Consumer::read_mem_alloc(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics.clone().expect("Metric required for MemAlloc"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::SchedStatWait => {
                                Consumer::read_sched_stat_wait(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics.clone().expect("Metric required for SchedStatWait"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::SchedStatRuntime => {
                                Consumer::read_sched_stat_runtime(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics
                                        .clone()
                                        .expect("Metric required for SchedStatRuntime"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::CpuIdle => {
                                Consumer::read_cpu_idle(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics.clone().expect("Metric required for CpuIdle"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                            #[cfg(feature = "monitoring-structs")]
                            Consumer::SslEvents => {
                                Consumer::read_ssl_events(
                                    &mut buffers,
                                    tot_events,
                                    offset,
                                    "otlp",
                                    metrics.clone().expect("Metric required for SslEvents"),
                                    cache.clone().expect("cache required for PacketLossMetrics"),
                                )
                                .await
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Cannot read events from buffer. Reason: {} ", e);
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
