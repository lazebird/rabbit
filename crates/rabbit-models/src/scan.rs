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

/// Scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub timeout_ms: u64,
    pub concurrent: usize,
    pub retry_count: u32,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        // ScannerConfig doesn't have direct equivalent in config.rs
        // These are internal performance tuning parameters
        Self {
            timeout_ms: 1500,
            concurrent: 256,
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
        let range = ScanRange::new(Ipv4Addr::new(10, 0, 0, 1), Ipv4Addr::new(10, 0, 0, 255));
        let json = serde_json::to_string(&range).unwrap();
        let parsed: ScanRange = serde_json::from_str(&json).unwrap();
        assert_eq!(range.start, parsed.start);
        assert_eq!(range.end, parsed.end);
    }

    #[test]
    fn test_scanner_config() {
        let config = ScannerConfig {
            timeout_ms: 2000,
            concurrent: 100,
            retry_count: 1,
        };
        assert_eq!(config.timeout_ms, 2000);
        assert_eq!(config.concurrent, 100);
        assert_eq!(config.retry_count, 1);
    }

    // ScanResult 已移至 rabbit-core 作为私有结构，此处不再测试

    #[test]
    fn test_scanner_state() {
        let state = ScannerState::Scanning { progress: 50 };
        match state {
            ScannerState::Scanning { progress } => assert_eq!(progress, 50),
            _ => panic!("Unexpected state"),
        }
    }
}
