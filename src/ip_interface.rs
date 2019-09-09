use std::io;
use std::mem;

use winapi::shared::ifdef::NET_LUID;
use winapi::shared::netioapi::{
    GetIpInterfaceEntry, InitializeIpInterfaceEntry, SetIpInterfaceEntry, MIB_IPINTERFACE_ROW,
};
use winapi::shared::ws2def::{AF_INET, AF_INET6};

pub struct MibIpInterfaceRow {
    pub inner: MIB_IPINTERFACE_ROW,
}

impl MibIpInterfaceRow {
    /// `new` initializes the members of an MIB_IPINTERFACE_ROW entry with default values.
    pub fn new(luid: NET_LUID, family: u16) -> io::Result<Self> {
        let mut row = MibIpInterfaceRow {
            inner: unsafe {
                let mut row: MIB_IPINTERFACE_ROW = mem::zeroed();
                InitializeIpInterfaceEntry(&mut row);
                row.InterfaceLuid = luid;
                row.Family = family;
                row
            },
        };

        row.get()?;
        Ok(row)
    }

    // `get` retrieves IP information for the specified interface on the local computer.
    fn get(&mut self) -> io::Result<()> {
        crate::cvt_dword(unsafe { GetIpInterfaceEntry(&mut self.inner) })?;

        // Patch that fixes SitePrefixLength issue
        // https://stackoverflow.com/questions/54857292/setipinterfaceentry-returns-error-invalid-parameter?noredirect=1
        match self.inner.Family {
            x if x == AF_INET as u16 => {
                if self.inner.SitePrefixLength > 32 {
                    self.inner.SitePrefixLength = 0;
                }
            }
            x if x == AF_INET6 as u16 => {
                if self.inner.SitePrefixLength > 128 {
                    self.inner.SitePrefixLength = 128;
                }
            }
            _ => (),
        }

        Ok(())
    }

    /// `set` sets the properties of an IP interface on the local computer.
    pub fn set(&mut self) -> io::Result<()> {
        crate::cvt_dword(unsafe { SetIpInterfaceEntry(&mut self.inner) })
    }
}
