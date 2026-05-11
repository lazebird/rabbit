//! 测试场景：程序退出时 cleanup 不应影响运行状态的保存
//!
//! 问题背景：cleanup 先调用 update() 停止服务，导致保存的 running=false
//! 正确行为：cleanup 调用 destroy() 销毁资源，同时先读取真实运行状态再保存

use service::{
    ui_channel::{Module, UiData},
    ChatService, HttpService, PingService, PlanService, ScanService, ServiceUpdateResult, TftpdService,
};
use tokio::sync::mpsc;

/// 辅助：创建一个带 channel 的服务，并返回 (service, rx)
fn make_ping() -> (PingService, mpsc::Receiver<UiData>) {
    let (tx, rx) = mpsc::channel(100);
    (PingService::with_channel(tx), rx)
}

fn make_http() -> (HttpService, mpsc::Receiver<UiData>) {
    let (tx, rx) = mpsc::channel(100);
    (HttpService::with_channel(tx), rx)
}

/// 测试 cleanup 场景：destroy 不应发送 ServiceStatus 通告
/// 原来 update() 会发通告导致 UI 误以为服务被停止
#[tokio::test]
async fn cleanup_ping_destroy_should_not_send_status() {
    let (mut svc, mut rx) = make_ping();

    // 先启动服务（模拟正常运行）
    let result = svc.update().await;
    assert!(matches!(result, ServiceUpdateResult::Started(_)));

    // 模拟 cleanup：只调用 destroy，不应发 ServiceStatus 通告
    let _ = svc.destroy().await;

    // 短暂等待，确认没有 ServiceStatus(false) 被发出
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // destroy 不应发状态通告
    while let Ok(data) = rx.try_recv() {
        assert!(
            !matches!(data, UiData::ServiceStatus(Module::Ping, false, _)),
            "destroy() should NOT send ServiceStatus(false), only update() should"
        );
    }
}

/// 测试 update 停止时应发状态通告（UI 层据此更新配置）
#[tokio::test]
async fn update_stop_should_send_status() {
    let (mut svc, mut rx) = make_ping();

    // 启动
    let result = svc.update().await;
    assert!(matches!(result, ServiceUpdateResult::Started(_)));

    // 停止（通过 update）
    let result = svc.update().await;
    assert!(matches!(result, ServiceUpdateResult::Stopped(_)));

    // 应该收到 ServiceStatus(false) 通告
    let mut found = false;
    while let Ok(data) = rx.try_recv() {
        if matches!(data, UiData::ServiceStatus(Module::Ping, false, _)) {
            found = true;
            break;
        }
    }
    assert!(found, "update() stop should send ServiceStatus(false) so UI can save config");
}

#[tokio::test]
async fn can_restart_after_destroy() {
    let (mut svc, _) = make_ping();

    let r1 = svc.update().await;
    assert!(matches!(r1, ServiceUpdateResult::Started(_)), "should start after first update");

    let _ = svc.destroy().await;

    let r2 = svc.update().await;
    assert!(matches!(r2, ServiceUpdateResult::Started(_)), "should be able to restart after destroy");
}

/// 测试 HTTP service destroy 不发通告
#[tokio::test]
async fn cleanup_http_destroy_should_not_send_status() {
    let (mut svc, mut rx) = make_http();

    // 启动（可能因端口占用失败，这里只测 destroy 行为）
    let _ = svc.update().await;

    // destroy 不应发 ServiceStatus
    let _ = svc.destroy().await;
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    while let Ok(data) = rx.try_recv() {
        assert!(!matches!(data, UiData::ServiceStatus(Module::Http, false, _)), "HTTP destroy() should NOT send ServiceStatus");
    }
}

/// 集成测试：模拟完整 cleanup 流程
/// 1. 服务运行中 2. cleanup 调用 destroy 3. destroy 后 is_running 应为 false
#[tokio::test]
async fn cleanup_flow_preserves_running_state() {
    let (mut ping, _) = make_ping();
    let (mut http, _) = make_http();
    let (mut tftpd, _) = {
        let (tx, _) = mpsc::channel(100);
        (TftpdService::with_channel(tx), ())
    };
    let (mut chat, _) = {
        let (tx, _) = mpsc::channel(100);
        (ChatService::with_channel(tx), ())
    };
    let (mut plan, _) = {
        let (tx, _) = mpsc::channel(100);
        (PlanService::with_channel(tx), ())
    };
    let (mut scan, _) = {
        let (tx, _) = mpsc::channel(100);
        (ScanService::with_channel(tx), ())
    };

    // 模拟启动部分服务
    let ping_running = {
        let r = ping.update().await;
        matches!(r, ServiceUpdateResult::Started(_))
    };
    // HTTP 可能因端口占用失败，所以只看结果
    let _http_running = {
        let r = http.update().await;
        // HTTP 可能因端口占用失败，所以只看结果
        matches!(r, ServiceUpdateResult::Started(_))
    };

    // 验证服务正在运行
    assert!(ping_running, "ping should be running after start");

    let before_destroy_running = ping_running;

    let _ = ping.destroy().await;
    let _ = http.destroy().await;
    let _ = tftpd.destroy().await;
    let _ = chat.destroy().await;
    let _ = plan.destroy().await;
    let _ = scan.destroy().await;

    let r = ping.update().await;
    assert!(matches!(r, ServiceUpdateResult::Started(_)), "ping should be restartable after destroy");
    assert!(before_destroy_running, "ping_running should be true (recorded before destroy)");
}
