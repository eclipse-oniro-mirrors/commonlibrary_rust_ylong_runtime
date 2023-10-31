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

//! Asynchronous tasks that wraps the future and get scheduled by the runtime.

pub mod builder;
pub mod join_handle;
mod join_set;
mod raw;
pub(crate) mod state;
mod task_handle;
mod waker;
pub(crate) mod yield_now;
use std::future::Future;
use std::mem;
use std::ptr::NonNull;
use std::sync::Weak;

pub use builder::TaskBuilder;
pub use join_handle::JoinHandle;
pub use join_set::JoinSet;
pub use yield_now::yield_now;

use crate::executor::Schedule;
use crate::task::raw::{Header, RawTask, TaskMngInfo};

pub(crate) enum VirtualTableType {
    Ylong,
    #[cfg(feature = "ffrt")]
    Ffrt,
}

#[cfg(not(feature = "ffrt"))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Qos levels.
pub enum Qos {
    /// Inherits parent's qos level
    Inherent = -1,
    /// Lowest qos
    Background,
    /// Utility qos
    Utility,
    /// Default qos
    Default,
    /// User initialiated qos
    UserInitiated,
    /// Deadline qos
    DeadlineRequest,
    /// Highest qos
    UserInteractive,
}

#[cfg(feature = "ffrt")]
pub use ylong_ffrt::Qos;

#[repr(transparent)]
#[derive(Clone)]
pub(crate) struct Task(pub(crate) RawTask);

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    pub(crate) fn run(self) {
        self.0.run();
        mem::forget(self);
    }
    #[cfg(not(feature = "ffrt"))]
    pub(crate) fn shutdown(self) {
        self.0.shutdown();
    }
}

impl Task {
    pub(crate) unsafe fn from_raw(ptr: NonNull<Header>) -> Task {
        Task(RawTask::form_raw(ptr))
    }

    pub(crate) fn create_task<T, S>(
        builder: &TaskBuilder,
        scheduler: Weak<S>,
        task: T,
        virtual_table_type: VirtualTableType,
    ) -> (Task, JoinHandle<T::Output>)
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
        S: Schedule,
    {
        let raw = Task::create_raw_task::<T, S>(builder, scheduler, task, virtual_table_type);

        let join = JoinHandle::new(raw);
        (Task(raw), join)
    }

    pub(crate) fn create_raw_task<T, S>(
        builder: &TaskBuilder,
        scheduler: Weak<S>,
        task: T,
        virtual_table_type: VirtualTableType,
    ) -> RawTask
    where
        T: Future,
        S: Schedule,
    {
        let ptr = Box::into_raw(TaskMngInfo::<T, S>::new(
            builder,
            scheduler,
            task,
            virtual_table_type,
        ));
        let non_ptr = NonNull::new(ptr as *mut Header);
        let ptr = if let Some(ptr) = non_ptr {
            ptr
        } else {
            panic!("task mem is null because not enough memory is available");
        };
        RawTask { ptr }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        self.0.drop_ref()
    }
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
    let rt = crate::executor::global_default_async();
    rt.block_on(task)
}
