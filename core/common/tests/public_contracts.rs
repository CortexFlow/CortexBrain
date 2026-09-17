use cortexbrain_common::formatters::{format_ipv4, format_ipv6};

#[test]
fn address_formatters_preserve_network_byte_order() {
    let ipv4 = u32::from_ne_bytes([192, 0, 2, 1]);
    assert_eq!(format_ipv4(ipv4), "192.0.2.1");
    assert_eq!(format_ipv4(0), "0.0.0.0");

    assert_eq!(format_ipv6(&[0x2001_0db8, 0, 0, 1]), "2001:db8:0:0:0:0:0:1");
    assert_eq!(format_ipv6(&[0; 4]), "0:0:0:0:0:0:0:0");
}

#[cfg(any(feature = "network-structs", feature = "monitoring-structs"))]
mod buffer_contracts {
    use cortexbrain_common::buffer_type::{IpProtocols, reverse_be_addr};

    #[test]
    fn protocol_numbers_follow_the_ipv4_header_contract() {
        assert!(matches!(IpProtocols::try_from(1), Ok(IpProtocols::ICMP)));
        assert!(matches!(IpProtocols::try_from(6), Ok(IpProtocols::TCP)));
        assert!(matches!(IpProtocols::try_from(17), Ok(IpProtocols::UDP)));

        for unknown in [0, 2, 255] {
            assert!(IpProtocols::try_from(unknown).is_err());
        }
    }

    #[test]
    fn raw_ipv4_addresses_are_reversed_by_octet() {
        let raw_address = u32::from_le_bytes([192, 0, 2, 1]);
        assert_eq!(reverse_be_addr(raw_address).octets(), [192, 0, 2, 1]);
        assert_eq!(reverse_be_addr(0).octets(), [0, 0, 0, 0]);
        assert_eq!(reverse_be_addr(u32::MAX).octets(), [255, 255, 255, 255]);
    }
}

#[cfg(feature = "network-structs")]
mod network_event_contracts {
    use cortexbrain_common::buffer_type::{PacketLog, TcpPacketRegistry, VethLog};
    use std::mem::{align_of, offset_of, size_of};

    fn assert_event_traits<T: aya::Pod + bytemuck::Zeroable + Copy>() {}

    #[test]
    fn network_events_are_pod_compatible() {
        assert_event_traits::<PacketLog>();
        assert_event_traits::<VethLog>();
        assert_event_traits::<TcpPacketRegistry>();
    }

    #[test]
    fn network_event_layouts_match_the_ebpf_side() {
        assert_eq!(size_of::<PacketLog>(), 24);
        assert_eq!(align_of::<PacketLog>(), 4);
        assert_eq!(offset_of!(PacketLog, proto), 0);
        assert_eq!(offset_of!(PacketLog, src_ip), 4);
        assert_eq!(offset_of!(PacketLog, src_port), 8);
        assert_eq!(offset_of!(PacketLog, dst_ip), 12);
        assert_eq!(offset_of!(PacketLog, dst_port), 16);
        assert_eq!(offset_of!(PacketLog, pid), 20);

        assert_eq!(size_of::<VethLog>(), 39);
        assert_eq!(align_of::<VethLog>(), 1);
        assert_eq!(offset_of!(VethLog, name), 0);
        assert_eq!(offset_of!(VethLog, state), 16);
        assert_eq!(offset_of!(VethLog, dev_addr), 24);
        assert_eq!(offset_of!(VethLog, event_type), 30);
        assert_eq!(offset_of!(VethLog, netns), 31);
        assert_eq!(offset_of!(VethLog, pid), 35);

        assert_eq!(size_of::<TcpPacketRegistry>(), 48);
        assert_eq!(align_of::<TcpPacketRegistry>(), 8);
        assert_eq!(offset_of!(TcpPacketRegistry, proto), 0);
        assert_eq!(offset_of!(TcpPacketRegistry, src_ip), 4);
        assert_eq!(offset_of!(TcpPacketRegistry, dst_ip), 8);
        assert_eq!(offset_of!(TcpPacketRegistry, src_port), 12);
        assert_eq!(offset_of!(TcpPacketRegistry, dst_port), 14);
        assert_eq!(offset_of!(TcpPacketRegistry, pid), 16);
        assert_eq!(offset_of!(TcpPacketRegistry, command), 20);
        assert_eq!(offset_of!(TcpPacketRegistry, cgroup_id), 40);
    }
}

#[cfg(feature = "monitoring-structs")]
mod monitoring_event_contracts {
    use cortexbrain_common::buffer_type::{
        CpuFrequency, CpuIdle, MemAlloc, PacketLossMetrics, SchedStatRuntime, SchedStatWait,
        SslEvent, TASK_COMM_LEN, TimeStampMetrics,
    };
    use std::mem::{align_of, offset_of, size_of};

    fn assert_event_traits<T: aya::Pod + bytemuck::Zeroable + Copy>() {}

    #[test]
    fn monitoring_events_are_pod_compatible() {
        assert_event_traits::<PacketLossMetrics>();
        assert_event_traits::<TimeStampMetrics>();
        assert_event_traits::<CpuFrequency>();
        assert_event_traits::<MemAlloc>();
        assert_event_traits::<SchedStatWait>();
        assert_event_traits::<SchedStatRuntime>();
        assert_event_traits::<CpuIdle>();
        assert_event_traits::<SslEvent>();
    }

    #[test]
    fn monitoring_event_layouts_match_the_ebpf_side() {
        assert_eq!(TASK_COMM_LEN, 16);

        assert_eq!(align_of::<PacketLossMetrics>(), 1);
        assert_eq!(offset_of!(PacketLossMetrics, tgid), 0);
        assert_eq!(offset_of!(PacketLossMetrics, comm), 4);
        assert_eq!(offset_of!(PacketLossMetrics, ts_us), 20);
        assert_eq!(offset_of!(PacketLossMetrics, sk_err), 28);
        assert_eq!(offset_of!(PacketLossMetrics, sk_drops), 52);

        assert_eq!(align_of::<TimeStampMetrics>(), 1);
        assert_eq!(offset_of!(TimeStampMetrics, delta_us), 0);
        assert_eq!(offset_of!(TimeStampMetrics, ts_us), 8);
        assert_eq!(offset_of!(TimeStampMetrics, tgid), 16);
        assert_eq!(offset_of!(TimeStampMetrics, comm), 20);
        assert_eq!(offset_of!(TimeStampMetrics, lport), 36);
        assert_eq!(offset_of!(TimeStampMetrics, af), 40);
        assert_eq!(offset_of!(TimeStampMetrics, saddr_v4), 42);
        assert_eq!(offset_of!(TimeStampMetrics, daddr_v6), 66);

        assert_eq!(size_of::<CpuFrequency>(), 24);
        assert_eq!(align_of::<CpuFrequency>(), 1);
        assert_eq!(offset_of!(CpuFrequency, bytes_alloc), 0);
        assert_eq!(offset_of!(CpuFrequency, pid), 4);
        assert_eq!(offset_of!(CpuFrequency, command), 8);

        assert_eq!(align_of::<MemAlloc>(), 1);
        assert_eq!(offset_of!(MemAlloc, tgid), 0);
        assert_eq!(offset_of!(MemAlloc, length), 4);
        assert_eq!(offset_of!(MemAlloc, addr), 12);
        assert_eq!(offset_of!(MemAlloc, command), 20);

        assert_eq!(align_of::<SchedStatWait>(), 1);
        assert_eq!(offset_of!(SchedStatWait, tgid), 0);
        assert_eq!(offset_of!(SchedStatWait, delay), 4);
        assert_eq!(offset_of!(SchedStatWait, command), 12);

        assert_eq!(align_of::<SchedStatRuntime>(), 1);
        assert_eq!(offset_of!(SchedStatRuntime, tgid), 0);
        assert_eq!(offset_of!(SchedStatRuntime, runtime), 4);
        assert_eq!(offset_of!(SchedStatRuntime, command), 12);

        assert_eq!(size_of::<CpuIdle>(), 8);
        assert_eq!(align_of::<CpuIdle>(), 1);
        assert_eq!(offset_of!(CpuIdle, cpu_id), 0);
        assert_eq!(offset_of!(CpuIdle, state), 4);

        assert_eq!(align_of::<SslEvent>(), 1);
        assert_eq!(offset_of!(SslEvent, tgid), 0);
        assert_eq!(offset_of!(SslEvent, comm), 4);
        assert_eq!(offset_of!(SslEvent, ts_us), 20);
        assert_eq!(offset_of!(SslEvent, direction), 28);
        assert_eq!(offset_of!(SslEvent, size), 29);
        assert_eq!(offset_of!(SslEvent, requested), 33);
    }
}
