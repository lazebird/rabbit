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
    pub state: TaskState,
    pub snooze_until: Option<DateTime<Local>>,
    pub last_triggered: Option<DateTime<Local>>,
}

impl Task {
    /// Create a new task
    pub fn new(id: String, title: String, schedule: Schedule) -> Self {
        Self {
            id,
            title,
            description: None,
            schedule,
            enabled: true,
            created_at: Local::now(),
            state: TaskState::Pending,
            snooze_until: None,
            last_triggered: None,
        }
    }

    /// Check if task is snoozed and should not trigger
    pub fn is_snoozed(&self) -> bool {
        if let Some(snooze_until) = self.snooze_until {
            Local::now() < snooze_until
        } else {
            false
        }
    }

    /// Acknowledge the task
    pub fn acknowledge(&mut self) {
        self.state = TaskState::Acknowledged;
        self.snooze_until = None;
    }

    /// Snooze the task for specified minutes
    pub fn snooze(&mut self, minutes: i64) {
        self.state = TaskState::Snoozed;
        self.snooze_until = Some(Local::now() + chrono::Duration::minutes(minutes));
    }

    /// Mark task as triggered
    pub fn trigger(&mut self) {
        self.state = TaskState::Triggered;
        self.last_triggered = Some(Local::now());
    }

    /// Reset task state to pending
    pub fn reset(&mut self) {
        self.state = TaskState::Pending;
        self.snooze_until = None;
    }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_state_transitions() {
        let schedule = Schedule::Once {
            datetime: Local::now() + chrono::Duration::hours(1),
        };
        let mut task = Task::new("test-1".to_string(), "Test Task".to_string(), schedule);

        assert_eq!(task.state, TaskState::Pending);
        assert!(!task.is_snoozed());

        task.trigger();
        assert_eq!(task.state, TaskState::Triggered);
        assert!(task.last_triggered.is_some());

        task.acknowledge();
        assert_eq!(task.state, TaskState::Acknowledged);

        task.reset();
        assert_eq!(task.state, TaskState::Pending);
    }

    #[test]
    fn test_task_snooze() {
        let schedule = Schedule::Once {
            datetime: Local::now() + chrono::Duration::hours(1),
        };
        let mut task = Task::new("test-2".to_string(), "Test Task".to_string(), schedule);

        task.snooze(10);
        assert_eq!(task.state, TaskState::Snoozed);
        assert!(task.is_snoozed());
        assert!(task.snooze_until.is_some());
    }
}
