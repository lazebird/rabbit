# Rabbit 项目测试指南

## 测试脚本

本项目提供两个测试脚本：

### 1. `test.sh` - 快速测试脚本（推荐）

简洁高效的测试脚本，执行所有关键测试步骤。

**使用方法：**
```bash
./test.sh
```

**执行步骤：**
1. ✓ 检查 Rust 工具链
2. ✓ 编译检查（cargo check）
3. ✓ 单元测试（cargo test --lib）
4. ✓ 集成测试（cargo test --test integration_tests）
5. ✓ 完整测试套件（cargo test）
6. ✓ Release 构建（cargo build --release）

**特点：**
- ⚡ 快速执行（约 1-2 分钟）
- 🎨 彩色输出，易于阅读
- 🛑 错误时立即停止（set -e）
- 📊 显示测试统计信息

### 2. `run-tests.sh` - 完整测试脚本

详细的测试脚本，包含格式检查和警告统计。

**使用方法：**
```bash
./run-tests.sh
```

**额外功能：**
- 📝 代码格式检查（cargo fmt --check）
- ⚠️ 编译警告统计
- 📄 详细的测试输出

## 手动运行测试

### 运行所有测试
```bash
cargo test
```

### 仅运行单元测试
```bash
cargo test --lib
```

### 仅运行集成测试
```bash
cargo test --test integration_tests
```

### 运行特定测试
```bash
cargo test test_ping_target_creation
```

### 运行测试并显示输出
```bash
cargo test -- --nocapture
```

## 测试覆盖

当前测试覆盖情况：

| 测试类型 | 数量 | 状态 |
|---------|------|------|
| 单元测试 | 24 | ✅ 通过 |
| 集成测试 | 75 | ✅ 通过 |
| **总计** | **99** | **✅ 通过** |

### 集成测试覆盖模块

- ✅ Ping 模块 (9 个测试)
- ✅ HTTP 模块 (10 个测试)
- ✅ TFTP 模块 (11 个测试)
- ✅ Chat 模块 (8 个测试)
- ✅ Plan 模块 (7 个测试)
- ✅ Scan 模块 (6 个测试)
- ✅ 配置模块 (7 个测试)
- ✅ UI 事件系统 (3 个测试)
- ✅ ViewModel 模块 (7 个测试)
- ✅ 边界和异常处理 (7 个测试)

## CI/CD 集成

可以在 GitHub Actions 中使用以下配置：

```yaml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: ./test.sh
```

## 故障排除

### 测试失败时

1. 查看详细输出：
   ```bash
   cargo test -- --nocapture
   ```

2. 运行特定失败的测试：
   ```bash
   cargo test <test_name>
   ```

3. 清理并重新构建：
   ```bash
   cargo clean
   cargo test
   ```

### 格式问题

如果格式检查失败：
```bash
cargo fmt
```

### 编译警告

查看警告详情：
```bash
cargo clippy
```

## 测试脚本特性

两个脚本都具备：

- ✅ **错误立即停止** - 使用 `set -e` 确保任何失败立即退出
- ✅ **彩色输出** - 清晰的状态指示
- ✅ **进度显示** - 实时显示测试进度
- ✅ **统计汇总** - 显示测试总数和通过率
- ✅ **时间记录** - 记录开始和完成时间
