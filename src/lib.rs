use std::io;

use winapi::shared::minwindef::DWORD;
use winapi::shared::winerror::NO_ERROR;

macro_rules! fn_table_iter {
    ($iter:ident) => {
        pub fn iter(&self) -> $iter {
            let ref_table = unsafe { self.0.as_mut().unwrap() };
            $iter {
                len: ref_table.NumEntries as usize,
                offset: 0,
                ptr: &ref_table.Table[0],
            }
        }
    }
}

macro_rules! declare_table_iter {
    ($iter:ident, $item:ident, $inner:ty) => {
        pub struct $iter {
            len: usize,
            offset: usize,
            ptr: *const $inner,
        }

        impl Iterator for $iter {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                if self.offset >= self.len {
                    return None;
                }

                Some(unsafe {
                    let val = *self.ptr;
                    self.offset += 1;
                    self.ptr.add(1);
                    $item { inner: val }
                })
            }
        }
    };
}

pub mod if2;
pub mod ip_forward;
pub mod ip_interface;
pub mod unicast_ip;

fn cvt_dword(d: DWORD) -> io::Result<()> {
    if d == NO_ERROR {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(d as i32))
    }
}
