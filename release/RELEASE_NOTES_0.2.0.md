# Rabbit 0.2.0 Release Notes

## Release Date

2026-05-13

## Platform

- linux-x64

## Changes

## Version 0.2.0 (2026/05/13)

### Features

- enhance version dialog layout with spacers (`19ac182`)
- implement custom version info dialog for upgrade prompts (`7c62ab0`)
- add diagnostic logging and tray helper for elevated process (`e8f3a77`)
- add cross-platform MAC address resolution via ARP (`b562d70`)
- add image processing and tray icon dependencies (`9aaff0b`)
- add fullscreen reminder and timer-based triggers (`f5a5a5c`)
- support multiple directories and split add buttons (`fdaaf7a`)
- implement taskbar progress for ping state (`949bc02`)
- improve config handling and UI updates (`cdd261c`)
- add module toggle event handler (`615ae8e`)
- add cross-platform privilege elevation support (`4df5e06`)
- add window position persistence with debounced save (`bb3e922`)
- add config persistence and UI state restoration (`24c00e6`)
- add Windows icon, console hide and ping stats (`26ec5c2`)
- add ping button state sync and window title update (`7d68e04`)
- add business state persistence and restore on startup (`b6c5aac`)
- add taskbar, shell integration and ping logging (`870110f`)
- implement TFTP, DNS lookup, autostart, hotkeys, and HTTP dir management (`b73da74`)
- update download progress in place (`0b24cfe`)
- add automated changelog generation and release notes integration (`877636a`)
- add auto-update system with download and install (`11ecc0e`)
- implement unified version management and update checking (`75970e0`)
- add date/time pickers, scan task, tab persistence, and version check (`d19401d`)
- implement file upload, task state management and chat heartbeat (`47cce65`)
- implement dark theme and compact layout (`5441779`)
- refactor UI config initialization and default values (`0bdfe08`)
- add rabbit-app module with UI and service integration (`3f14f61`)

### Bug Fixes

- conditionally enable windows subsystem (`9d61c4e`)
- optimize display refresh and scan state handling (`ee9c2d5`)
- set socket type hint to RAW (`438df71`)
- optimize startup, exit, and suppress console flashing (`bd9af45`)
- add auth agent detection and retry mechanism (`86a6dfc`)
- ensure thread-safe shutdown during version check (`56d196e`)
- handle non-JSON version check response (`04b0d85`)

### Documentation

- update ping status display rules (`34f4b79`)
- update data flow design and add implementation plan (`b7c6898`)
- update config structure and documentation (`3c07060`)
- update config implementation plan and structure (`f20475e`)
- add config implementation plan and issue tracking (`f71cbc9`)
- update window, HTTP, and privilege escalation fixes (`bee96be`)
- add testing guide and expand integration tests (`6861443`)
- add project documentation for architecture, requirements, and TFTP evaluation (`bae3231`)

### Refactoring

- replace magic strings with constants (`09b40d8`)
- remove is_running, add destroy/update to TftpcService (`92f3fb5`)
- replace polling threads with event-driven timeout and poll (`9959284`)
- embed icon at compile time (`471e887`)
- unify shutdown handling with Lifecycle module (`14f15c2`)
- replace gtk with ksni for Linux system tray (`33608b0`)
- remove redundant references and update dependencies (`9664f7d`)
- optimize code style and add linux gtk support (`5e1c227`)
- standardize log output with write_to and timestamp (`77487e7`)
- consolidate window config into JSON structure (`c8803a4`)
- restructure tasks storage and ui handling (`5822430`)
- rename crates to schema, adapter, service, and app (`21a80fd`)
- replace tower-http with custom file handler (`37c7e0b`)
- unify module state updates and taskbar handling (`69e64b2`)
- update taskbar progress calculation with sliding window (`b41bb93`)
- simplify code formatting and update privilege elevation implementation (`c73fb00`)
- separate config persistence from runtime state (`17287e9`)
- migrate to event-driven UI updates (`f56569e`)
- replace polling loop with event-driven refresh (`9e1ecbf`)
- simplify service management and code formatting (`c87a6a4`)
- optimize code quality and performance across modules (`ac25cd6`)
- extract common send_ui implementation to eliminate duplicate code (`54170b0`)
- remove service init methods and make stop/is_running private (`35d59e0`)
- add generic config getters and setters (`87fa8db`)
- track running state via config (`f26c92d`)
- replace static mut with parking_lot::Mutex for thread safety (`c53b54d`)
- remove unused code and imports (`e65164a`)
- remove dead code and optimize structures (`cfd26ab`)
- remove dead code and suppress warnings (`a2639bc`)
- fix unused variable warnings (`e29d8db`)
- unify service update interface with config handling (`3f5dfba`)
- improve service lifecycle and status handling (`0b29ff5`)
- remove unused models and simplify services (`9f64418`)
- simplify service config management (`e9ae1a0`)
- migrate from winapi to windows-sys (`9fb7fb7`)
- replace polling with channel-based UI updates (`3003776`)
- implement unified service update interface (`b7206be`)
- split TftpService into TftpdService and TftpcService (`0cbc13f`)
- migrate config to modules-based structure (`14a8e25`)
- migrate to HashMap-based config storage (`721a15c`)
- unify module toggle events and simplify state management (`8fdf864`)
- unify config update interface (`1a95d4f5`)
- improve config handling and add logging (`78c684f`)
- simplify elevation command handling (`d2e82d1`)
- unify config model with From trait conversion (`9fff16c`)
- optimize UI refresh and resize handling (`afc2788`)
- replace Slint with FLTK framework (`02b3b8a`)
- migrate from .NET to Rust with updated docs and configs (`13a9b2c`)

### Other Changes

- Improves Windows ARP resolution and MAC lookup diagnostics (`33d2683`)
- ``` feat(network): improve Windows ARP lookup using GetIpNetTable API (`a95c35f`)
- ``` refactor(adapter): consolidate window handle storage and access (`660e813`)
- ``` feat(systray): add Linux-specific FLTK widget extension import (`e6ee297`)
- ``` fix(adapter): remove unnecessary error logging in elevate_with_xelevate_inner (`4ef09fd`)
- ``` refactor(adapter): clean up unused code and improve error handling (`4a607dc`)
- ``` refactor(service): remove unused TftpService type alias (`2434b6d`)
- ``` feat(adapter): add unified systray management and platform utilities (`57da9ae`)
- ``` feat(adapter): add system tray support and improve elevation handling (`cb547ec`)
- ``` feat(network): improve MAC address resolution on Linux using ioctl (`ea3cb2e`)
- ``` feat: add Chinese localization and restructure project architecture (`686d405`)
- Fix release script to update Cargo.toml version before build (`7c310e1`)

## Download

| Platform | URL |
|----------|-----|
| linux-x64 |  |

## SHA256 Checksums

```
  rabbit-0.2.0-linux-x64
```
