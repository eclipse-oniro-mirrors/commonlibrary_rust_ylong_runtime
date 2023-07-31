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

use std::convert::TryInto;
use std::fmt::Error;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::time::{Duration, Instant};

use crate::time::wheel::{TimeOut, Wheel};
use crate::time::Clock;
cfg_ffrt! {
    use std::mem::MaybeUninit;
    use std::sync::Once;
}

// Time Driver
pub(crate) struct TimeDriver {
    start_time: Instant,
    pub(crate) wheel: Mutex<Wheel>,
}

pub(crate) struct TimeHandle {
    inner: Arc<TimeDriver>,
}

impl TimeDriver {
    #[cfg(not(feature = "ffrt"))]
    pub(crate) fn initialize() -> (TimeHandle, Arc<TimeDriver>) {
        let driver = Arc::new(TimeDriver {
            start_time: Instant::now(),
            wheel: Mutex::new(Wheel::new()),
        });
        (
            TimeHandle {
                inner: driver.clone(),
            },
            driver,
        )
    }

    #[cfg(feature = "ffrt")]
    pub(crate) fn get_ref() -> &'static Self {
        static mut DRIVER: MaybeUninit<TimeDriver> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                DRIVER.write(Self {
                    start_time: Instant::now(),
                    wheel: Mutex::new(Wheel::new()),
                });
            });

            &*DRIVER.as_ptr()
        }
    }

    pub(crate) fn start_time(&self) -> Instant {
        self.start_time
    }

    pub(crate) fn timer_register(&self, clock_entry: NonNull<Clock>) -> Result<u64, Error> {
        let mut lock = self.wheel.lock().unwrap();
        lock.insert(clock_entry)
    }

    pub(crate) fn timer_cancel(&self, clock_entry: NonNull<Clock>) {
        let mut lock = self.wheel.lock().unwrap();
        lock.cancel(clock_entry);
    }

    pub(crate) fn run(&self) -> Option<Duration> {
        let now = Instant::now();
        let now = now
            .checked_duration_since(self.start_time())
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX);

        let mut waker_list: [Option<Waker>; 32] = Default::default();
        let mut waker_idx = 0;

        let mut lock = self.wheel.lock().unwrap();

        let mut timeout = None;

        loop {
            match lock.poll(now) {
                TimeOut::ClockEntry(mut clock_entry) => {
                    let elapsed = lock.elapsed();
                    lock.set_last_elapsed(elapsed);

                    // Unsafe access to timer_handle is only unsafe when Sleep Drop,
                    // but does not let `Sleep` go to `Ready` before access to timer_handle fetched
                    // by poll.
                    let clock_handle = unsafe { clock_entry.as_mut() };
                    waker_list[waker_idx] = clock_handle.take_waker();
                    waker_idx += 1;

                    clock_handle.set_result(true);

                    if waker_idx == waker_list.len() {
                        for waker in waker_list.iter_mut() {
                            waker.take().unwrap().wake();
                        }

                        waker_idx = 0;
                    }
                }
                TimeOut::Duration(duration) => {
                    timeout = Some(duration);
                    break;
                }
                TimeOut::None => break,
            }
        }

        drop(lock);
        for waker in waker_list[0..waker_idx].iter_mut() {
            waker.take().unwrap().wake();
        }
        timeout
    }
}

impl Deref for TimeHandle {
    type Target = Arc<TimeDriver>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
