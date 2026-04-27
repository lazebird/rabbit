# UI Refresh Refactoring Plan

## 目标
将所有 crate 中的 `static mut` 声明转换为使用 `parking_lot::Mutex` 或 `std::sync::atomic::AtomicBool`，消除 Rust 1.95+ 不推荐的 unsafe static mut 警告。

## 完成状态 ✅

| Phase | 状态 | 说明 |
|-------|------|------|
| Phase 1: ui_refresh.rs | ✅ 完成 | 20 个 static mut 已转换 |
| Phase 2: ui_events.rs | ✅ 完成 | 1 个 static mut |
| Phase 3: ui_state.rs | ✅ 完成 | 2 个 static mut |
| Phase 4: platform 清理 | ✅ 完成 | 7 个 unused 函数已添加 #![allow] |
| Phase 5: 验证 | ✅ 完成 | 无警告 |

## 完成的变更详情

### ui_refresh.rs (Phase 1)
- Convert `static mut DISPLAYS/BROWSER` → `static Mutex<Option<...>>`
- Convert 所有按钮 (HTTP_BTN, PING_BTN, SCAN_BTN, TFTPD_BTN, TFTPC_BTN, PLAN_BTN, CHAT_BTN, SETTINGS_BTN) → `Mutex<Option<...>>`
- Convert `REFRESH_RUNNING` → `AtomicBool`
- 添加 getter 函数 (get_http_btn, get_ping_btn, etc.)
- 修复 mutable variable 警告

### ui_events.rs (Phase 2)
- Convert `static mut GLOBAL_EVENT_SENDER` → `static Mutex<Option<...>>`

### ui_state.rs (Phase 3)
- Convert `static mut MAIN_WINDOW` → `static Mutex<Option<...>>`
- Convert `static mut GLOBAL_UI_STATE` → `static Mutex<Option<Arc<...>>>`
- 使用 parking_lot::Mutex 避免与 std::sync::Mutex 冲突

### elevation.rs (Phase 4)
- 添加 `#![allow(dead_code)]` 到文件顶部
- 移除未使用的 `use std::process::exit` 导入
- 改用 `std::process::exit` 显式调用

## 验证结果

```bash
$ cargo build
   Compiling rabbit-platform v0.1.0
   Compiling rabbit-app v0.1.0
   Compiling rabbit-core v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.65s
```

✅ **所有警告已清除**

## 修改的文件清单

1. `crates/rabbit-app/src/ui/ui_refresh.rs` - 完整重构
2. `crates/rabbit-app/src/ui_events.rs` - static mut 转换
3. `crates/rabbit-app/src/ui_state.rs` - static mut 转换
4. `crates/rabbit-platform/src/elevation.rs` - 添加 allow 注释

## 回滚计划
如果需要回滚:
```bash
git checkout -- crates/rabbit-app/src/ui/ui_refresh.rs
git checkout -- crates/rabbit-app/src/ui_events.rs
git checkout -- crates/rabbit-app/src/ui_state.rs
git checkout -- crates/rabbit-platform/src/elevation.rs
```