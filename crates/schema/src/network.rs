//! Network provider trait for platform-independent network queries.
//!
//! Service layer uses this trait to obtain MAC addresses etc. without
//! depending on the `adapter` crate.

use std::collections::HashMap;
use std::net::Ipv4Addr;

/// Platform-specific network query interface.
/// `service` uses this trait to access platform capabilities without
/// depending on the `adapter` crate directly.
pub trait NetworkProvider: Send + Sync {
    /// Look up MAC address for a given IP from the kernel ARP table.
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String>;

    /// Return IP → MAC mapping for all local (non-loopback) interfaces.
    ///
    /// Used at scan start to resolve the scanner's own IPs without
    /// querying the kernel ARP table.
    fn get_local_mac_table(&self) -> HashMap<Ipv4Addr, String> {
        HashMap::new()
    }
}
