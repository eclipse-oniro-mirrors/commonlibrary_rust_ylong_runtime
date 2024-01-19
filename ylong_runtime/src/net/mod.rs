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

//! Asynchronous TCP/UDP binding for `ylong_runtime`

pub(crate) use driver::IoHandle;
pub(crate) use ready::{Ready, ReadyEvent};
pub(crate) use schedule_io::{ScheduleIO, Tick};

pub(crate) mod async_source;
pub(crate) mod sys;
pub(crate) use async_source::AsyncSource;

pub(crate) mod driver;
pub(crate) mod ready;
pub(crate) mod schedule_io;

cfg_net! {
    pub use sys::{TcpListener, TcpStream};
    pub use sys::{UdpSocket, ConnectedUdpSocket};
    pub use sys::{SplitReadHalf, SplitWriteHalf, BorrowReadHalf, BorrowWriteHalf};
    #[cfg(unix)]
    pub use sys::{UnixListener, UnixStream, UnixDatagram};
    pub use sys::ToSocketAddrs;
}

#[cfg(not(feature = "ffrt"))]
pub(crate) use driver::IoDriver;
