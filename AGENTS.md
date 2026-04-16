# AGENTS.md

This file provides guidance to Qoder (qoder.com) when working with code in this repository.

## Project Overview

Rabbit is a collection of Windows small tools built with .NET Framework 4.7.2 and Windows Forms. It integrates multiple networking utilities (ping, HTTP server, TFTP server/client, IP scanner, LAN chat) and productivity tools (task planner) into a single application.

## Build Commands

```bash
# Build the solution (Debug configuration)
msbuild rabbit.sln -p:Configuration=Debug

# Build the solution (Release configuration)
msbuild rabbit.sln -p:Configuration=Release

# Build a specific project
msbuild lazebird.rabbit.rabbit/lazebird.rabbit.rabbit.csproj -p:Configuration=Debug

# Clean build artifacts
msbuild rabbit.sln -t:Clean
```

Requirements:
- .NET Framework 4.7.2 SDK
- MSBuild (typically installed with Visual Studio or .NET Framework SDK)
- NuGet for package restoration

## Project Architecture

### Module Structure

The solution follows a modular architecture where each feature is implemented as a separate class library:

| Project | Description |
|---------|-------------|
| `lazebird.rabbit.rabbit` | Main WinForms application (entry point) |
| `lazebird.rabbit.common` | Shared utilities (logging, taskbar, options parsing) |
| `lazebird.rabbit.ping` | Ping functionality |
| `lazebird.rabbit.http` | HTTP server |
| `lazebird.rabbit.tftp` | TFTP client/server |
| `lazebird.rabbit.fs` | File system operations |
| `lazebird.rabbit.chat` | LAN chat |
| `lazebird.rabbit.plan` | Task planning/reminders |
| `lazebird.rabbit.conf` | Configuration management |
| `lazebird.rabbit.key` | Keyboard shortcut handling |

The `*.core` directories contain pre-built binary outputs, not source code. Do not modify them directly.

### Form Structure

The main form (`Form1`) is split across multiple partial class files:
- `Form1.cs` - Main form initialization, calls `init_form_*()` for each module
- `Form1.Designer.cs` - WinForms designer-generated UI code
- `form_ping.cs`, `form_http.cs`, `form_tftpd.cs`, etc. - Module-specific form logic

### Module API Pattern

Each module typically provides:
- Constructor taking an `Action<string>` log callback
- `start()` method to begin operation
- `stop()` method to halt operation
- Callback-based async operations

Example from `lazebird.rabbit.ping/rping.cs`:
```csharp
rping ping = new rping(log_callback);
ping.start(address, options, callback, data);
ping.stop();
```

### Options Parsing

Options are passed as semicolon-separated `key=value` strings and parsed using `ropt.parse_opts()`:
```csharp
Hashtable opts = ropt.parse_opts("interval=1000;count=10;taskbar=true");
```

### Logging

The `rlog` class in `lazebird.rabbit.common` handles logging:
- Writes to a `ListBox` UI component
- Optional file output via `savefile(path)`
- Thread-safe for cross-thread UI updates

## Release Process

The release build uses ILMerge to create a single executable (`sRabbit.exe`):
1. Pre-build: `tools/svgen.exe` generates version information
2. Build: MSBuild with Release configuration
3. Post-build: `tools/releasepack.bat` runs ILMerge to combine all DLLs into single EXE

## Language Support

The application supports Chinese and English through the `Language` class in `lang.cs`. Default is English. The `Language.trans()` method provides translation lookup.

## Dependencies

- `Microsoft.WindowsAPICodePack` - Windows 7+ taskbar progress bar integration
- `lazebird.vgen.version` - Automatic version generation

NuGet packages are restored automatically during build. Check `packages.config` files for specific versions.
