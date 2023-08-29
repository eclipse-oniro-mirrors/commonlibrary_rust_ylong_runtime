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

#![cfg(feature = "fs")]

use std::fs;
use std::io::SeekFrom;

use ylong_runtime::fs::{File, OpenOptions};
use ylong_runtime::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

/// SDV test cases for asynchronous file writing
///
/// # Brief
/// 1. Generate an asynchronous file IO with create.
/// 2. Write to an array of length 5.
/// 3. Start another task to read and write the same data as you read.
#[test]
fn sdv_async_fs_write() {
    let handle = ylong_runtime::spawn(async move {
        let mut file = File::create("./tests/tmp_file").await.unwrap();
        let buf = "hello".as_bytes().to_vec();
        let res = file.write(&buf).await.unwrap();
        assert_eq!(res, 5);
        file.sync_all().await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();

    let handle1 = ylong_runtime::spawn(async move {
        let mut file = File::open("./tests/tmp_file").await.unwrap();
        let mut buf = [0; 5];
        let res = file.read(&mut buf).await.unwrap();
        assert_eq!(res, 5);
        assert_eq!(&buf, "hello".as_bytes());
    });
    ylong_runtime::block_on(handle1).unwrap();
    fs::remove_file("./tests/tmp_file").unwrap();
}

/// SDV test cases for asynchronous file reading
///
/// # Brief
/// 1. Generate an asynchronous file IO with create.
/// 2. Write to an array of length 5.
/// 3. Start two tasks to read, write and read the same data.
#[test]
fn sdv_async_fs_read() {
    let handle = ylong_runtime::spawn(async move {
        let mut file = File::create("./tests/tmp_file2").await.unwrap();
        let buf = vec![1, 2, 3, 4, 5];
        let res = file.write(&buf).await.unwrap();
        assert_eq!(res, 5);
        file.sync_all().await.unwrap();

        let mut file = File::open("./tests/tmp_file2").await.unwrap();
        let mut buf = [0; 5];
        let res = file.read(&mut buf).await.unwrap();
        assert_eq!(res, 5);
        assert_eq!(buf, [1, 2, 3, 4, 5]);
    });
    ylong_runtime::block_on(handle).unwrap();

    let handle2 = ylong_runtime::spawn(async move {
        let mut file = File::open("./tests/tmp_file2").await.unwrap();
        let mut buf = [0; 5];
        let res = file.read(&mut buf).await.unwrap();
        assert_eq!(res, 5);
        assert_eq!(buf, [1, 2, 3, 4, 5]);
    });
    ylong_runtime::block_on(handle2).unwrap();
    fs::remove_file("./tests/tmp_file2").unwrap();
}

/// SDV test cases for asynchronous file multi-threaded read and write
///
/// # Brief
/// 1. Generate an asynchronous file IO with create.
/// 2. Start a task to perform a write operation.
/// 3. Start another task to perform a read operation.
#[test]
fn sdv_async_fs_rw() {
    let handle = ylong_runtime::spawn(async move {
        let _ = File::create("./tests/tmp_file3").await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();

    let handle = ylong_runtime::spawn(async move {
        let mut file = OpenOptions::new()
            .append(true)
            .open("./tests/tmp_file3")
            .await
            .unwrap();
        let buf = vec![45, 46, 47, 48, 49];
        let res = file.write(&buf).await.unwrap();
        assert_eq!(res, 5);

        let mut buf = [0; 16384];
        for (i, val) in buf.iter_mut().enumerate() {
            *val = i as u8;
        }
        let ret = file.write_all(&buf).await;
        assert!(ret.is_ok());
        file.sync_all().await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();

    let handle2 = ylong_runtime::spawn(async move {
        let mut file = File::open("./tests/tmp_file3").await;
        while file.is_err() {
            file = File::open("./tests/tmp_file3").await;
        }
        let mut file = file.unwrap();

        let mut buf = [0; 3];
        let mut ret = file.read(&mut buf).await.unwrap();
        while ret == 0 {
            ret = file.read(&mut buf).await.unwrap();
        }
        assert_eq!(ret, 3);

        let mut buf = [0; 2];
        let ret = file.read(&mut buf).await.unwrap();
        assert_eq!(ret, 2);

        let mut buf = Vec::new();
        let mut ret = file.read_to_end(&mut buf).await.unwrap();
        while ret == 0 {
            ret = file.read_to_end(&mut buf).await.unwrap();
        }
        assert_eq!(ret, 16384);
        let mut buf2 = [0; 16384];
        for (i, val) in buf2.iter_mut().enumerate() {
            *val = i as u8;
        }
        assert_eq!(&buf, &buf2);
    });
    ylong_runtime::block_on(handle2).unwrap();
    fs::remove_file("./tests/tmp_file3").unwrap();
}

/// SDV test cases for Asynchronous file multi-threaded read and write
///
/// # Brief
/// 1. Generate an asynchronous file IO with create.
/// 2. Start a task to write a large amount of data.
/// 3. Start another task for reading large amounts of data.
#[test]
fn sdv_async_fs_read_to_end() {
    let handle = ylong_runtime::spawn(async move {
        let mut file = File::create("./tests/tmp_file7").await.unwrap();
        let buf = [2; 40000];
        file.write_all(&buf).await.unwrap();
        file.sync_all().await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();
    let handle1 = ylong_runtime::spawn(async move {
        let mut file = File::open("./tests/tmp_file7").await.unwrap();
        let mut vec_buf = Vec::new();
        let ret = file.read_to_end(&mut vec_buf).await.unwrap();
        assert_eq!(ret, 40000);
    });
    ylong_runtime::block_on(handle1).unwrap();
    fs::remove_file("./tests/tmp_file7").unwrap();
}

/// SDV test cases for asynchronous file Seek
///
/// # Brief
/// 1. Generate an asynchronous file IO with create.
/// 2. Start a task to perform a write operation.
/// 3. Start another task for seek and read operations.
#[test]
fn sdv_async_fs_seek() {
    let handle = ylong_runtime::spawn(async move {
        let mut file = File::create("./tests/tmp_file4").await.unwrap();
        let buf = vec![65, 66, 67, 68, 69, 70, 71, 72, 73];
        let res = file.write(&buf).await.unwrap();
        assert_eq!(res, 9);
        file.sync_all().await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();

    let handle2 = ylong_runtime::spawn(async move {
        let mut file = File::open("./tests/tmp_file4").await.unwrap();
        let ret = file.seek(SeekFrom::Current(3)).await.unwrap();
        assert_eq!(ret, 3);

        let mut buf = [0; 1];
        let ret = file.read(&mut buf).await.unwrap();
        assert_eq!(ret, 1);
        assert_eq!(buf, [68]);

        let ret = file.seek(SeekFrom::Current(1)).await.unwrap();
        assert_eq!(ret, 5);

        let mut buf = [0; 1];
        let ret = file.read(&mut buf).await.unwrap();
        assert_eq!(ret, 1);
        assert_eq!(buf, [70]);

        let ret = file.seek(SeekFrom::Current(2)).await.unwrap();
        assert_eq!(ret, 8);

        let mut buf = [0; 2];
        let ret = file.read(&mut buf).await.unwrap();
        assert_eq!(ret, 1);
        assert_eq!(buf, [73, 0]);

        let ret = file.seek(SeekFrom::Start(0)).await.unwrap();
        assert_eq!(ret, 0);
        let mut buf = [0; 9];
        let ret = file.read(&mut buf).await.unwrap();
        assert_eq!(ret, 9);
        assert_eq!(buf, [65, 66, 67, 68, 69, 70, 71, 72, 73]);

        let ret = file.seek(SeekFrom::End(-1)).await.unwrap();
        assert_eq!(ret, 8);
        let mut buf = [0; 2];
        let ret = file.read(&mut buf).await.unwrap();
        assert_eq!(ret, 1);
        assert_eq!(buf, [73, 0]);
    });

    ylong_runtime::block_on(handle2).unwrap();
    fs::remove_file("./tests/tmp_file4").unwrap();
}

/// SDV test cases for Asynchronous file set permission
///
/// # Brief
/// 1. Generate an asynchronous file IO with create.
/// 2. Asynchronously get the permissions of the file.
/// 3. Change the permission to read only, set it to this file.
#[test]
fn sdv_async_fs_set_permission() {
    let handle = ylong_runtime::spawn(async move {
        let file = File::create("./tests/tmp_file5").await.unwrap();
        let mut perms = file.metadata().await.unwrap().permissions();
        perms.set_readonly(true);
        let ret = file.set_permissions(perms).await;
        assert!(ret.is_ok());
        let mut perms = file.metadata().await.unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        let ret = file.set_permissions(perms).await;
        assert!(ret.is_ok());
    });
    ylong_runtime::block_on(handle).unwrap();
    fs::remove_file("./tests/tmp_file5").unwrap();
}

/// SDV test cases for asynchronous file sync
///
/// # Brief
/// 1. Generate an asynchronous file IO with create.
/// 2. Call sync_all and sync_data after asynchronous write.
#[test]
fn sdv_async_fs_sync_all() {
    let handle = ylong_runtime::spawn(async move {
        let mut file = File::create("./tests/tmp_file6").await.unwrap();
        let buf = [2; 20000];
        let ret = file.write_all(&buf).await;
        assert!(ret.is_ok());
        let ret = file.sync_all().await;
        assert!(ret.is_ok());

        let buf = [2; 20000];
        let ret = file.write_all(&buf).await;
        assert!(ret.is_ok());
        let ret = file.sync_data().await;
        assert!(ret.is_ok());
    });
    ylong_runtime::block_on(handle).unwrap();
    fs::remove_file("./tests/tmp_file6").unwrap();
}
