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

//! Event-driven nonblocking net-io components.

mod poll;
pub use poll::Poll;

mod token;
pub use token::Token;

pub mod sys;
#[cfg(feature = "udp")]
pub use sys::{ConnectedUdpSocket, UdpSocket};
pub use sys::{Event, EventTrait, Events, Selector};
#[cfg(unix)]
pub use sys::{SocketAddr, UnixDatagram, UnixListener, UnixStream};
#[cfg(feature = "tcp")]
pub use sys::{TcpListener, TcpStream};

/// unix-specific
#[cfg(unix)]
pub mod unix {
    pub use crate::sys::SourceFd;
}

mod interest;
pub use interest::Interest;

mod source;
pub use source::{Fd, Source};

mod waker;
pub use waker::Waker;

#[cfg(not(any(feature = "tcp", feature = "udp")))]
std::compile_error!("tcp and udp feature must be enabled one of them for this crate.");
