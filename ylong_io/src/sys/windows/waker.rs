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
use std::sync::Arc;

use crate::sys::windows::iocp::CompletionPort;
use crate::{Event, Selector, Token};

#[derive(Debug)]
pub struct WakerInner {
    token: Token,
    completion_port: Arc<CompletionPort>,
}

impl WakerInner {
    /// Creates a new WakerInner.
    pub fn new(selector: &Selector, token: Token) -> io::Result<WakerInner> {
        Ok(WakerInner {
            token,
            completion_port: selector.clone_cp(),
        })
    }

    /// Waker allows cross-thread waking of Poll.
    pub fn wake(&self) -> io::Result<()> {
        let mut event = Event::new(self.token);
        event.set_readable();

        self.completion_port.post(event.as_completion_status())
    }
}
