//! Network provider trait for platform-independent network queries.
//!
//! Service layer uses this trait to obtain MAC addresses etc. without
//! depending on the `adapter` crate.

use std::net::Ipv4Addr;

/// Platform-specific network query interface.
/// `service` uses this trait to access platform capabilities without
/// depending on the `adapter` crate directly.
pub trait NetworkProvider: Send + Sync {
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String>;
}
