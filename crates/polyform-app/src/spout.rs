//! Spout texture sharing sender (Windows-only).
//!
//! Wraps the SpoutDX C++ library via a thin extern "C" bridge.
//! Sends BGRA pixel data to any Spout-compatible receiver.

#![cfg(target_os = "windows")]

use std::ffi::{CString, c_void};

unsafe extern "C" {
    fn spout_sender_create() -> *mut c_void;
    fn spout_sender_open(
        ptr: *mut c_void,
        name: *const i8,
        width: u32,
        height: u32,
    ) -> bool;
    fn spout_sender_send_image(
        ptr: *mut c_void,
        data: *const u8,
        width: u32,
        height: u32,
    ) -> bool;
    fn spout_sender_close(ptr: *mut c_void);
    fn spout_sender_destroy(ptr: *mut c_void);
}

pub struct SpoutSender {
    ptr: *mut c_void,
    width: u32,
    height: u32,
}

impl SpoutSender {
    /// Create and open a Spout sender with the given name and dimensions.
    /// Returns `None` if D3D11 initialization fails.
    pub fn new(name: &str, width: u32, height: u32) -> Option<Self> {
        let c_name = CString::new(name).ok()?;
        unsafe {
            let ptr = spout_sender_create();
            if ptr.is_null() {
                return None;
            }
            if !spout_sender_open(ptr, c_name.as_ptr(), width, height) {
                spout_sender_destroy(ptr);
                return None;
            }
            Some(Self { ptr, width, height })
        }
    }

    /// Send a frame of BGRA pixel data. The slice must contain exactly
    /// `width * height * 4` bytes.
    pub fn send_image(&self, data: &[u8]) -> bool {
        let expected = (self.width as usize) * (self.height as usize) * 4;
        if data.len() != expected {
            return false;
        }
        unsafe { spout_sender_send_image(self.ptr, data.as_ptr(), self.width, self.height) }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Drop for SpoutSender {
    fn drop(&mut self) {
        unsafe {
            spout_sender_close(self.ptr);
            spout_sender_destroy(self.ptr);
        }
    }
}

// SpoutDX is not thread-safe, but we only use it from the main thread.
// These impls allow storing SpoutSender in App which isn't Send/Sync anyway.
unsafe impl Send for SpoutSender {}
unsafe impl Sync for SpoutSender {}
