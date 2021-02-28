use std::mem;
use std::os::unix::io::RawFd;

use libc;
use macaddr::MacAddr6;
use nix::errno::Errno;
use nix::sys::socket;
use nix::Result;

#[allow(non_camel_case_types)]
type bdaddr_t = [u8; 6];

#[repr(C)]
#[derive(Debug)]
pub struct sockaddr_rc {
    pub sa_family: libc::sa_family_t,
    pub rc_bdaddr: bdaddr_t,
    pub rc_channel: u8,
}

#[derive(Debug)]
pub struct BtAddr(pub sockaddr_rc);

impl BtAddr {
    pub fn new(addr: MacAddr6, channel: u8) -> Self {
        Self(sockaddr_rc {
            sa_family: libc::AF_BLUETOOTH as u16,
            rc_bdaddr: addr.into_array(),
            rc_channel: channel,
        })
    }
}

#[repr(i32)]
#[derive(Debug)]
pub enum BtProtocol {
    L2cap = 0,
    Hci = 1,
    Sco = 2,
    Rfcomm = 3,
    Bnep = 4,
    Cmtp = 5,
    Hidp = 6,
    Avdtp = 7,
}

pub fn socket<T: Into<BtProtocol>>(
    ty: socket::SockType,
    protocol: T,
    flags: socket::SockFlag,
) -> Result<RawFd> {
    let domain = libc::AF_BLUETOOTH;
    let protocol = protocol.into() as libc::c_int;

    let mut ty = ty as libc::c_int;
    ty |= flags.bits();

    let res = unsafe { libc::socket(domain, ty, protocol) };

    Errno::result(res)
}

pub fn connect(fd: RawFd, addr: &BtAddr) -> Result<()> {
    //TODO understand why we are reversing the MAC address
    let mut rev = addr.0.rc_bdaddr.clone();
    rev.reverse();
    let addr = BtAddr(sockaddr_rc {
        sa_family: addr.0.sa_family,
        rc_bdaddr: rev,
        rc_channel: addr.0.rc_channel,
    });

    let res = unsafe {
        let (ptr, len) = (
            &*(&addr.0 as *const sockaddr_rc as *const libc::sockaddr),
            mem::size_of_val(&addr) as libc::socklen_t,
        );
        libc::connect(fd, ptr, len)
    };

    Errno::result(res).map(drop)
}
