//! Task Planner Service

use crate::{ui_channel::UiData, Result, ServiceError, ServiceUpdateResult};

use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{error, info};

/// Internal Task model
#[derive(Debug, Clone)]
struct Task {
    pub title: String,
    pub schedule: Schedule,
    pub enabled: bool,
    pub state: TaskState,
    pub snooze_until: Option<DateTime<Local>>,
    pub last_triggered: Option<DateTime<Local>>,
}

impl Task {
    pub fn new(title: String, schedule: Schedule) -> Self {
        Self {
            title,
            schedule,
            enabled: true,
            state: TaskState::Pending,
            snooze_until: None,
            last_triggered: None,
        }
    }

    pub fn is_snoozed(&self) -> bool {
        if let Some(snooze_until) = self.snooze_until {
            Local::now() < snooze_until
        } else {
            false
        }
    }

    pub fn trigger(&mut self) {
        self.state = TaskState::Triggered;
        self.last_triggered = Some(Local::now());
    }

    pub fn reset_for_next_trigger(&mut self) {
        self.state = TaskState::Pending;
        self.snooze_until = None;
        self.last_triggered = Some(Local::now());
    }
}

#[derive(Debug, Clone)]
enum Schedule {
    Once { datetime: DateTime<Local> },
    Repeating { datetime: DateTime<Local>, cycle: i32, unit: RepeatUnit },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepeatUnit {
    Minute,
    Hour,
    Day,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskState {
    Pending,
    Triggered,
    Acknowledged,
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

    /// 内部启动 Plan 服务
    async fn start(&mut self) -> Result<()> {
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

    /// 程序退出时调用，销毁资源，不发状态通告
    pub async fn destroy(&mut self) -> Result<()> {
        *self.running.write().await = false;
        info!("Plan service destroyed");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.running.write().await = false;
        info!("Plan service stopped");
        Ok(())
    }

    /// Add a new task with simple parameters
    pub async fn add_task(&self, date: &str, time: &str, cycle: i32, unit: &str, msg: &str) -> Result<()> {
        // Parse datetime
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

        let id = format!("task-{}", uuid::Uuid::new_v4());
        let task = Task::new(msg.to_string(), schedule);

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

    /// Check and trigger tasks
    async fn check_tasks(tasks: &Arc<RwLock<HashMap<String, Task>>>, logs: &Arc<RwLock<Vec<TaskLog>>>, tx: &Option<mpsc::Sender<UiData>>) {
        let now = Local::now();

        // 收集需要触发的任务 ID（只读锁）
        let task_ids: Vec<String> = tasks
            .read()
            .await
            .iter()
            .filter(|(_, t)| t.enabled && !t.is_snoozed() && t.state != TaskState::Acknowledged && Self::should_trigger(&t.schedule, now))
            .map(|(id, _)| id.clone())
            .collect();

        if task_ids.is_empty() {
            return;
        }

        // 获取写锁，原地修改任务
        let mut tasks_guard = tasks.write().await;
        for id in task_ids {
            if let Some(task) = tasks_guard.get_mut(&id) {
                task.trigger();

                if let Err(e) = rabbit_platform::notification::show_task_reminder(&task.title, None) {
                    error!("Failed to show notification: {}", e);
                }

                logs.write().await.push(task.title.clone());

                if let Some(ref tx) = tx {
                    let _ = tx.send(UiData::PlanReminder(task.title.clone())).await;
                }

                if let Schedule::Repeating { .. } = task.schedule {
                    task.reset_for_next_trigger();
                }
            }
        }
    }

    /// Check if a schedule should trigger at given time
    fn should_trigger(schedule: &Schedule, now: chrono::DateTime<chrono::Local>) -> bool {
        match schedule {
            Schedule::Once { datetime } => {
                let diff = (*datetime - now).num_seconds();
                (0..30).contains(&diff)
            }
            Schedule::Repeating { datetime, cycle, unit } => {
                let interval_secs = match unit {
                    RepeatUnit::Minute => *cycle as i64 * 60,
                    RepeatUnit::Hour => *cycle as i64 * 3600,
                    RepeatUnit::Day => *cycle as i64 * 86400,
                };
                let elapsed = (now - *datetime).num_seconds();
                if elapsed < 0 {
                    false
                } else {
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
