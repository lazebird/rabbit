# AGENTS.md

This file provides guidance to Qoder (qoder.com) when working with code in this repository.

## Project Overview

Rabbit is a cross-platform collection of networking utilities and productivity tools. It integrates multiple utilities (ping, HTTP server, TFTP server/client, IP scanner, LAN chat) and task planning into a single application.

**Tech Stack:** Rust + FLTK GUI framework

## Build Commands

```bash
# Build (Debug)
cargo build

# Build (Release)
cargo build --release

# Run
cargo run

# Run tests
cargo test

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

Requirements:
- Rust 1.75+ (2024 edition)
- Platform-specific build tools

## Project Architecture

### Four-Layer Architecture

```
┌─────────────────────────────────────────┐
│           Presentation Layer             │
│   View (FLTK UI) ← ViewModel (State)     │
├─────────────────────────────────────────┤
│           Business Layer                 │
│   PingService / HttpService / ...       │
├─────────────────────────────────────────┤
│           Data Layer                     │
│   Models + Repository (Persistence)      │
├─────────────────────────────────────────┤
│        Infrastructure Layer              │
│   WindowsPlatform / LinuxPlatform       │
└─────────────────────────────────────────┘
```

### Module Structure

```
rabbit/
├── crates/
│   ├── rabbit-app/       # Main entry + UI
│   ├── rabbit-core/      # Business services
│   ├── rabbit-models/    # Data models
│   └── rabbit-platform/  # Platform adapters
├── tests/
├── doc/
│   ├── architecture.md       # Architecture design
│   ├── requirements.md       # Requirements spec
│   └── tftp-evaluation.md    # TFTP library evaluation
└── AGENTS.md
```

### Feature Modules

| Module | Description |
|--------|-------------|
| Ping | ICMP ping with taskbar status |
| IP Scanner | Network IP scanning |
| HTTP Server | Simple HTTP file server |
| TFTP Server/Client | TFTP file transfer |
| Task Planner | Scheduled reminders |
| LAN Chat | UDP broadcast chat |

## Third-Party Libraries

| Module | Library | Notes |
|--------|---------|-------|
| UI Framework | fltk | Cross-platform native UI |
| Async Runtime | tokio | Async I/O |
| HTTP Server | axum | Lightweight async HTTP |
| TFTP | async-tftp | Async TFTP with Handler |
| Ping | surge-ping | ICMP ping |
| Serialization | serde | JSON/config |

## Release Build Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
panic = "abort"      # Reduce binary size
strip = true         # Strip symbols
```

## Platform Support

- Windows x64
- Linux x64 (glibc)
- Linux arm64

## Documentation

- `doc/architecture.md` - Technical selection and architecture
- `doc/requirements.md` - Detailed requirements and UI specs
- `doc/tftp-evaluation.md` - Rust TFTP library evaluation
- `doc/progress.md` - Development progress tracking
- `doc/requirements-gap-analysis.md` - Requirements vs implementation gap analysis
- `doc/version-management.md` - Version and release management
- `doc/ui-framework-evaluation.md` - UI framework comparison
- `doc/changelog-solution.md` - Changelog tool comparison
