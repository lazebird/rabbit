//! Application icon — embedded at compile time.
//!
//! Provides the app icon as raw RGBA data so each platform's systray code
//! can convert it to the target icon type without file-system dependencies.

use image::GenericImageView;

/// Raw RGBA icon data decoded from the embedded `.ico` file.
pub struct IconData {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Load the application icon.
///
/// The `.ico` file is embedded into the binary at compile time via
/// `include_bytes!`, so this never touches the file system at runtime.
pub fn load_app_icon() -> IconData {
    let bytes = include_bytes!("../resources/icon.ico");
    let img = image::load_from_memory(bytes).expect("embedded icon.ico is valid");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    IconData { rgba, width, height }
}
