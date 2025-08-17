// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! Auxiliary FFI types and utilities (wide strings, pointer wrappers) used by the real Profit DLL layer.
//!
//! See [MANUAL.md](../MANUAL.md#tipos-auxiliares) for details on types and conventions.
#![cfg(all(target_os = "windows", feature = "real_dll"))]

use std::ffi::c_void;

/// Converts a pointer to a null-terminated wide string (UTF-16) into a [`String`].
///
/// Returns an empty string if the pointer is null.
///
/// Used for interoperability with DLL functions that return UTF-16 pointers.
///
/// # Safety
/// The pointer must be valid and point to a UTF-16 sequence terminated by null.
pub unsafe fn utf16_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // Descobre comprimento até nul (scan linear).
    let mut len = 0usize;
    loop {
        let v = *ptr.add(len);
        if v == 0 {
            break;
        }
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}

/// Wrapper for pointers returned by the DLL that must be freed via **FreePointer**.
///
/// If the function does not exist, we do not attempt to free (avoids double free). May cause a minor leak.
///
/// See [MANUAL.md](../MANUAL.md#freepointer) for details.
pub struct ForeignBuffer {
    ptr: *mut c_void,
    free_fn: Option<unsafe extern "system" fn(*mut c_void)>,
}

impl ForeignBuffer {
    pub fn new(ptr: *mut c_void, free_fn: Option<unsafe extern "system" fn(*mut c_void)>) -> Self {
        Self { ptr, free_fn }
    }
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
    /// Consumes the buffer and returns the raw pointer.
    ///
    /// # Safety
    /// The returned pointer must be manually freed if necessary, and must not be used after the object is dropped.
    pub unsafe fn into_raw(self) -> *mut c_void {
        let p = self.ptr;
        std::mem::forget(self);
        p
    }
}

impl Drop for ForeignBuffer {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        if let Some(f) = self.free_fn {
            unsafe { f(self.ptr) };
        }
    }
}
