# Rabbit 配置局部更新方案

## 需求

配置更新/保存需要支持基于模块或局部更新，避免无关配置被覆盖/重置。

---

## 方案：字段更新器模式

### 设计思路

每次只更新特定模块或字段，保留其他配置不变。

### 接口设计

```rust
// 方式 1：专用方法（推荐）
pub fn update_ping(&mut self, config: PingConfig) -> Result<()>
pub fn update_http(&mut self, config: HttpConfig) -> Result<()>

// 方式 2：字段更新器
pub fn update_field<F>(&mut self, updater: F) -> Result<()>
where F: FnOnce(&mut AppConfig)
```

### 方式 1：专用方法

```rust
pub fn update_ping(&mut self, ping: PingConfig) -> Result<()> {
    self.config.modules.ping = ping;
    save_config(&self.config)
}

pub fn update_http(&mut self, http: HttpConfig) -> Result<()> {
    self.config.modules.http = http;
    save_config(&self.config)
}
```

调用示例：
```rust
let mut ping_cfg = vm.get_config().modules.ping.clone();
ping_cfg.target = "8.8.8.8".to_string();
vm.update_ping(ping_cfg)?;
```

### 方式 2：字段更新器

```rust
pub fn update_and_save<F>(&mut self, updater: F) -> Result<()>
where
    F: FnOnce(&mut AppConfig),
{
    updater(&mut self.config);
    save_config(&self.config)
}
```

调用示例：
```rust
vm.update_and_save(|cfg| {
    cfg.modules.ping.target = "8.8.8.8".to_string();
    cfg.modules.ping.interval = 500;
})?;
```

---

## 方案对比

| 方案 | 优点 | 缺点 |
|------|------|------|
| 专用方法 | 清晰、类型安全 | 模块多时方法变多 |
| 字段更新器 | 灵活、单一方法 | 需闭包处理 |
| section+map | 动态可扩展 | 需要字符串匹配 |

---

## 推荐：专用方法 + 更新器组合

```rust
impl AppViewModel {
    fn save(&mut self) -> Result<()> {
        save_config(&self.config)
    }

    pub fn update_and_save<F>(&mut self, updater: F) -> Result<()>
    where F: FnOnce(&mut AppConfig) {
        updater(&mut self.config);
        self.save()
    }

    pub fn update_ping(&mut self, target: String, interval: i32) -> Result<()> {
        self.update_and_save(|cfg| {
            cfg.modules.ping.target = target;
            cfg.modules.ping.interval = interval;
        })
    }
}
```

---

文档版本：1.0
创建日期：2026-04-23