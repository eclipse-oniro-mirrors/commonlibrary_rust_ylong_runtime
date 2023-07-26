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

//! # ylong_runtime
//! A runtime for writing IO-bounded and CPU-bounded applications.

#[cfg(all(
    feature = "ffrt",
    any(feature = "current_thread_runtime", feature = "multi_instance_runtime")
))]
compile_error!("Feature ffrt can not be enabled with feature current_thread_runtime or feature multi_instance_runtime");

#[cfg(all(feature = "ffrt", not(target_os = "linux")))]
compile_error!("Feature ffrt only works on linux currently");

extern crate core;

use std::future::Future;
#[macro_use]
pub(crate) mod macros;

use crate::error::ScheduleError;
use crate::task::{JoinHandle, Task, TaskBuilder};

pub mod builder;
pub mod error;
pub mod executor;

#[cfg(feature = "ffrt")]
pub(crate) mod ffrt;
#[cfg(feature = "fs")]
pub mod fs;
pub mod futures;
pub mod io;
pub mod iter;
#[cfg(feature = "macros")]
mod select;
#[cfg(feature = "macros")]
pub use ylong_runtime_macros::main;
#[cfg(feature = "macros")]
pub use ylong_runtime_macros::test;
pub(crate) mod spawn;
#[cfg(feature = "sync")]
pub mod sync;
pub mod task;
#[cfg(feature = "time")]
pub mod time;
pub mod util;

cfg_io! {
    pub mod net;
}

/// Using the default task setting, spawns a task onto the global runtime.
pub fn spawn<T, R>(task: T) -> JoinHandle<R>
where
    T: Future<Output = R>,
    T: Send + 'static,
    R: Send + 'static,
{
    TaskBuilder::new().spawn(task)
}

/// Using the default task setting, spawns a blocking task.
pub fn spawn_blocking<T, R>(task: T) -> JoinHandle<R>
where
    T: FnOnce() -> R,
    T: Send + 'static,
    R: Send + 'static,
{
    TaskBuilder::new().spawn_blocking(task)
}

/// Blocks the current thread until the `Future` passed in is completed.
pub fn block_on<T>(task: T) -> T::Output
where
    T: Future,
{
    let rt = executor::global_default_async();
    rt.block_on(task)
}
