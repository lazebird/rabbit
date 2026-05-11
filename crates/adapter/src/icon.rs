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

/// Returns the raw bytes of the embedded `.ico` file.
///
/// Useful for callers that need FLTK `IcoImage` or other formats
/// that consume the original `.ico` data rather than decoded RGBA.
pub fn ico_bytes() -> &'static [u8] {
    include_bytes!("../../../resources/icon.ico")
}

/// Load the application icon as decoded RGBA data.
///
/// The `.ico` file is embedded into the binary at compile time via
/// `include_bytes!`, so this never touches the file system at runtime.
pub fn load_app_icon() -> IconData {
    let bytes = ico_bytes();
    let img = image::load_from_memory(bytes).expect("embedded icon.ico is valid");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    IconData { rgba, width, height }
}
