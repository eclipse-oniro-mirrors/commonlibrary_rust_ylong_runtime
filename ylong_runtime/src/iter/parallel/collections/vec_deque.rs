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

use std::collections::VecDeque;

use super::par_vec_impl;
use crate::iter::parallel::{IntoParIter, ParIter};
par_vec_impl!(VecDeque<T>, Vec<T>, into_iter, impl <T>);

par_vec_impl!(&'a VecDeque<T>, Vec<&'a T>, iter, impl <'a, T>);

par_vec_impl!(&'a mut VecDeque<T>, Vec<&'a mut T>, iter_mut, impl <'a, T>);
