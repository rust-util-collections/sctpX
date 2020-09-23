//!
//! # SCTP server
//!

#![warn(missing_docs, unused_import_braces, unused_extern_crates)]

use myutil::{err::*, *};
use nix::{
    sys::socket::{
        bind, listen, recvfrom, sendto, setsockopt, socket, sockopt, AddressFamily, InetAddr,
        MsgFlags, SockAddr, SockFlag, SockType,
    },
    unistd::close,
};
use std::{mem, net::SocketAddr, os::unix::io::RawFd, sync::Arc};

const DATA_BUF_SIZE_LIMIT: usize = 8 * 1024 * 1024;
const RECV_BUF_SIZE_LIMIT: usize = 64 * 1024 * 1024;

/// SCTP handler
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Hdr {
    fd: RawFd,
}

impl Hdr {
    #[inline(always)]
    fn new(fd: RawFd) -> Hdr {
        Hdr { fd }
    }

    /// 公开此接口,
    /// 回调函数可以按需向对端回复消息
    #[inline(always)]
    pub fn sendto(&self, data: &[u8], peeraddr: &PeerAddr) -> Result<usize> {
        sendto(self.fd, data, &peeraddr.addr, MsgFlags::empty()).c(d!())
    }

    // 接收消息端口必须私有
    #[inline(always)]
    fn recvfrom(&self, data: &mut [u8]) -> Result<(usize, Option<SockAddr>)> {
        recvfrom(self.fd, data).c(d!())
    }
}

impl Drop for Hdr {
    fn drop(&mut self) {
        info_omit!(close(self.fd));
    }
}

/// 客户端地址
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PeerAddr {
    addr: SockAddr,
}

impl PeerAddr {
    #[inline(always)]
    fn new(addr: SockAddr) -> Self {
        PeerAddr { addr }
    }
}

/// Will block the current thread
/// - @addr: server at this address
/// - @data_bs: the max size of one message
/// - @cb: callback to deal with every message
/// - @keep_alive: enable this will get the effect like TCP-keepalive
pub fn start_server(
    addr: &str,
    data_bs: Option<usize>,
    cb: impl Fn(&[u8], Arc<Hdr>, PeerAddr) -> Result<()>,
    keep_alive: bool,
) -> Result<()> {
    let mut siz = data_bs.unwrap_or(4096);
    alt!(siz > DATA_BUF_SIZE_LIMIT, siz = DATA_BUF_SIZE_LIMIT);

    let hdr = gen_hdr(addr, 256 * siz, keep_alive).c(d!())?;
    let mut buf = vec![0; siz].into_boxed_slice();
    loop {
        if let Ok((size, Some(peer))) = info!(hdr.recvfrom(&mut buf)) {
            info_omit!(cb(&buf[0..size], Arc::clone(&hdr), PeerAddr::new(peer)));
        }
    }
}

// -@addr: "192.168.1.2:9458"
// -@recv_bs: max size of system-buffer for sctp recv-queue
// -@keep_alive: enable this will get the effect like TCP-keepalive
fn gen_hdr(addr: &str, recv_bs: usize, keep_alive: bool) -> Result<Arc<Hdr>> {
    let recv_bs = alt!(recv_bs > RECV_BUF_SIZE_LIMIT, RECV_BUF_SIZE_LIMIT, recv_bs);

    let fd = socket(
        AddressFamily::Inet,
        SockType::SeqPacket,
        SockFlag::empty(),
        None,
    )
    .c(d!())?;

    if keep_alive {
        disable_sctp_autoclose(fd).c(d!())?;
    }

    setsockopt(fd, sockopt::ReuseAddr, &true).c(d!())?;
    setsockopt(fd, sockopt::ReusePort, &true).c(d!())?;
    setsockopt(fd, sockopt::RcvBuf, &recv_bs).c(d!())?;

    addr.parse::<SocketAddr>()
        .c(d!())
        .map(|addr| SockAddr::new_inet(InetAddr::from_std(&addr)))
        .and_then(|addr| bind(fd, &addr).c(d!()))
        .and_then(|_| listen(fd, 6).c(d!()))
        .map(|_| Arc::new(Hdr::new(fd)))
}

#[inline(always)]
fn disable_sctp_autoclose(fd: RawFd) -> Result<()> {
    // libc 没有绑定, 手写~
    const SOL_SCTP: libc::c_int = 132;
    const SCTP_AUTOCLOSE: libc::c_int = 4;
    const DISABLE_AUTOCLOSE: libc::c_int = 0;

    if 0 != unsafe {
        libc::setsockopt(
            fd,
            SOL_SCTP,
            SCTP_AUTOCLOSE,
            &DISABLE_AUTOCLOSE as *const libc::c_int as *const libc::c_void,
            mem::size_of::<libc::c_int>() as libc::socklen_t,
        )
    } {
        return Err(eg!("Fail to disable 'SCTP_AUTOCLOSE' !!!"));
    }

    Ok(())
}
