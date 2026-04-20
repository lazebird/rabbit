# 变更日志方案对比与推荐

## 概述

本文档评估了 Rabbit 项目的主流变更日志（changelog）生成方案，从需求匹配度、流行度、维护活跃度和集成复杂度等维度进行对比分析。

## 需求分析

基于 Rabbit 项目的需求：

1. **基于 Git 的版本管理** - 根据版本/标签之间的 git 提交历史生成变更日志
2. **支持约定式提交（Conventional Commits）** - 解析标准化的提交信息（feat、fix、docs、refactor、perf、chore）
3. **基于标签的发布** - 支持通过 git 标签进行发布跟踪
4. **易于自动化** - 能够集成到 release.sh 脚本中
5. **独立实现** - 变更日志逻辑不应与业务代码耦合
6. **跨平台** - 支持 Linux、macOS 和 Windows（CI/CD）

## 主流方案对比

### 1. git-cliff

| 指标 | 值 |
|------|-----|
| **GitHub Stars** | 11,730 |
| **Forks** | 286 |
| **语言** | Rust |
| **许可证** | MIT / Apache-2.0 |
| **最后更新** | 2026 年 4 月（活跃） |
| **二进制大小** | 单个静态二进制文件（约 3-5 MB） |

**功能特性：**
- 高度可定制的 git 历史变更日志生成器
- 内置约定式提交解析器
- 支持正则表达式的自定义提交解析
- TOML 配置文件（`cliff.toml`）
- 输出为 Markdown、JSON 或自定义模板
- 自动支持 git 标签范围
- 跨平台（Linux、macOS、Windows）
- 可作为 CLI 工具或库使用
- 支持破坏性变更检测
- 支持提交元数据的正文/页脚解析

**优点：**
- 使用 Rust 编写 - 性能优秀
- 单个二进制文件分发 - 无依赖
- 配置极其灵活
- 开发活跃，社区活跃
- 完美适合 Rust 项目
- 无运行时依赖（无需 Node.js、Python 等）
- 与业务逻辑 cleanly 分离（独立脚本）

**缺点：**
- 需要安装二进制文件（或从源码编译）
- 高级功能的配置语法可能较复杂
- 不处理版本号提升（仅生成变更日志）

### 2. release-plz

| 指标 | 值 |
|------|-----|
| **GitHub Stars** | 1,346 |
| **Forks** | 159 |
| **语言** | Rust |
| **许可证** | MIT |
| **最后更新** | 2026 年 4 月（活跃） |
| **生态系统** | 专注于 Rust crates.io |

**功能特性：**
- 自动化 Rust crate 版本号提升
- 从 git 历史生成变更日志
- 为发布创建 Pull Request
- 支持约定式提交
- GitHub Actions 集成
- 支持 Cargo 工作区的 monorepo

**优点：**
- Rust 原生解决方案
- 处理完整的发布工作流（版本 + 变更日志 + PR）
- 为 Cargo 工作区设计
- 就绪 GitHub Actions

**缺点：**
- 专注于 crates.io 发布（对于内部/应用发布过于复杂）
- 需要 GitHub Actions 才能发挥全部功能
- 对自定义发布工作流的灵活性较低
- 与 Rust crate 语义紧密耦合
- 不是为 Rabbit 这类独立应用发布设计的

### 3. conventional-changelog（Node.js 生态系统）

| 指标 | 值 |
|------|-----|
| **GitHub Stars** | 8,419 |
| **Forks** | 737 |
| **语言** | TypeScript |
| **许可证** | ISC |
| **最后更新** | 2026 年 4 月（活跃） |

**功能特性：**
- 全面的约定式提交工具链
- 多个包（changelog、commit parser 等）
- 标准版本预设配置
- Angular、Ember、jQuery 等提交约定
- 用于集成的编程 API

**优点：**
- 成熟的生态系统（10+ 年历史）
- 广泛采用，社区支持好
- 灵活的 API 供编程使用
- 多种预设配置

**缺点：**
- 需要 Node.js 运行时
- 需要管理多个包
- NPM 依赖链
- 不适合没有 Node.js 的 Rust 项目
- 依赖 footprint 较重

### 4. standard-version

| 指标 | 值 |
|------|-----|
| **GitHub Stars** | 7,963 |
| **Forks** | 766 |
| **语言** | JavaScript |
| **许可证** | ISC |
| **最后更新** | 2026 年 3 月（维护模式） |

**功能特性：**
- 自动化版本号提升 + 变更日志生成
- 支持约定式提交
- 单命令发布
- 生命周期钩子（发布前/后）

**优点：**
- 简单的 CLI 接口
- 一个命令处理版本 + 变更日志
- 文档完善

**缺点：**
- **已弃用** - 维护者建议迁移到替代方案
- 需要 Node.js
- 不再接收功能更新
- Issue 未解决就关闭
- 社区正在迁移到后继方案（release-it、semantic-release）

### 5. git-changelog-action（仅 GitHub Actions）

| 指标 | 值 |
|------|-----|
| **GitHub Stars** | 158 |
| **Forks** | 52 |
| **语言** | JavaScript |
| **最后更新** | 2025 年 11 月 |

**功能特性：**
- GitHub Actions 变更日志生成
- 支持约定式提交
- 可配置的输出格式

**优点：**
- 易于 GitHub Actions 集成
- 无需本地设置

**缺点：**
- 仅限 GitHub Actions（不能在本地 release.sh 中使用）
- 灵活性有限
- 社区小
- 不适合 Rabbit 基于 bash 的发布工作流

### 6. commitizen（cz-cli）

| 指标 | 值 |
|------|-----|
| **GitHub Stars** | 17,459 |
| **Forks** | 566 |
| **语言** | JavaScript |
| **最后更新** | 2026 年 4 月（活跃） |

**功能特性：**
- 交互式提交信息格式化
- 强制执行约定式提交格式
- 多种适配器（Angular、Ember 等）

**优点：**
- 最流行的提交信息工具
- 帮助团队遵循约定式提交
- 交互式 CLI 引导提交格式

**缺点：**
- **是提交助手，不是变更日志生成器**
- 需要 Node.js
- 仅格式化提交，不生成变更日志
- 是补充工具，不能替代变更日志生成器

## 对比总结

| 方案 | Stars | 语言 | 运行时 | 变更日志 | 版本提升 | 约定式提交 | 独立运行 | 活跃度 |
|------|-------|------|--------|---------|---------|-----------|---------|--------|
| **git-cliff** | 11.7K | Rust | 无 | 是 | 否 | 是 | 是 | 高 |
| **release-plz** | 1.3K | Rust | 无 | 是 | 是 | 是 | 否 | 高 |
| **conventional-changelog** | 8.4K | TypeScript | Node.js | 是 | 部分 | 是 | 否 | 高 |
| **standard-version** | 8.0K | JavaScript | Node.js | 是 | 是 | 是 | 否 | 低（已弃用） |
| **git-changelog-action** | 158 | JavaScript | GitHub Actions | 是 | 否 | 是 | 否 | 中 |
| **commitizen** | 17.5K | JavaScript | Node.js | 否 | 否 | 是 | 否 | 高 |

## 推荐方案

### 主要推荐：git-cliff

**对于 Rabbit 项目，推荐使用 git-cliff 方案。**

**理由：**

1. **完美匹配需求**
   - 从标签之间的 git 历史生成变更日志
   - 内置约定式提交解析器
   - 基于标签的版本跟踪
   - 与 bash 脚本集成（release.sh）
   - 完全独立于业务逻辑

2. **Rust 生态系统一致**
   - 使用 Rust 编写（与 Rabbit 项目相同）
   - 单个静态二进制文件 - 无运行时依赖
   - 易于分发和安装
   - 无需 Node.js 或其他依赖

3. **经过验证的流行度和活跃度**
   - 11,730 GitHub stars（仅变更日志工具中最高）
   - 活跃开发（最后更新：2026 年 4 月）
   - 大型社区和文档
   - 被 Hacker News 和多个博客推荐

4. **集成简单**
   - 已在 `git-changelog.sh` 中实现
   - 与当前 release.sh 工作流配合
   - 可通过 cargo、homebrew 或预编译二进制安装
   - 通过 `cliff.toml` 配置（TOML 格式，与 Cargo.toml 相同）

5. **未来灵活性**
   - 自定义模板支持不同输出格式
   - 正则解析器支持自定义提交格式
   - 破坏性变更支持
   - 可输出到 stdout 供脚本组合使用

### 可选补充工具

- **commitizen（cz-cli）**：用于在开发期间强制执行约定式提交（可选，需要 Node.js）
- **release-plz**：如果 Rabbit 未来需要发布 crate 到 crates.io 可考虑

## 实现指南

### 当前实现

Rabbit 项目已经有可用的实现：

1. **`git-changelog.sh`** - 独立的变更日志生成器脚本
   - 从 git 历史解析约定式提交
   - 按类型分类提交（feat/fix/docs/refactor/perf/chore）
   - 输出 Markdown 到 stdout，日志到 stderr
   - 支持基于标签的版本范围

2. **`release.sh`** - 发布自动化脚本
   - 集成 `git-changelog.sh` 生成发布说明
   - 自动创建带注释的 git 标签
   - 构建前更新 Cargo.toml 版本号

### 迁移到 git-cliff（可选）

如果要从自定义 `git-changelog.sh` 迁移到 git-cliff：

#### 步骤 1：安装 git-cliff

```bash
# 通过 cargo
cargo install git-cliff

# 或从 https://github.com/orhun/git-cliff/releases 下载预编译二进制
```

#### 步骤 2：创建 cliff.toml 配置

```toml
# cliff.toml
[changelog]
body = """
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
- {{ commit.message | upper_first | trim }} (`{{ commit.id | truncate(length=7, end="") }}`)\
{% endfor %}
{% endfor %}
"""

[git]
conventional_commits = true
commit_parsers = [
    { message = "^feat", group = "新功能" },
    { message = "^fix", group = "修复" },
    { message = "^perf", group = "性能优化" },
    { message = "^refactor", group = "重构" },
    { message = "^docs", group = "文档" },
    { message = "^chore", group = "其他" },
    { message = "^ci", skip = true },
]
filter_commits = true
tag_pattern = "v[0-9]*"
```

#### 步骤 3：更新 release.sh

替换 `generate_release_notes()` 函数中的变更日志调用：

```bash
# 之前（当前）：
changelog=$("${SCRIPT_DIR}/git-changelog.sh" "$version" 2>/dev/null)

# 之后（git-cliff）：
changelog=$(git-cliff --config "${SCRIPT_DIR}/cliff.toml" --tag "v${version}" 2>/dev/null)
```

### 建议：保持当前实现

目前，**保持当前的 `git-changelog.sh` 实现**，因为：

1. 已经可用且满足所有需求
2. 零外部依赖（纯 bash）
3. 无需额外安装步骤
4. 逻辑已独立于业务代码
5. 如果需要更高级功能，可以稍后迁移到 git-cliff

考虑迁移到 git-cliff 的情况：
- 需要更复杂的提交解析规则
- 需要 HTML/JSON 输出格式
- 需要破坏性变更检测
- 团队偏好使用标准化工具

## 约定式提交参考

Rabbit 项目遵循[约定式提交](https://www.conventionalcommits.org/)规范：

```
<类型>[可选 范围]: <描述>

[可选 正文]

[可选 脚注]
```

### 类型

| 类型 | 描述 | 变更日志章节 |
|------|------|-------------|
| `feat` | 新功能 | 新功能 |
| `fix` | 修复 bug | 修复 |
| `docs` | 仅文档更改 | 文档 |
| `refactor` | 代码重构 | 重构 |
| `perf` | 性能改进 | 性能优化 |
| `chore` | 维护任务 | 其他 |

### 示例

```
feat(ui): 为设置面板添加深色模式切换
fix(scanner): 修复扫描 /24 子网时的崩溃问题
docs: 更新 v0.3.0 架构图
refactor(core): 将 ping 服务提取到独立模块
perf(http-server): 使用 sendfile 减少 40% 内存使用
chore(release): 提升版本到 0.3.0
```

## 发布的 Git 工作流

```bash
# 1. 更新版本并构建
./release.sh 0.3.0

# 2. 审查生成的发布说明
cat release/RELEASE_NOTES_0.3.0.md

# 3. 推送到远程（包含标签）
git push origin main --tags

# 4. 查看特定版本的变更日志
./git-changelog.sh 0.3.0

# 5. 查看所有版本标签
git tag -l "v*"
```

## 参考资料

- [git-cliff 文档](https://git-cliff.org)
- [git-cliff GitHub 仓库](https://github.com/orhun/git-cliff)
- [约定式提交规范](https://www.conventionalcommits.org/zh-hans/)
- [release-plz 文档](https://release-plz.dev)
- [conventional-changelog](https://github.com/conventional-changelog/conventional-changelog)
