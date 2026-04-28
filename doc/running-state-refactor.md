# 运行状态接口冗余分析报告

## 设计原则

1. **持久化运行状态**: 以配置模块 (`rabbit-models/config.rs`) 提供的 `running` 字段为准
2. **非持久化运行状态**: 使用内部局部变量记录和跟踪
3. **运行状态不应该有独立的 API 接口**: 不应该有专门的 getter/setter

## 完成状态 ✅

| 阶段 | 状态 | 说明 |
|------|------|------|
| Phase 1: ViewModel 精简 | ✅ 完成 | 删除 5 字段 + 10 API |
| Phase 2: App 清理 | ✅ 完成 | 更新 20 处引用 |
| Phase 3: 验证 | ✅ 完成 | cargo check 通过 |

## 执行概要

### Phase 1: ViewModel 精简
- [x] 删除 `AppViewModel` 中的 5 个 `running` 字段
- [x] 删除 10 个 getter/setter 方法

### Phase 2: App 清理
- [x] 更新 startup 时从配置读取运行状态 (`config.modules.get_bool(...)`)
- [x] 移除 `view_model.set_*_running()` 调用 (14 处)
- [x] 移除 `view_model.is_*_running()` 调用 (6 处)
- [x] 保留 `ui_state.set_*_running()` (用于 UI 刷新)
- [x] 更新 `handle_ui_data` 中的引用

## 修改的文件

| 文件 | 修改量 |
|------|--------|
| `rabbit-app/src/view_model.rs` | 98 → ~50 行 (删除 48 行) |
| `rabbit-app/src/app.rs` | 修改 ~20 处引用 |

## 回滚计划

```bash
git checkout -- crates/rabbit-app/src/view_model.rs
git checkout -- crates/rabbit-app/src/app.rs
```