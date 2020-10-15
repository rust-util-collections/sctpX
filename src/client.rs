//!
//! # SCTP client
//!

#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

use myutil::{err::*, *};
use nix::{
    sys::socket::{
        recvfrom, sendto, socket, AddressFamily, InetAddr, MsgFlags, SockAddr,
        SockFlag, SockType,
    },
    unistd::close,
};
use std::{net::SocketAddr, os::unix::io::RawFd};

/// SCTP handler
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Hdr {
    fd: RawFd,
}

impl Hdr {
    /// create a new handler
    #[inline(always)]
    pub fn new() -> Result<Hdr> {
        socket(
            AddressFamily::Inet,
            SockType::SeqPacket,
            SockFlag::empty(),
            None,
        )
        .c(d!())
        .map(|fd| Hdr { fd })
    }

    /// sendmsg to server
    #[inline(always)]
    pub fn sendto_straddr(
        &self,
        data: &[u8],
        server_addr: &str,
    ) -> Result<usize> {
        server_addr
            .parse::<SocketAddr>()
            .c(d!())
            .and_then(|addr| self.sendto(data, addr).c(d!()))
    }

    /// sendmsg to server
    #[inline(always)]
    pub fn sendto(
        &self,
        data: &[u8],
        server_addr: SocketAddr,
    ) -> Result<usize> {
        let peeraddr = SockAddr::new_inet(InetAddr::from_std(&server_addr));
        sendto(self.fd, data, &peeraddr, MsgFlags::empty()).c(d!())
    }

    /// recvmsg from server
    #[inline(always)]
    pub fn recvfrom(
        &self,
        data: &mut [u8],
    ) -> Result<(usize, Option<SockAddr>)> {
        recvfrom(self.fd, data).c(d!())
    }
}

impl Drop for Hdr {
    fn drop(&mut self) {
        info_omit!(close(self.fd));
    }
}
