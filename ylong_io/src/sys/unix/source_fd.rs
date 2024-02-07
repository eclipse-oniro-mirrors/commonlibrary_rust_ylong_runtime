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
use std::os::unix::io::RawFd;

use crate::{Fd, Interest, Selector, Source, Token};

/// SourceFd allows any type of FD to register with Poll.
#[derive(Debug)]
pub struct SourceFd<'a>(pub &'a RawFd);

impl<'a> Source for SourceFd<'a> {
    fn register(
        &mut self,
        selector: &Selector,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        selector.register(self.get_fd(), token, interests)
    }

    fn reregister(
        &mut self,
        selector: &Selector,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        selector.reregister(self.get_fd(), token, interests)
    }

    fn deregister(&mut self, selector: &Selector) -> io::Result<()> {
        selector.deregister(self.get_fd())
    }

    fn get_fd(&self) -> Fd {
        *self.0
    }
}
