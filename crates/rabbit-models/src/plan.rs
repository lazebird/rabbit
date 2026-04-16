//! Task Planner Module Models

use chrono::{DateTime, Local, NaiveTime};
use serde::{Deserialize, Serialize};

/// Task/Reminder item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub schedule: Schedule,
    pub enabled: bool,
    pub created_at: DateTime<Local>,
}

/// Schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    Once { datetime: DateTime<Local> },
    Daily { time: NaiveTime },
    Weekly { day: WeekDay, time: NaiveTime },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeekDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Pending,
    Triggered,
    Acknowledged,
    Snoozed,
    Disabled,
}

/// Task execution log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLog {
    pub task_id: String,
    pub triggered_at: DateTime<Local>,
    pub acknowledged_at: Option<DateTime<Local>>,
}
