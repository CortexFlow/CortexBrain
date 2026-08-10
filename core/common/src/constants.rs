/// Environment variable name for the BPF program file path.
/// Used by all components to load their eBPF programs.
pub const BPF_PATH: &str = "BPF_PATH";

/// Environment variable name for the BPF map pinning path.
/// Used for sharing maps between eBPF programs.
pub const PIN_MAP_PATH: &str = "PIN_MAP_PATH";

/// Environment variable name for the OpenSSL library path used by SSL uprobes.
/// When set, this path is used directly. When unset, the library is located
/// by searching the default system library directories.
pub const LIBSSL_PATH: &str = "LIBSSL_PATH";
