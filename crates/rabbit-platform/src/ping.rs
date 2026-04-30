//! Platform-specific ping implementation

use super::Result;
use std::net::IpAddr;


/// Platform ping capability
pub struct PlatformPing;

impl Default for PlatformPing {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformPing {
    /// Create a new ping instance
    pub fn new() -> Self {
        Self
    }
    
    /// Resolve hostname to IP address
    pub async fn resolve(&self, hostname: &str) -> Result<Option<IpAddr>> {
        use tokio::net::lookup_host;
        
        let addrs: Vec<_> = lookup_host(hostname).await?.collect();
        Ok(addrs.into_iter().next().map(|addr| addr.ip()))
    }
}


/// Check if the application has permission to use raw sockets
pub fn check_ping_permission() -> bool {
    #[cfg(target_os = "windows")]
    {
        // On Windows, check if running as administrator
        is_elevated()
    }
    
    #[cfg(target_os = "linux")]
    {
        // On Linux, check for CAP_NET_RAW capability or root
        unsafe { libc::getuid() == 0 }
    }
    
    #[cfg(target_os = "macos")]
    {
        unsafe { libc::getuid() == 0 }
    }
}

#[cfg(target_os = "windows")]
fn is_elevated() -> bool {
    use std::process::Command;
    
    // Simple check - try to open a privileged resource
    Command::new("net")
        .args(&["session"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Request elevated permissions (platform-specific)
pub fn request_elevation() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Use ShellExecute to relaunch as admin
        println!("Please run as administrator for ping functionality");
    }
    
    #[cfg(unix)]
    {
        println!("Please run with sudo for ping functionality");
    }
    
    Ok(())
}
