use std::io;
use std::mem;
use std::ptr;

use winapi::shared::netioapi::{
    CreateUnicastIpAddressEntry, DeleteUnicastIpAddressEntry, FreeMibTable,
    GetUnicastIpAddressTable, InitializeUnicastIpAddressEntry, MIB_UNICASTIPADDRESS_ROW,
    PMIB_UNICASTIPADDRESS_TABLE,
};
use winapi::shared::ws2def::ADDRESS_FAMILY;

pub struct MibUnicastIpAddressRow {
    pub inner: MIB_UNICASTIPADDRESS_ROW,
}

impl MibUnicastIpAddressRow {
    /// `new` initializes a MibUnicastIPAddressRow structure with default values for a unicast IP address entry on the local computer.
    pub fn new() -> Self {
        MibUnicastIpAddressRow::default()
    }

    /// `create` adds a new unicast IP address entry on the local computer.
    pub fn create(&self) -> io::Result<()> {
        crate::cvt_dword(unsafe { CreateUnicastIpAddressEntry(&self.inner) })
    }

    /// `delete` deletes an existing unicast IP address entry on the local computer.
    pub fn delete(&self) -> io::Result<()> {
        crate::cvt_dword(unsafe { DeleteUnicastIpAddressEntry(&self.inner) })
    }
}

impl Default for MibUnicastIpAddressRow {
    fn default() -> Self {
        MibUnicastIpAddressRow {
            inner: unsafe {
                let mut row: MIB_UNICASTIPADDRESS_ROW = mem::zeroed();
                InitializeUnicastIpAddressEntry(&mut row);
                row
            },
        }
    }
}

pub struct MibUnicastIpAddressTable(PMIB_UNICASTIPADDRESS_TABLE);

impl MibUnicastIpAddressTable {
    /// `new` initializes a MIB_UNICASTIPADDRESS_TABLE table pointed by a
    /// PMIB_UNICASTIPADDRESS_TABLE according to the address family
    pub fn new(family: ADDRESS_FAMILY) -> io::Result<Self> {
        unsafe {
            let mut table: PMIB_UNICASTIPADDRESS_TABLE = ptr::null_mut();
            crate::cvt_dword(GetUnicastIpAddressTable(family, &mut table))?;
            Ok(MibUnicastIpAddressTable(table))
        }
    }

    fn_table_iter! {MibUnicastIpAddressTableIter}
}

impl Drop for MibUnicastIpAddressTable {
    fn drop(&mut self) {
        unsafe {
            FreeMibTable(self.0 as *mut _);
        }
    }
}

declare_table_iter! {
    MibUnicastIpAddressTableIter,
    MibUnicastIpAddressRow,
    MIB_UNICASTIPADDRESS_ROW
}
