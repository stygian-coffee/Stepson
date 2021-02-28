pub mod bluez;

use std::fmt;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::rc::Rc;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{anyhow, Result};

use dbus::arg::prop_cast;
use dbus::blocking::stdintf::org_freedesktop_dbus::{ObjectManager, Properties};
use dbus::blocking::Connection;
use dbus::strings::Path;

use nix::sys::socket;

use futures::ready;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

const BLUEZ_DEST: &str = "org.bluez";
//const BLUEZ_ADAPTER: &str = "org.bluez.Adapter1";
const BLUEZ_DEVICE: &str = "org.bluez.Device1";
//const BLUEZ_SERVICE: &str = "org.bluez.GattService1";
//const BLUEZ_CHARACTERISTIC: &str = "org.bluez.GattCharacteristic1";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Manager {
    conn: Rc<Connection>,
}

impl Manager {
    pub fn new() -> Result<Self> {
        let conn = Rc::new(Connection::new_system()?);

        Ok(Self { conn })
    }

    pub fn get_devices(&self) -> Result<Vec<Device>> {
        let proxy = self.conn.with_proxy(BLUEZ_DEST, "/", DEFAULT_TIMEOUT);

        let devices = proxy
            .get_managed_objects()?
            .into_iter()
            .filter(|(_k, v)| v.keys().any(|i| i.starts_with(BLUEZ_DEVICE)))
            .map(|(k, _v)| Device::from_dbus_path(self.conn.clone(), &k))
            .collect();

        devices
    }
}

pub struct Device {
    pub path: String,
    pub addr: String,
    pub name: String,
}

impl Device {
    fn from_dbus_path(conn: Rc<Connection>, path: &Path) -> Result<Self> {
        let proxy = conn.with_proxy(BLUEZ_DEST, path, DEFAULT_TIMEOUT);

        let props = proxy.get_all(BLUEZ_DEVICE)?;

        let path = path.to_string();
        let addr = prop_cast::<String>(&props, "Address")
            .ok_or(anyhow!("Unable to get MAC address for device"))?
            .clone();
        let name = prop_cast::<String>(&props, "Name")
            .ok_or(anyhow!("Unable to get name for device {}", addr))?
            .clone();

        Ok(Self { path, addr, name })
    }

    pub fn bt_stream(&self) -> Result<BtStream> {
        BtStream::connect(&self.addr)
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Device")
            .field("path", &self.path)
            .field("addr", &self.addr)
            .field("name", &self.name)
            .finish()
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Device {} {}", self.addr, self.name)
    }
}

#[derive(Debug)]
pub struct BtStream {
    sock: RawFd,
}

impl BtStream {
    pub fn connect(addr: &str) -> Result<Self> {
        let sock = bluez::socket(
            socket::SockType::Stream,
            bluez::BtProtocol::Rfcomm,
            socket::SockFlag::empty(),
        )?;

        let addr = bluez::BtAddr::new(FromStr::from_str(addr)?, 9);

        bluez::connect(sock, &addr)?;

        Ok(Self { sock })
    }

    pub fn shutdown(&self, how: socket::Shutdown) -> io::Result<()> {
        socket::shutdown(self.sock, how).map_err(|e| e.as_errno().unwrap().into())
        // e is always an Errno
    }
}

impl io::Read for BtStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        socket::recv(self.sock, buf, socket::MsgFlags::empty())
            .map_err(|e| e.as_errno().unwrap().into()) // e is always an Errno
    }
}

impl io::Write for BtStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        socket::send(self.sock, buf, socket::MsgFlags::empty())
            .map_err(|e| e.as_errno().unwrap().into()) // e is always an Errno
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsRawFd for BtStream {
    fn as_raw_fd(&self) -> RawFd {
        self.sock
    }
}

#[derive(Debug)]
pub struct AsyncBtStream {
    inner: AsyncFd<BtStream>,
}

impl AsyncBtStream {
    pub fn new(bt: BtStream) -> io::Result<Self> {
        Ok(Self {
            inner: AsyncFd::new(bt)?,
        })
    }
}

impl AsyncRead for AsyncBtStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let unpinned = self.get_mut();

        // Safety: the later code in this method does not de-initialize anything
        let buf_inner =
            unsafe { &mut *(buf.unfilled_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8]) };

        loop {
            let mut guard = ready!(unpinned.inner.poll_read_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().read(buf_inner)) {
                Ok(n) => {
                    buf.advance(n?);
                    return Poll::Ready(Ok(()));
                }
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncBtStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let unpinned = self.get_mut();

        loop {
            let mut guard = ready!(unpinned.inner.poll_write_ready_mut(cx))?;

            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.inner.get_ref().shutdown(socket::Shutdown::Write)?;
        Poll::Ready(Ok(()))
    }
}
