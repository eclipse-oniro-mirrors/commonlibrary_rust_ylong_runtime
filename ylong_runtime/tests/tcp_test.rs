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

#![cfg(feature = "net")]

use ylong_runtime::io::{AsyncReadExt, AsyncWriteExt};
use ylong_runtime::net::{TcpListener, TcpStream};

const ADDR: &str = "127.0.0.1:0";

/// SDV test cases for `TcpListener`.
///
/// # Brief
/// 1. Bind `TcpListener`.
/// 2. Call local_addr(), set_ttl(), ttl(), take_error().
/// 3. Check result is correct.
#[test]
fn sdv_tcp_listener_interface() {
    let handle = ylong_runtime::spawn(async {
        let server = TcpListener::bind(ADDR).await.unwrap();

        server.set_ttl(101).unwrap();
        assert_eq!(server.ttl().unwrap(), 101);

        assert!(server.take_error().unwrap().is_none());
    });
    ylong_runtime::block_on(handle).unwrap();
}

/// SDV test cases for `TcpStream`.
///
/// # Brief
/// 1. Bind `TcpListener` and wait for `accept()`.
/// 2. After accept, try to write buf.
/// 2. `TcpStream` connect to listener and try to read buf.
/// 4. Check result is correct.
#[test]
fn sdv_tcp_stream_try() {
    let handle = ylong_runtime::spawn(async move {
        let listener = TcpListener::bind(ADDR).await.unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = ylong_runtime::spawn(async move {
            let mut stream = TcpStream::connect(addr).await;
            while stream.is_err() {
                stream = TcpStream::connect(addr).await;
            }
            let stream = stream.unwrap();
            let mut buf = vec![0; 5];
            stream.readable().await.unwrap();
            stream.try_read(&mut buf).unwrap();
            assert_eq!(buf, b"hello");
        });

        let (stream, _) = listener.accept().await.unwrap();
        stream.writable().await.unwrap();
        stream.try_write(b"hello").unwrap();

        handle.await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();
}

/// SDV test cases for `TcpStream`.
///
/// # Brief
/// 1. Bind `TcpListener` and wait for `accept()`.
/// 2. `TcpStream` connect to listener.
/// 3. Call peer_addr(), local_addr(), set_ttl(), ttl(), set_nodelay(),
///    nodelay(), take_error().
/// 4. Check result is correct.
#[test]
fn sdv_tcp_stream_basic() {
    let handle = ylong_runtime::spawn(async move {
        let listener = TcpListener::bind(ADDR).await.unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = ylong_runtime::spawn(async move {
            let mut stream = TcpStream::connect(addr).await;
            while stream.is_err() {
                stream = TcpStream::connect(addr).await;
            }
            let stream = stream.unwrap();

            assert_eq!(stream.peer_addr().unwrap(), addr);
            assert_eq!(
                stream.local_addr().unwrap().ip(),
                std::net::Ipv4Addr::new(127, 0, 0, 1)
            );
            stream.set_ttl(101).unwrap();
            assert_eq!(stream.ttl().unwrap(), 101);
            stream.set_nodelay(true).unwrap();
            assert!(stream.nodelay().unwrap());
            assert!(stream.linger().unwrap().is_none());
            stream
                .set_linger(Some(std::time::Duration::from_secs(1)))
                .unwrap();
            assert_eq!(
                stream.linger().unwrap(),
                Some(std::time::Duration::from_secs(1))
            );
            assert!(stream.take_error().unwrap().is_none());
        });

        listener.accept().await.unwrap();

        handle.await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();
}

/// SDV test cases for `TcpStream`.
///
/// # Brief
/// 1. Bind `TcpListener` and wait for `accept()`.
/// 2. `TcpStream` connect to listener.
/// 3. Call peek() to get.
/// 4. Check result is correct.
#[test]
fn sdv_tcp_stream_peek() {
    let handle = ylong_runtime::spawn(async {
        let listener = TcpListener::bind(ADDR).await.unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = ylong_runtime::spawn(async move {
            let mut stream = TcpStream::connect(addr).await;
            while stream.is_err() {
                stream = TcpStream::connect(addr).await;
            }
            let stream = stream.unwrap();

            let mut buf = [0; 100];
            let len = stream.peek(&mut buf).await.expect("peek failed!");
            let buf = &buf[0..len];
            assert_eq!(len, 5);
            assert_eq!(String::from_utf8_lossy(buf), "hello");
        });

        let (mut stream, _) = listener.accept().await.unwrap();
        stream.write(b"hello").await.unwrap();

        handle.await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();
}

#[test]
fn sdv_tcp_global_runtime() {
    let handle = ylong_runtime::spawn(async move {
        let listener = TcpListener::bind(ADDR).await.expect("Bind Listener Failed");
        let addr = listener.local_addr().unwrap();

        // Start a thread as client side
        let handle = ylong_runtime::spawn(async move {
            let mut client = TcpStream::connect(addr).await;
            while client.is_err() {
                client = TcpStream::connect(addr).await;
            }
            let mut client = client.unwrap();

            let n = client
                .write(b"hello server")
                .await
                .expect("client send failed");
            assert_eq!(n, "hello server".len());

            let mut recv_buf = [0_u8; 12];
            let n = client
                .read(&mut recv_buf)
                .await
                .expect("client recv failed");
            assert_eq!(
                std::str::from_utf8(&recv_buf).unwrap(),
                "hello client".to_string()
            );
            assert_eq!(n, "hello client".len());
        });

        let (mut socket, _) = listener.accept().await.expect("Bind accept Failed");
        loop {
            let mut buf = [0_u8; 12];
            match socket.read(&mut buf).await.expect("recv Failed") {
                0 => break,
                n => {
                    assert_eq!(
                        std::str::from_utf8(&buf).unwrap(),
                        "hello server".to_string()
                    );
                    assert_eq!(n, "hello server".len());
                }
            };

            socket
                .write(b"hello client")
                .await
                .expect("failed to write to socket");
        }
        handle.await.unwrap();
    });
    ylong_runtime::block_on(handle).expect("block_on failed");
}

#[cfg(feature = "multi_instance_runtime")]
#[test]
fn sdv_tcp_multi_runtime() {
    use ylong_runtime::builder::RuntimeBuilder;
    let runtime = RuntimeBuilder::new_multi_thread().build().unwrap();

    runtime.block_on(async {
        let tcp = TcpListener::bind(ADDR).await.unwrap();
        let addr = tcp.local_addr().unwrap();

        let client = runtime.spawn(async move {
            let mut tcp = TcpStream::connect(addr).await;
            while tcp.is_err() {
                tcp = TcpStream::connect(addr).await;
            }
            let mut tcp = tcp.unwrap();
            let buf = [3; 100];
            tcp.write_all(&buf).await.unwrap();

            let mut buf = [0; 100];
            tcp.read_exact(&mut buf).await.unwrap();
            assert_eq!(buf, [2; 100]);
        });

        let (mut stream, _) = tcp.accept().await.unwrap();
        let mut buf = [0; 100];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [3; 100]);

        let buf = [2; 100];
        stream.write_all(&buf).await.unwrap();

        client.await.unwrap();
    });
}

/// SDV test cases for `TcpStream` of split().
///
/// # Brief
/// 1. Bind `TcpListener` and wait for `accept()`.
/// 2. `TcpStream` connect to listener.
/// 3. Split TcpStream into read half and write half with borrowed.
/// 4. Write with write half and read with read half.
/// 5. Check result is correct.
#[test]
fn sdv_tcp_split_borrow_half() {
    let handle = ylong_runtime::spawn(async {
        let listener = TcpListener::bind(ADDR).await.unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = ylong_runtime::spawn(async move {
            let mut stream = TcpStream::connect(addr).await;
            while stream.is_err() {
                stream = TcpStream::connect(addr).await;
            }
            let mut stream = stream.unwrap();

            let (mut read_half, mut write_half) = stream.split();
            write_half.write(b"I am write half.").await.unwrap();
            write_half.flush().await.unwrap();
            write_half.shutdown().await.unwrap();

            let mut buf = [0; 6];
            let n = read_half.read(&mut buf).await.expect("server read err");
            assert_eq!(n, 6);
            assert_eq!(buf, [1, 2, 3, 4, 5, 6]);
        });

        let (mut stream, _) = listener.accept().await.unwrap();
        let (mut read_half, mut write_half) = stream.split();
        let mut buf = [0; 16];
        let n = read_half.read(&mut buf).await.expect("server read err");
        assert_eq!(n, 16);
        assert_eq!(
            String::from_utf8(Vec::from(buf)).unwrap().as_str(),
            "I am write half."
        );

        let data1 = [1, 2, 3];
        let data2 = [4, 5, 6];
        let slice1 = std::io::IoSlice::new(&data1);
        let slice2 = std::io::IoSlice::new(&data2);
        write_half.write_vectored(&[slice1, slice2]).await.unwrap();

        handle.await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();
}

/// SDV test cases for `TcpStream` of into_split().
///
/// # Brief
/// 1. Bind `TcpListener` and wait for `accept()`.
/// 2. `TcpStream` connect to listener.
/// 3. Split TcpStream into read half and write half with owned.
/// 4. Write with write half and read with read half.
/// 5. Check result is correct.
#[test]
fn sdv_tcp_split_owned_half() {
    let handle = ylong_runtime::spawn(async move {
        let listener = TcpListener::bind(ADDR).await.unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = ylong_runtime::spawn(async move {
            let mut stream = TcpStream::connect(addr).await;
            while stream.is_err() {
                stream = TcpStream::connect(addr).await;
            }
            let stream = stream.unwrap();
            let (mut read_half, mut write_half) = stream.into_split();

            write_half.write(b"I am write half.").await.unwrap();
            write_half.flush().await.unwrap();
            write_half.shutdown().await.unwrap();

            let mut buf = [0; 6];
            let n = read_half.read(&mut buf).await.expect("server read err");
            assert_eq!(n, 6);
            assert_eq!(buf, [1, 2, 3, 4, 5, 6]);
        });

        let (mut stream, _) = listener.accept().await.unwrap();
        let (mut read_half, mut write_half) = stream.split();

        let mut buf = [0; 16];
        let n = read_half.read(&mut buf).await.expect("server read err");
        assert_eq!(n, 16);
        assert_eq!(
            String::from_utf8(Vec::from(buf)).unwrap().as_str(),
            "I am write half."
        );
        let data1 = [1, 2, 3];
        let data2 = [4, 5, 6];
        let slice1 = std::io::IoSlice::new(&data1);
        let slice2 = std::io::IoSlice::new(&data2);
        write_half.write_vectored(&[slice1, slice2]).await.unwrap();

        handle.await.unwrap();
    });
    ylong_runtime::block_on(handle).unwrap();
}

/// SDV case for dropping TcpStream outside of worker context
///
/// # Breif
/// 1. Starts 2 tasks via `spawn` that simulates a connection between client and
///    server
/// 2. Returns the streams to the main thread which is outside of the worker
///    context
/// 3. Drops the streams and it should not cause Panic
#[test]
#[cfg(not(feature = "ffrt"))]
fn sdv_tcp_drop_out_context() {
    let (tx, rx) = ylong_runtime::sync::oneshot::channel();
    let handle = ylong_runtime::spawn(async move {
        let tcp = TcpListener::bind(ADDR).await.unwrap();
        let addr = tcp.local_addr().unwrap();
        tx.send(addr).unwrap();
        let (mut stream, _) = tcp.accept().await.unwrap();
        let mut buf = [0; 10];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [3; 10]);

        let buf = [2; 10];
        stream.write_all(&buf).await.unwrap();
        stream
    });

    let client = ylong_runtime::block_on(async move {
        let addr = rx.await.unwrap();
        let mut tcp = TcpStream::connect(addr).await;
        while tcp.is_err() {
            tcp = TcpStream::connect(addr).await;
        }
        let mut tcp = tcp.unwrap();
        let buf = [3; 10];
        tcp.write_all(&buf).await.unwrap();

        let mut buf = [0; 10];
        tcp.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [2; 10]);
        tcp
    });

    let server = ylong_runtime::block_on(handle).unwrap();

    drop(server);
    drop(client);
}

/// SDV case for canceling TcpStream and then reconnecting on the same port
///
/// # Breif
/// 1. Starts a TCP connection using port 8201
/// 2. Cancels the TCP connection before its finished
/// 3. Starts another TCP connection using the same port 8201
/// 4. checks if the connection is successful.
#[cfg(feature = "time")]
#[test]
fn sdv_tcp_cancel() {
    use std::time::Duration;

    use ylong_runtime::time::sleep;

    let (tx, rx) = ylong_runtime::sync::oneshot::channel();
    let server = ylong_runtime::spawn(async {
        let tcp = TcpListener::bind(ADDR).await.unwrap();
        let addr = tcp.local_addr().unwrap();
        tx.send(addr).unwrap();
        let (mut stream, _) = tcp.accept().await.unwrap();
        sleep(Duration::from_secs(10000)).await;

        let mut buf = [0; 100];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [3; 100]);

        let buf = [2; 100];
        stream.write_all(&buf).await.unwrap();
    });

    let client = ylong_runtime::spawn(async {
        let addr = rx.await.unwrap();
        let mut tcp = TcpStream::connect(addr).await;
        while tcp.is_err() {
            tcp = TcpStream::connect(addr).await;
        }
        sleep(Duration::from_secs(10000)).await;
        let mut tcp = tcp.unwrap();
        let buf = [3; 100];
        tcp.write_all(&buf).await.unwrap();

        let mut buf = [0; 100];
        tcp.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [2; 100]);
    });

    server.cancel();
    client.cancel();
    let ret = ylong_runtime::block_on(server);
    assert!(ret.is_err());
    let ret = ylong_runtime::block_on(client);
    assert!(ret.is_err());

    let (tx, rx) = ylong_runtime::sync::oneshot::channel();
    let server = ylong_runtime::spawn(async move {
        let tcp = TcpListener::bind(ADDR).await.unwrap();
        let addr = tcp.local_addr().unwrap();
        tx.send(addr).unwrap();
        let (mut stream, _) = tcp.accept().await.unwrap();

        let mut buf = [0; 100];
        stream.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [3; 100]);

        let buf = [2; 100];
        stream.write_all(&buf).await.unwrap();
    });

    let client = ylong_runtime::spawn(async move {
        let addr = rx.await.unwrap();
        let mut tcp = TcpStream::connect(addr).await;
        while tcp.is_err() {
            tcp = TcpStream::connect(addr).await;
        }
        let mut tcp = tcp.unwrap();
        let buf = [3; 100];
        tcp.write_all(&buf).await.unwrap();

        let mut buf = [0; 100];
        tcp.read_exact(&mut buf).await.unwrap();
        assert_eq!(buf, [2; 100]);
    });

    ylong_runtime::block_on(server).unwrap();
    ylong_runtime::block_on(client).unwrap();
}
