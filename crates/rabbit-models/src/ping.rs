//! Ping Module Models

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Ping target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingTarget {
    pub address: String,
    pub ip: Option<IpAddr>,
    pub count: u32,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub stop_on_loss: bool,
}

impl PingTarget {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            ip: None,
            count: 4,
            interval_ms: 1000,
            timeout_ms: 2000,
            stop_on_loss: false,
        }
    }
}

/// Ping result for a single packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub seq: u16,
    pub success: bool,
    pub duration_ms: Option<f64>,
    pub error: Option<String>,
}

/// Summary of ping session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingSummary {
    pub target: String,
    pub sent: u32,
    pub received: u32,
    pub lost: u32,
    pub loss_rate: f64,
    pub min_ms: Option<f64>,
    pub max_ms: Option<f64>,
    pub avg_ms: Option<f64>,
}

/// Ping session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingState {
    Idle,
    Running,
    Paused,
    Completed,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_target_new() {
        let target = PingTarget::new("127.0.0.1");
        assert_eq!(target.address, "127.0.0.1");
        assert_eq!(target.count, 4);
        assert_eq!(target.interval_ms, 1000);
        assert_eq!(target.timeout_ms, 2000);
        assert!(target.ip.is_none());
    }

    #[test]
    fn test_ping_target_serialization() {
        let target = PingTarget::new("google.com");
        let json = serde_json::to_string(&target).unwrap();
        let parsed: PingTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(target.address, parsed.address);
    }

    #[test]
    fn test_ping_result_success() {
        let result = PingResult {
            seq: 1,
            success: true,
            duration_ms: Some(15.5),
            error: None,
        };
        assert!(result.success);
        assert_eq!(result.seq, 1);
        assert!(result.duration_ms.is_some());
    }

    #[test]
    fn test_ping_result_failure() {
        let result = PingResult {
            seq: 2,
            success: false,
            duration_ms: None,
            error: Some("Timeout".to_string()),
        };
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_ping_summary() {
        let summary = PingSummary {
            target: "8.8.8.8".to_string(),
            sent: 4,
            received: 3,
            lost: 1,
            loss_rate: 25.0,
            min_ms: Some(10.0),
            max_ms: Some(20.0),
            avg_ms: Some(15.0),
        };
        assert_eq!(summary.target, "8.8.8.8");
        assert_eq!(summary.sent, 4);
        assert_eq!(summary.loss_rate, 25.0);
    }

    #[test]
    fn test_ping_state() {
        assert_ne!(PingState::Idle, PingState::Running);
        assert_ne!(PingState::Running, PingState::Paused);
        assert_eq!(PingState::Error, PingState::Error);
    }
}
