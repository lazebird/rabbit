//! Task Planner Service

use crate::{Result, ServiceError, ServiceUpdateResult, ui_channel::{UiData, Module}};
use chrono::{Datelike, Local};
use rabbit_models::plan::{Schedule, Task, TaskLog, TaskState, RepeatUnit};
use rabbit_platform::notification::show_task_reminder;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{error, info};

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

    /// Initialize the service
    pub async fn init(&mut self) -> Result<()> {
        info!("Plan service initialized");
        Ok(())
    }

    /// Start the scheduler
    pub async fn start(&mut self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(ServiceError::AlreadyRunning);
        }
        *running = true;
        drop(running);

        let tasks = Arc::clone(&self.tasks);
        let logs = Arc::clone(&self.logs);
        let running = Arc::clone(&self.running);
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(30));

            while *running.read().await {
                check_interval.tick().await;
                Self::check_tasks(&tasks, &logs, &tx).await;
            }
        });

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

    pub fn is_running(&self) -> bool {
        *self.running.blocking_read()
    }

    pub async fn stop(&mut self) -> Result<()> {
        *self.running.write().await = false;
        info!("Plan service stopped");
        Ok(())
    }

    /// Add a new task
    pub async fn add_task(&self, task: Task) -> Result<()> {
        let id = task.id.clone();
        self.tasks.write().await.insert(id.clone(), task);
        info!("Added task: {}", id);
        Ok(())
    }

    /// Remove a task
    pub async fn remove_task(&self, id: &str) -> Result<()> {
        self.tasks.write().await.remove(id);
        info!("Removed task: {}", id);
        Ok(())
    }

    /// Update a task
    pub async fn update_task(&self, task: Task) -> Result<()> {
        let id = task.id.clone();
        self.tasks.write().await.insert(id.clone(), task);
        info!("Updated task: {}", id);
        Ok(())
    }

    /// Get a task by ID
    pub async fn get_task(&self, id: &str) -> Option<Task> {
        self.tasks.read().await.get(id).cloned()
    }

    /// Get all tasks
    pub async fn get_all_tasks(&self) -> Vec<Task> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// Get enabled tasks
    pub async fn get_enabled_tasks(&self) -> Vec<Task> {
        self.tasks.read().await
            .values()
            .filter(|t| t.enabled)
            .cloned()
            .collect()
    }

    /// Acknowledge a triggered task
    pub async fn acknowledge_task(&self, id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.acknowledge();
            info!("Task acknowledged: {}", id);
            
            // Add log entry
            let log = TaskLog {
                task_id: id.to_string(),
                triggered_at: task.last_triggered.unwrap_or_else(|| Local::now()),
                acknowledged_at: Some(Local::now()),
            };
            self.logs.write().await.push(log);
        }
        Ok(())
    }

    /// Snooze a task
    pub async fn snooze_task(&self, id: &str, minutes: u32) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.snooze(minutes as i64);
            info!("Task {} snoozed for {} minutes", id, minutes);
        }
        Ok(())
    }

    /// Reset a task to pending state
    pub async fn reset_task(&self, id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            task.reset();
            info!("Task reset: {}", id);
        }
        Ok(())
    }

    /// Get task logs
    pub async fn get_logs(&self) -> Vec<TaskLog> {
        self.logs.read().await.clone()
    }

    /// Check and trigger tasks
    async fn check_tasks(
        tasks: &Arc<RwLock<HashMap<String, Task>>>,
        logs: &Arc<RwLock<Vec<TaskLog>>>,
        tx: &Option<mpsc::Sender<UiData>>,
    ) {
        let now = Local::now();
        let tasks_to_trigger: Vec<(String, Task)> = tasks.read().await
            .iter()
            .filter(|(_, t)| {
                t.enabled 
                    && !t.is_snoozed() 
                    && t.state != TaskState::Acknowledged
                    && Self::should_trigger(&t.schedule, now)
            })
            .map(|(id, t)| (id.clone(), t.clone()))
            .collect();

        for (id, mut task) in tasks_to_trigger {
            task.trigger();
            
            if let Err(e) = show_task_reminder(&task) {
                error!("Failed to show notification: {}", e);
            }

            logs.write().await.push(TaskLog {
                task_id: task.id.clone(),
                triggered_at: now,
                acknowledged_at: None,
            });

            if let Some(ref tx) = tx {
                let _ = tx.send(UiData::PlanReminder(task.title.clone())).await;
            }

            if matches!(task.schedule, Schedule::Repeating { .. }) {
                task.reset_for_next_trigger();
            }

            tasks.write().await.insert(id, task);
        }
    }

    /// Check if a schedule should trigger at given time
    fn should_trigger(schedule: &Schedule, now: chrono::DateTime<chrono::Local>) -> bool {
        match schedule {
            Schedule::Once { datetime } => {
                let diff = (*datetime - now).num_seconds();
                diff >= 0 && diff < 30
            }
            Schedule::Daily { time } => {
                let now_time = now.time();
                let diff = (now_time - *time).num_seconds();
                diff >= 0 && diff < 30
            }
            Schedule::Weekly { day, time } => {
                let weekday = now.weekday();
                let matches_day = match day {
                    rabbit_models::plan::WeekDay::Monday => weekday == chrono::Weekday::Mon,
                    rabbit_models::plan::WeekDay::Tuesday => weekday == chrono::Weekday::Tue,
                    rabbit_models::plan::WeekDay::Wednesday => weekday == chrono::Weekday::Wed,
                    rabbit_models::plan::WeekDay::Thursday => weekday == chrono::Weekday::Thu,
                    rabbit_models::plan::WeekDay::Friday => weekday == chrono::Weekday::Fri,
                    rabbit_models::plan::WeekDay::Saturday => weekday == chrono::Weekday::Sat,
                    rabbit_models::plan::WeekDay::Sunday => weekday == chrono::Weekday::Sun,
                };

                if !matches_day {
                    return false;
                }

                let now_time = now.time();
                let diff = (now_time - *time).num_seconds();
                diff >= 0 && diff < 30
            }
            Schedule::Repeating { datetime, cycle, unit } => {
                // Calculate the interval in seconds
                let interval_secs = match unit {
                    RepeatUnit::Minute => *cycle as i64 * 60,
                    RepeatUnit::Hour => *cycle as i64 * 3600,
                    RepeatUnit::Day => *cycle as i64 * 86400,
                };
                
                // Time elapsed since the start datetime
                let elapsed = (now - *datetime).num_seconds();
                
                // Trigger if we've passed the start time and are at an interval boundary
                if elapsed < 0 {
                    false // Haven't reached start time yet
                } else {
                    // Check if we're within 30 seconds of an interval boundary
                    let remainder = elapsed % interval_secs;
                    remainder < 30
                }
            }
        }
    }
}

impl Default for PlanService {
    fn default() -> Self {
        Self::new()
    }
}
