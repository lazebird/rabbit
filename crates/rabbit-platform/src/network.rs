//! Network interface utilities

use super::Result;
use std::net::Ipv4Addr;

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
    let _output = Command::new("ip")
        .args(&["-j", "addr", "show"])
        .output()?;
    
    #[cfg(target_os = "macos")]
    let output = Command::new("ifconfig")
        .output()?;
    
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

/// Calculate IP range from CIDR or start/end
pub fn calculate_ip_range(start: Ipv4Addr, end: Ipv4Addr) -> Vec<Ipv4Addr> {
    let start_u32 = u32::from(start);
    let end_u32 = u32::from(end);
    
    (start_u32..=end_u32)
        .map(|ip| Ipv4Addr::from(ip))
        .collect()
}
