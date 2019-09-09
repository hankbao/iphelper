use std::io;
use std::mem;
use std::ptr::{self, NonNull};

use winapi::shared::netioapi::{
    CancelMibChangeNotify2, CreateIpForwardEntry2, DeleteIpForwardEntry2, FreeMibTable,
    GetIpForwardTable2, InitializeIpForwardEntry, NotifyRouteChange2, MIB_IPFORWARD_ROW2,
    MIB_NOTIFICATION_TYPE, PMIB_IPFORWARD_ROW2, PMIB_IPFORWARD_TABLE2,
};
use winapi::shared::ntdef::{HANDLE, PVOID};
use winapi::shared::ws2def::{ADDRESS_FAMILY, AF_UNSPEC};

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

type RouteChange2Context = Box<dyn FnMut(MIB_NOTIFICATION_TYPE, &MibIpForwardRow2)>;

pub struct RouteChange2Notifier {
    handle: HANDLE,
    context: NonNull<RouteChange2Context>,
}

impl RouteChange2Notifier {
    pub fn new<F>(callback: F) -> io::Result<RouteChange2Notifier>
    where
        F: 'static + FnMut(MIB_NOTIFICATION_TYPE, &MibIpForwardRow2),
    {
        let callback: RouteChange2Context = Box::new(callback);
        let context =
            NonNull::new(Box::into_raw(Box::new(callback))).expect("Box::into_raw returned null");

        let mut handle = ptr::null_mut();
        crate::cvt_dword(unsafe {
            NotifyRouteChange2(
                AF_UNSPEC as u16,
                Some(route_change2_callback),
                context.as_ptr() as *mut _,
                0,
                &mut handle,
            )
        })?;

        Ok(RouteChange2Notifier { handle, context })
    }
}

impl Drop for RouteChange2Notifier {
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
unsafe extern "system" fn route_change2_callback(
    context: PVOID,
    row: PMIB_IPFORWARD_ROW2,
    ntype: MIB_NOTIFICATION_TYPE,
) {
    let mut callback: Box<RouteChange2Context> = Box::from_raw(context as *mut _);
    if !row.is_null() {
        callback(ntype, &MibIpForwardRow2 { inner: *row });
    }

    // we'll free context in RouteChange2Notifier::drop
    mem::forget(callback);
}
