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

//! Event-driven non-blocking Tcp/Udp for unix

macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

cfg_tcp! {
    mod tcp;
    pub use self::tcp::{TcpListener, TcpStream};
}

cfg_udp! {
    mod udp;
    pub use self::udp::{UdpSocket, ConnectedUdpSocket};
}

mod uds;
pub use uds::{SocketAddr, UnixDatagram, UnixListener, UnixStream};

#[cfg(target_os = "linux")]
mod epoll;
#[cfg(target_os = "linux")]
pub use epoll::{Event, Events, Selector};

mod socket_addr;

#[cfg(target_os = "macos")]
mod kqueue;
#[cfg(target_os = "macos")]
pub use kqueue::{Event, Events, Selector};

pub(crate) mod socket;
mod waker;

pub(crate) use waker::WakerInner;

mod source_fd;
pub use source_fd::SourceFd;
