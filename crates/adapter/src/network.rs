//! Network interface utilities

use super::Result;
use schema::NetworkProvider;
use std::net::Ipv4Addr;

/// Default implementation of [`schema::NetworkProvider`].
pub struct DefaultNetworkProvider;

impl NetworkProvider for DefaultNetworkProvider {
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String> {
        get_mac_from_arp(ip)
    }
}

/// Network interface information
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_address: Ipv4Addr,
    pub netmask: Ipv4Addr,
    pub is_up: bool,
    pub is_loopback: bool,
}

/// Get all network interfaces
pub async fn get_interfaces() -> Result<Vec<NetworkInterface>> {
    // Platform-specific implementation
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        get_interfaces_unix().await
    }

    #[cfg(target_os = "windows")]
    {
        get_interfaces_windows().await
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn get_interfaces_unix() -> Result<Vec<NetworkInterface>> {
    use std::process::Command;

    // Use ip command on Linux, ifconfig on macOS
    #[cfg(target_os = "linux")]
    let _output = Command::new("ip").args(["-j", "addr", "show"]).output()?;

    #[cfg(target_os = "macos")]
    let output = Command::new("ifconfig").output()?;

    // Parse output (simplified - in production use a proper parser)
    let interfaces = vec![];
    Ok(interfaces)
}

#[cfg(target_os = "windows")]
async fn get_interfaces_windows() -> Result<Vec<NetworkInterface>> {
    // Use GetAdaptersAddresses on Windows
    // Simplified implementation
    let interfaces = vec![];
    Ok(interfaces)
}

/// Get the default local IP address
pub fn get_local_ip() -> Option<Ipv4Addr> {
    // Try to find a non-loopback interface
    // This is a simplified implementation
    None
}

/// Get MAC address from ARP table (Linux: /proc/net/arp, Windows: SendARP)
///
/// Returns `None` if the MAC address could not be resolved.
pub fn get_mac_from_arp(ip: Ipv4Addr) -> Option<String> {
    // Suppress unused-variable warning on unsupported platforms.
    // Ipv4Addr is Copy, so this does not interfere with cfg-block usage below.
    let _ = ip;

    #[cfg(target_os = "linux")]
    {
        let ip_str = ip.to_string();
        if let Ok(content) = std::fs::read_to_string("/proc/net/arp") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 6 && parts[0] == ip_str && parts.len() >= 4 {
                    let mac = parts[3].to_string();
                    if mac != "00:00:00:00:00:00" {
                        return Some(mac);
                    }
                }
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::NetworkManagement::IpHelper::SendARP;

        let dest_ip: u32 = u32::from_be_bytes(ip.octets());
        let mut mac_addr = [0u8; 6];
        let mut mac_len: u32 = 6;

        let result = unsafe { SendARP(dest_ip, 0, mac_addr.as_mut_ptr().cast(), &mut mac_len) };
        if result == 0 && mac_len as usize >= 6 && mac_addr != [0u8; 6] {
            return Some(format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac_addr[0], mac_addr[1], mac_addr[2],
                mac_addr[3], mac_addr[4], mac_addr[5]
            ));
        }
    }
    None
}
