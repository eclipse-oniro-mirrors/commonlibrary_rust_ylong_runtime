// Copyright (c) 2023 Huawei Device Co., Ltd.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io;
use std::mem::size_of;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use libc::{c_int, sockaddr, sockaddr_in, sockaddr_in6, sockaddr_storage, socklen_t};

#[repr(C)]
pub(crate) union SocketAddrLibC {
    v4: libc::sockaddr_in,
    v6: libc::sockaddr_in6,
}

impl SocketAddrLibC {
    pub(crate) fn as_ptr(&self) -> *const sockaddr {
        let ptr: *const SocketAddrLibC = self;
        ptr.cast::<sockaddr>()
    }
}

pub(crate) fn socket_addr_trans(addr: &SocketAddr) -> (SocketAddrLibC, socklen_t) {
    match addr {
        SocketAddr::V4(ref addr) => {
            let sockaddr_in = libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: addr.port().to_be(),
                sin_addr: libc::in_addr {
                    s_addr: u32::from_ne_bytes(addr.ip().octets()),
                },
                sin_zero: [0; 8],
                #[cfg(target_os = "macos")]
                sin_len: 0,
            };

            (
                SocketAddrLibC { v4: sockaddr_in },
                size_of::<libc::sockaddr_in>() as socklen_t,
            )
        }

        SocketAddr::V6(ref addr) => {
            let sin6_addr = libc::in6_addr {
                s6_addr: addr.ip().octets(),
            };

            let sockaddr_in6 = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as libc::sa_family_t,
                sin6_port: addr.port().to_be(),
                sin6_addr,
                sin6_flowinfo: addr.flowinfo(),
                sin6_scope_id: addr.scope_id(),
                #[cfg(target_os = "macos")]
                sin6_len: 0,
            };

            (
                SocketAddrLibC { v6: sockaddr_in6 },
                size_of::<libc::sockaddr_in6>() as socklen_t,
            )
        }
    }
}

/// Converts a `sockaddr_storage` filled in by the kernel (by `recvmsg`,
/// `accept`, ...) back into a Rust `SocketAddr`.
///
/// The fields are read explicitly: the byte layout of `std::net`
/// address types is not guaranteed to match the libc `sockaddr_*`
/// definitions, so a reinterpret cast is not portable.
///
/// # Safety
///
/// `storage` must point to a valid, initialized `sockaddr_storage` whose
/// `ss_family` matches the address actually stored.
pub(crate) unsafe fn trans_addr_2_socket(
    storage: *const sockaddr_storage,
) -> io::Result<SocketAddr> {
    match (*storage).ss_family as c_int {
        libc::AF_INET => {
            let addr = &*storage.cast::<sockaddr_in>();
            Ok(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr)),
                u16::from_be(addr.sin_port),
            )))
        }
        libc::AF_INET6 => {
            let addr = &*storage.cast::<sockaddr_in6>();
            Ok(SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::from(addr.sin6_addr.s6_addr),
                u16::from_be(addr.sin6_port),
                addr.sin6_flowinfo,
                addr.sin6_scope_id,
            )))
        }
        _ => {
            let err = io::Error::new(io::ErrorKind::Other, "Cannot transfer address into socket.");
            Err(err)
        }
    }
}
