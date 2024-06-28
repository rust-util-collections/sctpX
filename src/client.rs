//!
//! # SCTP client
//!

#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

use nix::{
    sys::socket::{
        recvfrom, sendto, socket, AddressFamily, MsgFlags, SockFlag, SockType,
        SockaddrStorage,
    },
    unistd::close,
};
use ruc::*;
use std::{
    net::SocketAddr,
    os::{fd::AsRawFd, unix::io::OwnedFd},
};

/// SCTP handler
#[derive(Debug)]
pub struct Hdr {
    fd: OwnedFd,
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
        sendto(
            self.fd.as_raw_fd(),
            data,
            &SockaddrStorage::from(server_addr),
            MsgFlags::empty(),
        )
        .c(d!())
    }

    /// recvmsg from server
    #[inline(always)]
    pub fn recvfrom(
        &self,
        data: &mut [u8],
    ) -> Result<(usize, Option<SockaddrStorage>)> {
        recvfrom(self.fd.as_raw_fd(), data).c(d!())
    }
}

impl Drop for Hdr {
    fn drop(&mut self) {
        ruc::info_omit!(close(self.fd.as_raw_fd()));
    }
}
