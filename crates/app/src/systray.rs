// Platform dispatch for system tray.
//
// Linux:   ksni (pure Rust StatusNotifierItem via D-Bus) — no libappindicator.
// Other:   tray-icon crate (native APIs on Windows/macOS).

// Platform dispatch for system tray.
//
// Linux:   ksni (pure Rust StatusNotifierItem via D-Bus) — no libappindicator.
// Other:   tray-icon crate (native APIs on Windows/macOS).

#[cfg(target_os = "linux")]
#[path = "systray_linux.rs"]
mod imp;
#[cfg(target_os = "linux")]
pub use self::imp::*;

#[cfg(not(target_os = "linux"))]
#[path = "systray_non_linux.rs"]
mod imp;
#[cfg(not(target_os = "linux"))]
pub use self::imp::*;
