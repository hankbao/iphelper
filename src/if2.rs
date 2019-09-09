use std::io;
use std::mem;

use winapi::shared::ifdef::NET_LUID;
use winapi::shared::netioapi::{GetIfEntry2, MIB_IF_ROW2};

pub struct IfRow2 {
    pub inner: MIB_IF_ROW2,
}

impl IfRow2 {
    /// `new` sets `InterfaceLuid` member of an MIB_IF_ROW2 entry as `luid` and initializes
    ///  the other members with default values.
    pub fn new(luid: NET_LUID) -> io::Result<Self> {
        let mut row = IfRow2 {
            inner: unsafe {
                let mut row: MIB_IF_ROW2 = mem::zeroed();
                row.InterfaceLuid = luid;
                row
            },
        };

        row.get()?;
        Ok(row)
    }

    /// `get` retrieves information for the specified interface on the local computer.
    pub fn get(&mut self) -> io::Result<()> {
        crate::cvt_dword(unsafe { GetIfEntry2(&mut self.inner) })
    }
}
