//! X11 error handler — installed before the FLTK event loop to log any
//! X11 protocol errors that may cause the transient FlError::Internal
//! observed during startup.  Chains to FLTK's internal handler so normal
//! error handling is preserved.

#![cfg(target_os = "linux")]

use std::ffi::{c_char, c_int, c_ulong, c_void};
use std::sync::OnceLock;

#[repr(C)]
struct XErrorEvent {
    type_: c_int,
    display: *mut c_void,
    serial: c_ulong,
    error_code: c_char,
    request_code: c_char,
    minor_code: c_char,
    resource_id: c_ulong,
}

type XErrorHandler = unsafe extern "C" fn(*mut c_void, *mut XErrorEvent) -> c_int;

extern "C" {
    fn XSetErrorHandler(handler: Option<XErrorHandler>) -> Option<XErrorHandler>;
    fn XGetErrorText(display: *mut c_void, code: c_int, buf: *mut c_char, len: c_int) -> c_int;
}

static PREV: OnceLock<XErrorHandler> = OnceLock::new();

unsafe extern "C" fn diag_handler(dpy: *mut c_void, ev: *mut XErrorEvent) -> c_int {
    let e = &*ev;

    // Resolve the error code to a human-readable string via Xlib.
    let mut buf = [0i8; 512];
    XGetErrorText(dpy, e.error_code as c_int, buf.as_mut_ptr(), buf.len() as c_int);
    let desc = std::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy();

    rabbit_diag::log(&format!(
        "[X11 Error] {} (code={} opcode={} minor={} serial={} res_id={})",
        desc, e.error_code, e.request_code, e.minor_code, e.serial, e.resource_id,
    ));
    // Chain to FLTK's handler so its internal error tracking still works.
    if let Some(&prev) = PREV.get() {
        prev(dpy, ev)
    } else {
        0
    }
}

/// Install the X11 error diagnostic handler.
///
/// Must be called before the FLTK event loop is entered.  Safe to call
/// multiple times — only the first call installs the handler.
pub fn install() {
    unsafe {
        // XSetErrorHandler never returns NULL per Xlib docs, so the
        // Option will always be Some (the default or FLTK's handler).
        if let Some(prev) = XSetErrorHandler(Some(diag_handler)) {
            let _ = PREV.set(prev);
            rabbit_diag::log("[X11] X11 error handler installed (chained)");
        }
    }
}
