//! IP Scanner Module Models

use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

/// IP range to scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRange {
    pub start: Ipv4Addr,
    pub end: Ipv4Addr,
    pub port: u16,
}

impl ScanRange {
    pub fn new(start: Ipv4Addr, end: Ipv4Addr) -> Self {
        Self {
            start,
            end,
            port: 0, // ICMP ping by default
        }
    }
}

/// Scan result for a single host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub ip: Ipv4Addr,
    pub online: bool,
    pub hostname: Option<String>,
    pub response_time_ms: Option<f64>,
    pub open_ports: Vec<u16>,
}

/// Scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub timeout_ms: u64,
    pub concurrent: usize,
    pub retry_count: u32,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            concurrent: 100,
            retry_count: 1,
        }
    }
}

/// Scanner state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScannerState {
    Idle,
    Scanning { progress: u8 },
    Completed,
    Cancelled,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_range_new() {
        let start = Ipv4Addr::new(192, 168, 1, 1);
        let end = Ipv4Addr::new(192, 168, 1, 254);
        let range = ScanRange::new(start, end);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
        assert_eq!(range.port, 0);
    }

    #[test]
    fn test_scan_range_serialization() {
        let range = ScanRange::new(
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(10, 0, 0, 255),
        );
        let json = serde_json::to_string(&range).unwrap();
        let parsed: ScanRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range.start, parsed.start);
        assert_eq!(range.end, parsed.end);
    }

    #[test]
    fn test_scanner_config_default() {
        let config = ScannerConfig::default();
        assert_eq!(config.timeout_ms, 2000);
        assert_eq!(config.concurrent, 100);
        assert_eq!(config.retry_count, 1);
    }

    #[test]
    fn test_scan_result() {
        let result = ScanResult {
            ip: Ipv4Addr::new(192, 168, 1, 100),
            online: true,
            hostname: Some("host.example.com".to_string()),
            response_time_ms: Some(5.5),
            open_ports: vec![80, 443],
        };
        assert!(result.online);
        assert!(result.hostname.is_some());
        assert_eq!(result.open_ports.len(), 2);
    }

    #[test]
    fn test_scanner_state() {
        let state = ScannerState::Scanning { progress: 50 };
        match state {
            ScannerState::Scanning { progress } => assert_eq!(progress, 50),
            _ => panic!("Unexpected state"),
        }
    }
}
