// leveldb/include/leveldb/status.h
// leveldb/util/status.cc

#![allow(dead_code)]
use crate::slice::Slice;
use std::alloc;
use std::mem::size_of;
use std::ptr::null;

pub struct Status {
    // OK status has a None 'state'. Otherwise, 'state' is an alloced array of memory of the
    // following form:
    //    state[0..3] == length of message
    //    state[4]    == code
    //    state[5..]  == message
    // TODO: Use *const u8? Test to see if conversion from mut to const to mut causes error
    // NOTE: C++ uses char
    //state: Option<*const u8>,
    state: *const u8,
}

impl Status {
    // Create a success state
    pub fn new() -> Self {
        Self { state: null() }
    }

    // TODO: Copy

    // Return a success state.
    // NOTE: Kept all caps to align with C++ name and not class with 'ok' method below
    #[allow(non_snake_case)]
    pub fn OK() -> Self {
        Self::new()
    }

    // Return erorr status of an appropriate type.
    pub fn not_found(msg: &Slice, msg2: &Slice) -> Self {
        Self::from_code_msgs(Code::NotFound, msg, msg2)
    }

    pub fn corruption(msg: &Slice, msg2: &Slice) -> Self {
        Self::from_code_msgs(Code::Corruption, msg, msg2)
    }

    pub fn not_supported(msg: &Slice, msg2: &Slice) -> Self {
        Self::from_code_msgs(Code::NotSupported, msg, msg2)
    }

    pub fn invalid_argument(msg: &Slice, msg2: &Slice) -> Self {
        Self::from_code_msgs(Code::InvalidArgument, msg, msg2)
    }

    pub fn io_error(msg: &Slice, msg2: &Slice) -> Self {
        Self::from_code_msgs(Code::IOError, msg, msg2)
    }

    // Returns true iff the status indicates success.
    // NOTE: C++ checks if state is nullptr
    pub fn ok(&self) -> bool {
        self.state.is_null()
    }

    // Returns true iff the status indicates a NotFound error.
    pub fn is_not_found(&self) -> bool {
        self.code() == Code::NotFound
    }

    // Returns true iff the status indicates a Corruption error.
    pub fn is_corruption(&self) -> bool {
        self.code() == Code::Corruption
    }

    // Returns true iff the status indicates a IOError.
    pub fn is_io_error(&self) -> bool {
        self.code() == Code::IOError
    }

    // Returns true iff the status indicates a NotSupported error.
    pub fn is_not_supported_error(&self) -> bool {
        self.code() == Code::NotSupported
    }

    // Returns true iff the status indicates a InvalidArgument error.
    pub fn is_invalid_argument(&self) -> bool {
        self.code() == Code::InvalidArgument
    }

    fn code(&self) -> Code {
        if !self.state.is_null() {
            unsafe { Code::try_from(*self.state.add(4)).unwrap() }
        } else {
            Code::Ok
        }
    }

    fn from_code_msgs(code: Code, msg: &Slice, msg2: &Slice) -> Self {
        assert!(code != Code::Ok);
        let (len1, len2) = (msg.size(), msg2.size());
        let size = (len1 + if len2 != 0 { 2 + len2 } else { 0 }) as u32;
        unsafe {
            let layout = alloc::Layout::array::<u8>(size as usize + 5).unwrap();
            let result = alloc::alloc(layout);
            // NOTE: C++ uses Rust equivalent of size_of_val
            result.copy_from((&size) as *const u32 as *const u8, size_of::<u32>());
            *result.add(4) = code as u8;
            result.add(5).copy_from(msg.data().cast(), len1);
            if len2 != 0 {
                *result.add(5 + len1) = b':';
                *result.add(6 + len1) = b' ';
                result.add(7 + len1).copy_from(msg.data().cast(), len2);
            }
            Self {
                state: result as *const u8,
            }
        }
    }

    fn copy_state(state: *const u8) -> *const u8 {
        unsafe {
            let mut size = 0u32;
            // NOTE: C++ uses Rust equivalent of size_of_val
            state.copy_to((&mut size) as *mut u32 as *mut u8, size_of::<u32>());
            let size = size as usize;
            let layout = alloc::Layout::array::<u8>(size + 5).unwrap();
            let result = alloc::alloc(layout);
            result.copy_from(state, size + 5);
            result as *const u8
        }
    }
}

impl Clone for Status {
    fn clone(&self) -> Self {
        //let state = self.state.map(|p| Self::copy_state(p as *const _));
        Self {
            state: if self.state.is_null() {
                null()
            } else {
                Self::copy_state(self.state)
            },
        }
        //Self { state }
    }
}

// NOTE: `Copy` can't be implemented like it is in C++ since types implementing `Copy` can't have
// destructors
//impl Copy for Status {}

impl ToString for Status {
    // Return a string representation of this status suitable for printing.
    // Returns the string "OK" for success
    fn to_string(&self) -> String {
        if !self.state.is_null() {
            // NOTE: C++ handles a default case for an unknown code. However, this cannot happen
            // with Rust's enums
            let mut result = match self.code() {
                Code::Ok => "OK",
                Code::NotFound => "NotFound: ",
                Code::Corruption => "Corruption: ",
                Code::NotSupported => "Not implemented: ",
                Code::InvalidArgument => "Invalid argument: ",
                Code::IOError => "IO error: ",
            }
            .to_owned();
            unsafe {
                let mut length = 0u32;
                // NOTE: C++ uses the Rust equivalent of size_of_val
                ((&mut length) as *mut u32)
                    .cast::<u8>()
                    .copy_from(self.state, size_of::<u32>());
                let (l, length) = (result.len(), length as usize);
                result.push_str(&"\0".repeat(length));
                result
                    .as_mut_ptr()
                    .add(l)
                    .copy_from(self.state.add(5), length);
            }
            result
        } else {
            String::from("ok")
        }
    }
}

impl Drop for Status {
    fn drop(&mut self) {
        if !self.state.is_null() {
            unsafe {
                let mut size = 0u32;
                self.state
                    .copy_to((&mut size) as *mut u32 as *mut u8, size_of::<u32>());
                alloc::dealloc(
                    self.state as *mut u8,
                    alloc::Layout::array::<u8>(size as usize + 5).unwrap(),
                );
            }
        }
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
#[repr(u8)]
enum Code {
    Ok,
    NotFound,
    Corruption,
    NotSupported,
    InvalidArgument,
    IOError,
}

impl std::convert::TryFrom<u8> for Code {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Ok),
            1 => Ok(Self::NotFound),
            2 => Ok(Self::Corruption),
            3 => Ok(Self::NotSupported),
            4 => Ok(Self::InvalidArgument),
            5 => Ok(Self::IOError),
            _ => Err(format!("unknown code: {}", value)),
        }
    }
}
