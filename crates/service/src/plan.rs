//! Task Planner Service
//!
//! Uses per-task tokio timers for precise trigger, no polling.

use crate::{ui_channel::UiData, Module, Result, ServiceError, ServiceUpdateResult};

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep_until;
use tracing::info;

/// Internal Task model
struct Task {
    pub title: String,
    pub schedule: Schedule,
    pub enabled: bool,
    pub handle: Option<tokio::task::AbortHandle>,
}

impl Task {
    fn new(title: String, schedule: Schedule) -> Self {
        Self {
            title,
            schedule,
            enabled: true,
            handle: None,
        }
    }
}

#[derive(Clone)]
enum Schedule {
    Once { datetime: DateTime<Local> },
    Repeating { datetime: DateTime<Local>, cycle: i32, unit: RepeatUnit },
}

#[derive(Clone, Copy)]
enum RepeatUnit {
    Minute,
    Hour,
    Day,
}

type TaskLog = String;

/// Task planner service
pub struct PlanService {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    logs: Arc<RwLock<Vec<TaskLog>>>,
    running: Arc<RwLock<bool>>,
    tx: Option<mpsc::Sender<UiData>>,
}

impl PlanService {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            tx: None,
        }
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            tx: Some(tx),
        }
    }

    pub async fn send(&self, data: UiData) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(data).await;
        }
    }

    /// Start the Plan service (starts timers for all existing tasks)
    async fn start(&mut self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(ServiceError::AlreadyRunning);
        }
        *running = true;
        drop(running);

        // Re-launch timers for all loaded tasks
        let tasks = Arc::clone(&self.tasks);
        let mut guard = tasks.write().await;
        for (_, task) in guard.iter_mut() {
            Self::spawn_timer(task, Arc::clone(&self.logs), self.tx.clone());
        }

        info!("Plan service started");
        Ok(())
    }

    pub async fn update(&mut self) -> ServiceUpdateResult {
        let running = *self.running.read().await;
        if running {
            match self.stop().await {
                Ok(()) => ServiceUpdateResult::Stopped("Plan service stopped".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to stop: {}", e)),
            }
        } else {
            match self.start().await {
                Ok(()) => ServiceUpdateResult::Started("Plan service started".to_string()),
                Err(e) => ServiceUpdateResult::Error(format!("Failed to start: {}", e)),
            }
        }
    }

    /// Stop all task timers
    pub async fn destroy(&mut self) -> Result<()> {
        *self.running.write().await = false;
        Self::abort_all(&self.tasks).await;
        info!("Plan service destroyed");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.running.write().await = false;
        Self::abort_all(&self.tasks).await;
        info!("Plan service stopped");
        Ok(())
    }

    async fn abort_all(tasks: &Arc<RwLock<HashMap<String, Task>>>) {
        let mut guard = tasks.write().await;
        for (_, task) in guard.iter_mut() {
            if let Some(h) = task.handle.take() {
                h.abort();
            }
        }
    }

    /// Add a new task
    pub async fn add_task(&self, date: &str, time: &str, cycle: i32, unit: &str, msg: &str, override_conflict: bool) -> Result<bool> {
        if msg.is_empty() {
            return Err(ServiceError::Config("Message cannot be empty".into()));
        }

        let datetime = if let (Ok(d), Ok(t)) = (NaiveDate::parse_from_str(date, "%Y/%m/%d"), NaiveTime::parse_from_str(time, "%H:%M")) {
            NaiveDateTime::new(d, t).and_local_timezone(Local).unwrap()
        } else {
            Local::now()
        };

        let schedule = if cycle > 0 {
            let repeat_unit = match unit {
                "hour" => RepeatUnit::Hour,
                "day" => RepeatUnit::Day,
                _ => RepeatUnit::Minute,
            };
            Schedule::Repeating { datetime, cycle, unit: repeat_unit }
        } else {
            Schedule::Once { datetime }
        };

        let mut tasks = self.tasks.write().await;
        let existed = tasks.contains_key(msg);

        if existed && !override_conflict {
            return Err(ServiceError::Config(format!("Task '{}' already exists", msg)));
        }

        let mut task = Task::new(msg.to_string(), schedule);

        // Spawn timer if service is running
        if *self.running.read().await {
            Self::spawn_timer(&mut task, Arc::clone(&self.logs), self.tx.clone());
        }

        tasks.insert(msg.to_string(), task);
        info!("Added task: {}", msg);
        Ok(existed)
    }

    /// Remove a task by message
    pub async fn remove_task(&self, msg: &str) -> Result<()> {
        if msg.is_empty() {
            return Err(ServiceError::Config("Message cannot be empty".into()));
        }
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.remove(msg) {
            if let Some(h) = task.handle {
                h.abort();
            }
            info!("Removed task: {}", msg);
        }
        Ok(())
    }

    /// Spawn a timer for a task
    fn spawn_timer(task: &mut Task, logs: Arc<RwLock<Vec<TaskLog>>>, tx: Option<mpsc::Sender<UiData>>) {
        let title = task.title.clone();
        let schedule = task.schedule.clone();

        let logs = Arc::clone(&logs);

        let handle = tokio::spawn(async move {
            match schedule {
                Schedule::Once { datetime } => {
                    // Sleep until the scheduled time
                    let now = Local::now();
                    let delay = if datetime > now {
                        (datetime - now).to_std().unwrap_or(std::time::Duration::from_secs(0))
                    } else {
                        // Already past, trigger immediately
                        std::time::Duration::from_secs(0)
                    };
                    sleep_until(tokio::time::Instant::now() + delay).await;
                    trigger_task(&title, &logs, &tx).await;
                }
                Schedule::Repeating { datetime, cycle, unit } => {
                    let interval = match unit {
                        RepeatUnit::Minute => Duration::minutes(cycle as i64),
                        RepeatUnit::Hour => Duration::minutes((cycle as i64) * 60),
                        RepeatUnit::Day => Duration::days(cycle as i64),
                    };

                    // Calculate first trigger delay
                    let now = Local::now();
                    let mut next = datetime;

                    // If datetime is in the past, fast-forward to next future trigger
                    if next <= now {
                        let elapsed = now - next;
                        let intervals_passed = (elapsed.num_seconds() / interval.num_seconds()).max(0) + 1;
                        next = next + interval * intervals_passed as i32;
                    }

                    loop {
                        let delay = if next > now {
                            (next - now).to_std().unwrap_or(std::time::Duration::from_secs(0))
                        } else {
                            std::time::Duration::from_secs(0)
                        };
                        sleep_until(tokio::time::Instant::now() + delay).await;
                        trigger_task(&title, &logs, &tx).await;
                        next = next + interval;
                    }
                }
            }
        });

        task.handle = Some(handle.abort_handle());
    }
}

/// Trigger a task: send log + reminder
async fn trigger_task(title: &str, logs: &Arc<RwLock<Vec<TaskLog>>>, tx: &Option<mpsc::Sender<UiData>>) {
    let log_msg = format!("[{}] Task triggered: {}", Local::now().format("%H:%M:%S"), title);
    logs.write().await.push(log_msg.clone());
    info!("Task triggered: {}", title);

    if let Some(tx) = tx {
        let _ = tx.send(UiData::Log(Module::Plan, log_msg)).await;
        let _ = tx.send(UiData::PlanReminder(title.to_string())).await;
    }
}

impl Default for PlanService {
    fn default() -> Self {
        Self::new()
    }
}
