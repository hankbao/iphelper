use std::io;
use std::mem;
use std::ptr::{self, NonNull};

use winapi::shared::netioapi::{
    CancelMibChangeNotify2, CreateUnicastIpAddressEntry, DeleteUnicastIpAddressEntry, FreeMibTable,
    GetUnicastIpAddressTable, InitializeUnicastIpAddressEntry, NotifyUnicastIpAddressChange,
    MIB_NOTIFICATION_TYPE, MIB_UNICASTIPADDRESS_ROW, PMIB_UNICASTIPADDRESS_ROW,
    PMIB_UNICASTIPADDRESS_TABLE,
};
use winapi::shared::ntdef::{HANDLE, PVOID};
use winapi::shared::ws2def::{ADDRESS_FAMILY, AF_UNSPEC};

pub struct UnicastIpAddress {
    pub inner: MIB_UNICASTIPADDRESS_ROW,
}

impl UnicastIpAddress {
    /// `new` initializes a MibUnicastIPAddressRow structure with default values for a unicast IP address entry on the local computer.
    pub fn new() -> Self {
        UnicastIpAddress::default()
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

impl Default for UnicastIpAddress {
    fn default() -> Self {
        UnicastIpAddress {
            inner: unsafe {
                let mut row: MIB_UNICASTIPADDRESS_ROW = mem::zeroed();
                InitializeUnicastIpAddressEntry(&mut row);
                row
            },
        }
    }
}

pub struct UnicastIpAddressTable(PMIB_UNICASTIPADDRESS_TABLE);

impl UnicastIpAddressTable {
    /// `new` initializes a MIB_UNICASTIPADDRESS_TABLE table pointed by a
    /// PMIB_UNICASTIPADDRESS_TABLE according to the address family
    pub fn new(family: ADDRESS_FAMILY) -> io::Result<Self> {
        unsafe {
            let mut table: PMIB_UNICASTIPADDRESS_TABLE = ptr::null_mut();
            crate::cvt_dword(GetUnicastIpAddressTable(family, &mut table))?;
            Ok(UnicastIpAddressTable(table))
        }
    }

    fn_table_iter! {UnicastIpAddressTableIter}
}

impl Drop for UnicastIpAddressTable {
    fn drop(&mut self) {
        unsafe {
            FreeMibTable(self.0 as *mut _);
        }
    }
}

declare_table_iter! {
    UnicastIpAddressTableIter,
    UnicastIpAddress,
    MIB_UNICASTIPADDRESS_ROW
}

type UnicastIpAddressChangeContext = Box<dyn FnMut(MIB_NOTIFICATION_TYPE, &UnicastIpAddress)>;

pub struct UnicastIpAddressChangeNotifier {
    handle: HANDLE,
    context: NonNull<UnicastIpAddressChangeContext>,
}

impl UnicastIpAddressChangeNotifier {
    pub fn new<F>(callback: F) -> io::Result<UnicastIpAddressChangeNotifier>
    where
        F: 'static + FnMut(MIB_NOTIFICATION_TYPE, &UnicastIpAddress),
    {
        let callback: UnicastIpAddressChangeContext = Box::new(callback);
        let context =
            NonNull::new(Box::into_raw(Box::new(callback))).expect("Box::into_raw returned null");

        let mut handle = ptr::null_mut();
        crate::cvt_dword(unsafe {
            NotifyUnicastIpAddressChange(
                AF_UNSPEC as u16,
                Some(unicast_ip_address_callback),
                context.as_ptr() as *mut _,
                0,
                &mut handle,
            )
        })?;

        Ok(UnicastIpAddressChangeNotifier { handle, context })
    }
}

impl Drop for UnicastIpAddressChangeNotifier {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                CancelMibChangeNotify2(self.handle);
            }
            drop(Box::from_raw(self.context.as_ptr()));
        }
    }
}

#[allow(clippy::cast_ptr_alignment)]
unsafe extern "system" fn unicast_ip_address_callback(
    context: PVOID,
    row: PMIB_UNICASTIPADDRESS_ROW,
    ntype: MIB_NOTIFICATION_TYPE,
) {
    if !row.is_null() {
        let mut callback: Box<UnicastIpAddressChangeContext> = Box::from_raw(context as *mut _);
        callback(ntype, &UnicastIpAddress { inner: *row });
        // we'll free context in UnicastIpAddressChangeNotifier::drop
        mem::forget(callback);
    }
}
