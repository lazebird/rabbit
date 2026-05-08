//! Task Planner Service
//!
//! Precise timer: sleep_until next trigger time.
//! Timer expires -> scan tasks -> trigger due ones -> recalculate.
//! No polling. Recalculates on task add/remove/trigger.

use crate::{ui_channel::UiData, Module, Result, ServiceError, ServiceUpdateResult};

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Notify, RwLock};
use tokio::time::{sleep_until, Instant};
use tracing::info;

struct Task {
    title: String,
    schedule: Schedule,
}

impl Task {
    fn new(title: String, schedule: Schedule) -> Self {
        Self { title, schedule }
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

impl RepeatUnit {
    fn to_duration(self, cycle: i32) -> Duration {
        match self {
            Self::Minute => Duration::minutes(cycle as i64),
            Self::Hour => Duration::minutes((cycle as i64) * 60),
            Self::Day => Duration::days(cycle as i64),
        }
    }
}

type TaskLog = String;

pub struct PlanService {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    logs: Arc<RwLock<Vec<TaskLog>>>,
    running: Arc<RwLock<bool>>,
    tx: Option<mpsc::Sender<UiData>>,
    timer_notify: Arc<Notify>,
    timer_handle: Option<tokio::task::AbortHandle>,
}

impl PlanService {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            tx: None,
            timer_notify: Arc::new(Notify::new()),
            timer_handle: None,
        }
    }

    pub fn with_channel(tx: mpsc::Sender<UiData>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            tx: Some(tx),
            timer_notify: Arc::new(Notify::new()),
            timer_handle: None,
        }
    }

    pub async fn send(&self, data: UiData) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(data).await;
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(ServiceError::AlreadyRunning);
        }
        *running = true;
        drop(running);

        self.restart_timer();
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

    pub async fn destroy(&mut self) -> Result<()> {
        *self.running.write().await = false;
        self.abort_timer();
        info!("Plan service destroyed");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.running.write().await = false;
        self.abort_timer();
        info!("Plan service stopped");
        Ok(())
    }

    fn abort_timer(&mut self) {
        if let Some(handle) = self.timer_handle.take() {
            handle.abort();
        }
    }

    fn restart_timer(&mut self) {
        self.abort_timer();

        let tasks = Arc::clone(&self.tasks);
        let logs = Arc::clone(&self.logs);
        let running = Arc::clone(&self.running);
        let tx = self.tx.clone();
        let timer_notify = Arc::clone(&self.timer_notify);

        let handle = tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                let (next_instant, due_tasks) = Self::calc_next(&tasks).await;

                match next_instant {
                    Some(instant) => {
                        tokio::select! {
                            _ = sleep_until(instant) => {
                                if !due_tasks.is_empty() {
                                    for title in &due_tasks {
                                        trigger_task(title, &logs, &tx).await;
                                    }

                                    let mut tasks_guard = tasks.write().await;
                                    for title in &due_tasks {
                                        if let Some(task) = tasks_guard.get_mut(title) {
                                            match &mut task.schedule {
                                                Schedule::Once { .. } => {
                                                    tasks_guard.remove(title);
                                                }
                                                Schedule::Repeating { datetime, cycle, unit } => {
                                                    let interval = unit.to_duration(*cycle);
                                                    let now = Local::now();
                                                    if *datetime <= now {
                                                        let elapsed = now - *datetime;
                                                        let intervals = (elapsed.num_seconds() / interval.num_seconds()).max(0) as i32;
                                                        *datetime += interval * (intervals + 1);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    drop(tasks_guard);
                                }
                                continue;
                            }
                            _ = timer_notify.notified() => {
                                continue;
                            }
                        }
                    }
                    None => {
                        timer_notify.notified().await;
                        continue;
                    }
                }
            }
        });

        self.timer_handle = Some(handle.abort_handle());
    }

    /// Calculate next trigger time
    /// - Past one-time tasks: ignored (not triggered)
    /// - Past repeating tasks: calculate next FUTURE trigger time
    async fn calc_next(
        tasks: &Arc<RwLock<HashMap<String, Task>>>,
    ) -> (Option<Instant>, Vec<String>) {
        let tasks_guard = tasks.read().await;
        let now = Local::now();
        let mut earliest: Option<(DateTime<Local>, Vec<String>)> = None;

        for (_, task) in tasks_guard.iter() {
            let next_time = match &task.schedule {
                Schedule::Once { datetime } => {
                    if *datetime > now {
                        Some(*datetime)
                    } else {
                        // Past one-time task: ignore
                        None
                    }
                }
                Schedule::Repeating { datetime, cycle, unit } => {
                    let interval = unit.to_duration(*cycle);
                    let mut next = *datetime;

                    // Calculate next FUTURE trigger time
                    if next <= now {
                        let elapsed = now - next;
                        let intervals = (elapsed.num_seconds() / interval.num_seconds()).max(0) as i32;
                        next = *datetime + interval * (intervals + 1);
                    }
                    Some(next)
                }
            };

            if let Some(nt) = next_time {
                match &mut earliest {
                    Some((earliest_time, titles)) => {
                        if nt < *earliest_time {
                            *earliest_time = nt;
                            *titles = vec![task.title.clone()];
                        } else if nt == *earliest_time {
                            titles.push(task.title.clone());
                        }
                    }
                    None => {
                        earliest = Some((nt, vec![task.title.clone()]));
                    }
                }
            }
        }

        match earliest {
            Some((dt, titles)) => {
                let now = Local::now();
                let delay = if dt > now {
                    (dt - now).to_std().unwrap_or(std::time::Duration::from_secs(0))
                } else {
                    std::time::Duration::from_secs(0)
                };
                (Some(Instant::now() + delay), titles)
            }
            None => (None, vec![]),
        }
    }

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

        let task = Task::new(msg.to_string(), schedule);
        tasks.insert(msg.to_string(), task);
        info!("Added task: {}", msg);

        self.timer_notify.notify_one();

        Ok(existed)
    }

    pub async fn remove_task(&self, msg: &str) -> Result<()> {
        if msg.is_empty() {
            return Err(ServiceError::Config("Message cannot be empty".into()));
        }
        let mut tasks = self.tasks.write().await;
        if tasks.remove(msg).is_some() {
            info!("Removed task: {}", msg);
            self.timer_notify.notify_one();
        }
        Ok(())
    }
}

async fn trigger_task(title: &str, logs: &Arc<RwLock<Vec<TaskLog>>>, tx: &Option<mpsc::Sender<UiData>>) {
    let log_msg = format!("Task triggered: {}", title);
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
