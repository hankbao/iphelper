use std::io;
use std::mem;
use std::ptr;

use winapi::shared::netioapi::{
    CreateIpForwardEntry2, DeleteIpForwardEntry2, FreeMibTable, GetIpForwardTable2,
    InitializeIpForwardEntry, MIB_IPFORWARD_ROW2, PMIB_IPFORWARD_TABLE2,
};
use winapi::shared::ws2def::ADDRESS_FAMILY;

// #[derive(Default)]
pub struct MibIpForwardRow2 {
    pub inner: MIB_IPFORWARD_ROW2,
}

impl MibIpForwardRow2 {
    /// `new` initializes a MIB_IPFORWARD_ROW2 structure with default values for an IP route entry on the local computer.
    pub fn new() -> Self {
        MibIpForwardRow2::default()
    }

    /// `create` creates a new IP route entry on the local computer.
    pub fn create(&self) -> io::Result<()> {
        crate::cvt_dword(unsafe { CreateIpForwardEntry2(&self.inner) })
    }

    /// `delete` deletes an IP route entry on the local computer.
    pub fn delete(&self) -> io::Result<()> {
        crate::cvt_dword(unsafe { DeleteIpForwardEntry2(&self.inner) })
    }
}

impl Default for MibIpForwardRow2 {
    fn default() -> Self {
        MibIpForwardRow2 {
            inner: unsafe {
                let mut row: MIB_IPFORWARD_ROW2 = mem::zeroed();
                InitializeIpForwardEntry(&mut row);
                row
            },
        }
    }
}

pub struct MibIpForwardTable2(PMIB_IPFORWARD_TABLE2);

impl MibIpForwardTable2 {
    /// `new` initializes a MIB_IPFORWARD_TABLE2 table pointed by a
    /// PMIB_IPFORWARD_TABLE2 according to the address family
    pub fn new(family: ADDRESS_FAMILY) -> io::Result<Self> {
        unsafe {
            let mut table: PMIB_IPFORWARD_TABLE2 = ptr::null_mut();
            crate::cvt_dword(GetIpForwardTable2(family, &mut table))?;
            Ok(MibIpForwardTable2(table))
        }
    }

    fn_table_iter! {MibIpForwardTable2Iter}
}

impl Drop for MibIpForwardTable2 {
    fn drop(&mut self) {
        unsafe {
            FreeMibTable(self.0 as *mut _);
        }
    }
}

declare_table_iter! {
    MibIpForwardTable2Iter,
    MibIpForwardRow2,
    MIB_IPFORWARD_ROW2
}
