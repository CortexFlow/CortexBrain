pub struct OFFSETS;

/*
    * ETHERNET TYPE II FRAME:
    * Reference: https://it.wikipedia.org/wiki/Frame_Ethernet
    *
    * 6 bytes for the destination mac address;
    * 6 bytes for the source mac address;
    * 2 bytes for the ethertype;
    * from 46 to 1500 bytes for the payload;
    * 4 bytes for the FCS (Checksum)

*/

/*
    * Ipv4 stack reference:
    * https://en.wikipedia.org/wiki/IPv4#Header
    *
    * Original reference:

   |  Time to Live |    Protocol   |         Header Checksum       |     4 bytes            12
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                       Source Address                          |     4 bytes            16
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Destination Address                        |     4 bytes            20
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Options                    |    Padding    |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    *                    Internet Datagram Header
    *
    *
    *
    *
    *
    *                        TCP/UDP header datagram:
    *
    *   reference: https://en.wikipedia.org/wiki/User_Datagram_Protocol
    *
    *    src port: 34 byte
    *    dst port: 36 byte

*/

impl OFFSETS {
    pub const IPV4_ETHERTYPE: u16 = 0x0800;

    //IPV4 STACK
    pub const SRC_BYTE_OFFSET: usize = 12; //source address offset for ipv4 addresses
    pub const DST_BYTE_OFFSET: usize = 16; //destination address offset for ipv4 addresses
    pub const IPV4_PROTOCOL_OFFSET: usize = 9; //ipv4 protocol offset

    //ETHERNET STACK
    pub const SRC_MAC: usize = 6; // source mac address offset
    pub const DST_MAC: usize = 6; // destination mac address offset
    pub const ETHERTYPE_BYTES: usize = 2; // ethertype bytes doc: https://en.wikipedia.org/wiki/EtherType

    //TCP UDP STACK
    pub const SRC_PORT_OFFSET_FROM_IP_HEADER: usize = 0; //source port offset
    pub const DST_PORT_OFFSET_FROM_IP_HEADER: usize = 2; //destination port offset

    // TOTAL BYTES SUM
    pub const ETH_STACK_BYTES: usize = OFFSETS::SRC_MAC + OFFSETS::DST_MAC + OFFSETS::ETHERTYPE_BYTES; // ethernet protocol total stacked bytes
    pub const DST_T0TAL_BYTES_OFFSET: usize = OFFSETS::ETH_STACK_BYTES + OFFSETS::DST_BYTE_OFFSET; // destination total bytes offset
    pub const SRC_T0TAL_BYTES_OFFSET: usize = OFFSETS::ETH_STACK_BYTES + OFFSETS::SRC_BYTE_OFFSET; //source total bytes offset
    pub const PROTOCOL_T0TAL_BYTES_OFFSET: usize =
        OFFSETS::ETH_STACK_BYTES + OFFSETS::IPV4_PROTOCOL_OFFSET; // total bytes offset
}
