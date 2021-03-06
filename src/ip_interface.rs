use std::io;
use std::mem;
use std::ptr::{self, NonNull};

use winapi::shared::ifdef::NET_LUID;
use winapi::shared::netioapi::{
    CancelMibChangeNotify2, GetIpInterfaceEntry, InitializeIpInterfaceEntry,
    NotifyIpInterfaceChange, SetIpInterfaceEntry, MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE,
    PMIB_IPINTERFACE_ROW,
};
use winapi::shared::ntdef::{HANDLE, PVOID};
use winapi::shared::ws2def::{ADDRESS_FAMILY, AF_INET, AF_INET6};

pub struct IpInterface {
    pub inner: MIB_IPINTERFACE_ROW,
}

impl IpInterface {
    /// `new` initializes the members of an MIB_IPINTERFACE_ROW entry with default values.
    pub fn new(luid: NET_LUID, family: u16) -> io::Result<Self> {
        let mut row = IpInterface {
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

    /// `get` retrieves IP information for the specified interface on the local computer.
    pub fn get(&mut self) -> io::Result<()> {
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

    /// `notify_change` registers to be notified for changes to all IP interfaces, IPv4 interfaces,
    /// or IPv6 interfaces on a local computer.
    pub fn notify_change<F>(
        family: ADDRESS_FAMILY,
        callback: F,
    ) -> io::Result<IpInterfaceChangeNotifier>
    where
        F: 'static + FnMut(MIB_NOTIFICATION_TYPE, &IpInterface),
    {
        IpInterfaceChangeNotifier::new(family, callback)
    }
}

type IpInterfaceChangeContext = Box<dyn FnMut(MIB_NOTIFICATION_TYPE, &IpInterface)>;

pub struct IpInterfaceChangeNotifier {
    handle: HANDLE,
    context: NonNull<IpInterfaceChangeContext>,
}

impl IpInterfaceChangeNotifier {
    fn new<F>(family: ADDRESS_FAMILY, callback: F) -> io::Result<IpInterfaceChangeNotifier>
    where
        F: 'static + FnMut(MIB_NOTIFICATION_TYPE, &IpInterface),
    {
        let callback: IpInterfaceChangeContext = Box::new(callback);
        let context =
            NonNull::new(Box::into_raw(Box::new(callback))).expect("Box::into_raw returned null");

        let mut handle = ptr::null_mut();
        crate::cvt_dword(unsafe {
            NotifyIpInterfaceChange(
                family as u16,
                Some(ip_interface_callback),
                context.as_ptr() as *mut _,
                0,
                &mut handle,
            )
        })?;

        Ok(IpInterfaceChangeNotifier { handle, context })
    }
}

impl Drop for IpInterfaceChangeNotifier {
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
unsafe extern "system" fn ip_interface_callback(
    context: PVOID,
    row: PMIB_IPINTERFACE_ROW,
    ntype: MIB_NOTIFICATION_TYPE,
) {
    if !row.is_null() {
        let mut callback: Box<IpInterfaceChangeContext> = Box::from_raw(context as *mut _);
        callback(ntype, &IpInterface { inner: *row });
        // we'll free context in IpInterfaceChangeNotifier::drop
        mem::forget(callback);
    }
}
