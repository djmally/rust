// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use prelude::v1::*;
use io::prelude::*;

use io::{self, Cursor};
use iter::repeat;
use libc;
use ptr;
use str;
use sync::Mutex;
use sys::c;
use sys::cvt;
use sys::handle::Handle;

struct NoClose(Option<Handle>);

pub struct Stdin {
    handle: NoClose,
    utf8: Mutex<io::Cursor<Vec<u8>>>,
}
pub struct Stdout(NoClose);
pub struct Stderr(NoClose);

fn get(handle: libc::DWORD) -> io::Result<NoClose> {
    let handle = unsafe { c::GetStdHandle(handle) };
    if handle == libc::INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else if handle.is_null() {
        Err(io::Error::new(io::ErrorKind::Other,
                           "no stdio handle available for this process", None))
    } else {
        Ok(NoClose::new(handle))
    }
}

fn write(handle: libc::HANDLE, data: &[u8]) -> io::Result<usize> {
    let utf16 = match str::from_utf8(data).ok() {
        Some(utf8) => utf8.utf16_units().collect::<Vec<u16>>(),
        None => return Err(invalid_encoding()),
    };
    let mut written = 0;
    try!(cvt(unsafe {
        c::WriteConsoleW(handle,
                         utf16.as_ptr() as libc::LPCVOID,
                         utf16.len() as u32,
                         &mut written,
                         ptr::null_mut())
    }));

    // FIXME if this only partially writes the utf16 buffer then we need to
    //       figure out how many bytes of `data` were actually written
    assert_eq!(written as usize, utf16.len());
    Ok(data.len())
}

impl Stdin {
    pub fn new() -> Stdin {
        Stdin {
            handle: get(c::STD_INPUT_HANDLE).unwrap(),
            utf8: Mutex::new(Cursor::new(Vec::new())),
        }
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let mut utf8 = self.utf8.lock().unwrap();
        // Read more if the buffer is empty
        if utf8.position() as usize == utf8.get_ref().len() {
            let mut utf16: Vec<u16> = repeat(0u16).take(0x1000).collect();
            let mut num = 0;
            try!(cvt(unsafe {
                c::ReadConsoleW(self.handle.raw(),
                                utf16.as_mut_ptr() as libc::LPVOID,
                                utf16.len() as u32,
                                &mut num,
                                ptr::null_mut())
            }));
            utf16.truncate(num as usize);
            // FIXME: what to do about this data that has already been read?
            let data = match String::from_utf16(&utf16) {
                Ok(utf8) => utf8.into_bytes(),
                Err(..) => return Err(invalid_encoding()),
            };
            *utf8 = Cursor::new(data);
        }

        // MemReader shouldn't error here since we just filled it
        utf8.read(buf)
    }
}

impl Stdout {
    pub fn new() -> Stdout {
        Stdout(get(c::STD_OUTPUT_HANDLE).unwrap())
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        write(self.0.raw(), data)
    }
}

impl Stderr {
    pub fn new() -> Stderr {
        Stderr(get(c::STD_ERROR_HANDLE).unwrap())
    }

    pub fn write(&self, data: &[u8]) -> io::Result<usize> {
        write(self.0.raw(), data)
    }
}

impl NoClose {
    fn new(handle: libc::HANDLE) -> NoClose {
        NoClose(Some(Handle::new(handle)))
    }

    fn raw(&self) -> libc::HANDLE { self.0.as_ref().unwrap().raw() }
}

impl Drop for NoClose {
    fn drop(&mut self) {
        self.0.take().unwrap().into_raw();
    }
}

fn invalid_encoding() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "text was not valid unicode",
                   None)
}
