#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
use core::slice;
use std::{borrow::Cow, ffi, fmt::Display};

#[derive(Clone, thiserror::Error, Eq, PartialEq, Hash, Debug)]
#[repr(C)]
pub struct Error {
    r#type: ErrorType,
    msg: *mut ffi::c_char,
}

impl Error {
    pub fn new() -> Self {
        Self {
            r#type: ErrorType::None,
            msg: std::ptr::null_mut(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(C)]
enum ErrorType {
    None = -1,
    Exception,
    #[allow(dead_code)]
    Validation,
}

#[allow(non_camel_case_types)]
type handle_T = core::ffi::c_int;

#[doc(hidden)]
pub type BufHandle = handle_T;

#[repr(C)]
pub struct String {
    pub(super) data: *mut ffi::c_char,
    pub(super) size: usize,
}

impl self::String {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            assert!(self.len() <= isize::MAX as usize);
            unsafe { slice::from_raw_parts(self.data as *const u8, self.size) }
        }
    }

    /// Returns a pointer to the `String`'s buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const ffi::c_char {
        self.data as _
    }

    /// Creates a `String` from a byte slice by allocating `bytes.len() + 1`
    /// bytes of memory and copying the contents of `bytes` into it, followed
    /// by a null byte.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let data = unsafe { libc::malloc(bytes.len() + 1) as *mut ffi::c_char };

        unsafe { libc::memcpy(data as *mut _, bytes.as_ptr() as *const _, bytes.len()) };

        unsafe { *data.add(bytes.len()) = 0 };

        Self {
            data: data as *mut _,
            size: bytes.len(),
        }
    }

    /// Returns `true` if the `String` has a length of zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the `String`, *not* including the final null byte.
    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    /// Creates a new, empty `String`.
    #[inline]
    pub fn new() -> Self {
        Self {
            data: core::ptr::null_mut(),
            size: 0,
        }
    }

    /// Converts the `String` into Rust's `std::string::String`. If it already
    /// holds a valid UTF-8 byte sequence no allocation is made. If it doesn't
    /// the `String` is copied and all invalid sequences are replaced with `�`.
    #[inline]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        std::string::String::from_utf8_lossy(self.as_bytes())
    }
}

impl Clone for String {
    #[inline]
    fn clone(&self) -> Self {
        Self::from_bytes(self.as_bytes())
    }
}

extern "C" {
    pub(crate) fn nvim_buf_get_name(
        buf: BufHandle,
        arena: *mut core::ffi::c_void,
        err: *mut Error,
    ) -> String;

}
