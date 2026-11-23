/// Environment variable name for the BPF program file path.
/// Used by all components to load their eBPF programs.
pub const BPF_PATH: &str = "BPF_PATH";

/// Environment variable name for the BPF map pinning path.
/// Used for sharing maps between eBPF programs.
pub const PIN_MAP_PATH: &str = "PIN_MAP_PATH";

/// Default directory where BPF maps are pinned when PIN_MAP_PATH is not set.
pub const DEFAULT_PIN_MAP_PATH: &str = "/sys/fs/bpf/maps";

/// Name of the pinned blocklist map.
pub const BLOCKLIST_MAP_NAME: &str = "Blocklist";

/// Environment variable pointing to a local blocklist file (Docker/non-Kubernetes).
pub const BLOCKLIST_FILE_ENV: &str = "BLOCKLIST_FILE";

/// Default interval between blocklist refreshes.
pub const DEFAULT_BLOCKLIST_SYNC_INTERVAL_SECS: u64 = 30;

/// Environment variable name for the OpenSSL library path used by SSL uprobes.
/// When set, this path is used directly. When unset, the library is located
/// by searching the default system library directories.
pub const LIBSSL_PATH: &str = "LIBSSL_PATH";

pub const DEFAULT_MAP_READING_TIME: &str = "DEFAULT_MAP_READING_TIME";

/// Direction of a connection observed by the identity service.
pub const CONNECTION_DIRECTION_INGRESS: u8 = 0;
pub const CONNECTION_DIRECTION_EGRESS: u8 = 1;

/// Idle time after which a previously observed identity flow is considered new again.
pub const IDENTITY_FLOW_TTL_SECS: u64 = 300;

/// Environment variable name for the identity flow cache capacity (entries).
pub const IDENTITY_FLOW_CACHE_CAPACITY: &str = "IDENTITY_FLOW_CACHE_CAPACITY";

/// Default number of identity flows tracked before LRU eviction kicks in.
pub const DEFAULT_IDENTITY_FLOW_CACHE_CAPACITY: usize = 65_536;
