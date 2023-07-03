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

mod async_buf_reader;
mod async_buf_writer;

pub use async_buf_reader::AsyncBufReader;
pub use async_buf_writer::AsyncBufWriter;

// We took this value from std::io
pub const DEFAULT_BUF_SIZE: usize = if cfg!(target_os = "espidf") {
    512
} else {
    8 * 1024
};
