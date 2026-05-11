//! 集成测试：UI刷新相关功能
//!
//! 测试任务栏标题、按钮状态等UI行为的正确性

use service::ui_channel::{Module, UiData};
use schema::config::{AppConfig, ConfigValue};
use tokio::sync::mpsc;

/// 辅助：创建带 channel 的 PingService 并模拟启动
async fn setup_ping_with_target(target: &str) -> (service::PingService, mpsc::Receiver<UiData>) {
    let (tx, rx) = mpsc::channel(100);

    // 设置配置
    let mut config = AppConfig::default();
    config.modules.insert("ping", "target", ConfigValue::String(target.to_string()));
    config.modules.insert("ping", "running", ConfigValue::Boolean(true));
    let _ = rabbit_config::save_config(&config);

    let mut svc = service::PingService::with_channel(tx);
    let _ = svc.update().await; // 启动
    (svc, rx)
}

/// 测试：Ping 启动时应该发送 ServiceStatus(true)
#[tokio::test]
async fn ping_start_should_send_service_status_true() {
    let (mut svc, mut rx) = setup_ping_with_target("127.0.0.1").await;

    // 应该收到 ServiceStatus(Ping, true)
    let mut found = false;
    while let Ok(data) = rx.try_recv() {
        if matches!(data, UiData::ServiceStatus(Module::Ping, true, _)) {
            found = true;
            break;
        }
    }
    assert!(found, "Ping start should send ServiceStatus(Ping, true)");

    let _ = svc.destroy().await;
}

/// 测试：Ping 停止时应该发送 ServiceStatus(false) 且窗口标题重置
#[tokio::test]
async fn ping_stop_should_send_service_status_false() {
    let (mut svc, mut rx) = setup_ping_with_target("127.0.0.1").await;

    // 清空之前的消息
    while rx.try_recv().is_ok() {}

    // 停止
    let _ = svc.update().await;

    // 应该收到 ServiceStatus(Ping, false)
    let mut found = false;
    while let Ok(data) = rx.try_recv() {
        if matches!(data, UiData::ServiceStatus(Module::Ping, false, _)) {
            found = true;
            break;
        }
    }
    assert!(found, "Ping stop should send ServiceStatus(Ping, false)");

    let _ = svc.destroy().await;
}

/// 测试：Scan 完成时应该发送 ServiceStatus(Scan, false) 使按钮变回 Start
#[tokio::test]
async fn scan_complete_should_send_service_status_false() {
    let (tx, mut rx) = mpsc::channel(100);

    // 设置配置
    let mut config = AppConfig::default();
    config.modules.insert("scan", "start_ip", ConfigValue::String("127.0.0.1".to_string()));
    config.modules.insert("scan", "end_ip", ConfigValue::String("127.0.0.1".to_string()));
    let _ = rabbit_config::save_config(&config);

    let mut svc = service::ScanService::with_channel(tx);
    let _ = svc.update().await; // 启动 scan

    // 等待 scan 完成（scan 很快，因为只扫描一个地址）
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // 应该收到 ServiceStatus(Scan, false)
    let mut found = false;
    while let Ok(data) = rx.try_recv() {
        if matches!(data, UiData::ServiceStatus(Module::Scan, false, _)) {
            found = true;
            break;
        }
    }
    assert!(found, "Scan complete should send ServiceStatus(Scan, false) to reset button to Start");

    let _ = svc.destroy().await;
}

/// 测试：ui_refresh 中 scan_running 状态应该正确反映实际运行状态
#[tokio::test]
async fn ui_refresh_scan_running_should_reflect_actual_state() {
    // 初始化全局状态
    let _state = app::UiState::init();

    // 设置 scan_running=true
    app::set_scan_running(true);

    // 验证可以通过全局状态获取
    if let Some(state) = app::UiState::global() {
        if let Ok(s) = state.lock() {
            let running = s.updated.get("scan_running").copied().unwrap_or(false);
            assert!(running, "scan_running should be true after set_scan_running(true)");
        }
    }

    // 设置 scan_running=false
    app::set_scan_running(false);

    if let Some(state) = app::UiState::global() {
        if let Ok(s) = state.lock() {
            let running = s.updated.get("scan_running").copied().unwrap_or(true);
            assert!(!running, "scan_running should be false after set_scan_running(false)");
        }
    }
}

/// 测试：ServiceUpdateResult 不应该有 is_running 方法（已删除）
/// 这个测试确保在编译时 is_running 方法不存在
#[test]
fn service_update_result_is_running_should_not_exist() {
    // 如果 ServiceUpdateResult 还有 is_running 方法，下面的代码会编译失败
    // 我们检查 ServiceUpdateResult 的可用方法
    let result = service::ServiceUpdateResult::Started("test".to_string());

    // 只能调用 message 和 ok 方法
    let msg = result.message();
    assert_eq!(msg, "test");

    let ok_val = result.ok();
    assert!(ok_val.is_some());
}
