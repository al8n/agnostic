<div align="center">

<!-- <img src="https://raw.githubusercontent.com/al8n/agnostic/main/art/logo.png" height = "200px"> -->

<h1>Agnostic I/O</h1>

</div>
<div align="center">

`agnostic-io` defines I/O traits in [Sans-I/O] style for any async runtime.

[<img alt="github" src="https://img.shields.io/badge/github-al8n/agnostic--io-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Fagnostic-io" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/agnostic/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/agnostic?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-agnostic--io-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/agnostic-io?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/agnostic-io?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

</div>

## Introduction

`agnostic-io` provides runtime-agnostic I/O traits following the [Sans-I/O] design philosophy. It defines standardized async I/O interfaces that work across tokio, async-std, smol, and other runtimes without coupling your protocol implementations to specific I/O primitives.

### What is Sans-I/O?

[Sans-I/O] (French for "without I/O") is a design pattern that **separates protocol logic from I/O implementation**. Instead of tightly coupling your code to specific I/O libraries, you:

1. Define abstract I/O traits (what `agnostic-io` provides)
2. Implement your protocol logic against these traits
3. Let runtime-specific implementations handle the actual I/O

This approach makes your code:
- **Testable**: Mock I/O without real sockets
- **Portable**: Works with any async runtime
- **Reusable**: Protocol implementations work everywhere
- **Maintainable**: Changes to I/O layer don't affect protocol logic

### Key Features

- **Runtime Agnostic**: Works with tokio, async-std, smol, and more
- **`no_std` Compatible**: Can be used in embedded environments
- **Zero-Cost**: Trait-based design compiles away
- **Comprehensive**: AsyncRead, AsyncWrite, AsyncSeek, and more
- **Tokio Compatible**: Optional compatibility layer for tokio traits

## Installation

```toml
[dependencies]
agnostic-io = "0.1"
```

### Feature Flags

```toml
# Standard library support (default)
agnostic-io = { version = "0.1", features = ["std"] }

# Allocation support without std
agnostic-io = { version = "0.1", default-features = false, features = ["alloc"] }

# Tokio I/O trait compatibility
agnostic-io = { version = "0.1", features = ["tokio"] }

# no_std without allocations
agnostic-io = { version = "0.1", default-features = false }
```

## Core Traits

### AsyncRead

Async reading of byte streams:

```rust
pub trait AsyncRead {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;
}
```

### AsyncWrite

Async writing of byte streams:

```rust
pub trait AsyncWrite {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>>;

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>>;

    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>>;
}
```

### AsyncBufRead

Buffered async reading:

```rust
pub trait AsyncBufRead: AsyncRead {
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<&[u8]>>;

    fn consume(self: Pin<&mut Self>, amt: usize);
}
```

## Quick Start

### Using AsyncRead

```rust
use agnostic_io::AsyncReadExt;

async fn read_data<R: agnostic_io::AsyncRead + Unpin>(
    reader: &mut R
) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; 1024];
    let n = reader.read(&mut buffer).await?;
    buffer.truncate(n);
    Ok(buffer)
}

// Works with any runtime's TCP stream:
#[tokio::main]
async fn main() {
    let mut stream = tokio::net::TcpStream::connect("127.0.0.1:8080")
        .await
        .unwrap();

    let data = read_data(&mut stream).await.unwrap();
    println!("Read {} bytes", data.len());
}
```

### Using AsyncWrite

```rust
use agnostic_io::AsyncWriteExt;

async fn write_data<W: agnostic_io::AsyncWrite + Unpin>(
    writer: &mut W,
    data: &[u8],
) -> std::io::Result<()> {
    writer.write_all(data).await?;
    writer.flush().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut stream = tokio::net::TcpStream::connect("127.0.0.1:8080")
        .await
        .unwrap();

    write_data(&mut stream, b"Hello, world!").await.unwrap();
}
```

### Building a Protocol

Here's how to build a runtime-agnostic protocol:

```rust
use agnostic_io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};

// A simple framing protocol
pub struct FrameCodec<T> {
    inner: T,
}

impl<T> FrameCodec<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    // Read a length-prefixed frame
    pub async fn read_frame(&mut self) -> std::io::Result<Vec<u8>> {
        // Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        self.inner.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        // Read frame data
        let mut frame = vec![0u8; len];
        self.inner.read_exact(&mut frame).await?;

        Ok(frame)
    }

    // Write a length-prefixed frame
    pub async fn write_frame(&mut self, data: &[u8]) -> std::io::Result<()> {
        // Write length prefix
        let len = (data.len() as u32).to_be_bytes();
        self.inner.write_all(&len).await?;

        // Write frame data
        self.inner.write_all(data).await?;
        self.inner.flush().await?;

        Ok(())
    }
}

// Usage with any runtime:
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let stream = tokio::net::TcpStream::connect("127.0.0.1:8080").await?;
    let mut codec = FrameCodec::new(stream);

    // Send a frame
    codec.write_frame(b"Hello").await?;

    // Receive a frame
    let response = codec.read_frame().await?;
    println!("Received: {:?}", response);

    Ok(())
}
```

## Extension Traits

`agnostic-io` provides convenient extension traits:

### AsyncReadExt

```rust
use agnostic_io::AsyncReadExt;

async fn example<R: agnostic_io::AsyncRead + Unpin>(reader: &mut R) {
    // Read exact number of bytes
    let mut buf = [0u8; 10];
    reader.read_exact(&mut buf).await.unwrap();

    // Read to end
    let mut data = Vec::new();
    reader.read_to_end(&mut data).await.unwrap();

    // Read to string
    let mut text = String::new();
    reader.read_to_string(&mut text).await.unwrap();

    // Take (limit reading)
    let mut limited = reader.take(100);
    limited.read_to_end(&mut data).await.unwrap();

    // Chain readers
    let reader2 = std::io::Cursor::new(b"more data");
    let mut chained = reader.chain(reader2);
}
```

### AsyncWriteExt

```rust
use agnostic_io::AsyncWriteExt;

async fn example<W: agnostic_io::AsyncWrite + Unpin>(writer: &mut W) {
    // Write all bytes
    writer.write_all(b"Hello").await.unwrap();

    // Write formatted data
    writer.write_fmt(format_args!("Number: {}", 42)).await.unwrap();

    // Flush buffered data
    writer.flush().await.unwrap();

    // Close the writer
    writer.close().await.unwrap();
}
```

### AsyncBufReadExt

```rust
use agnostic_io::AsyncBufReadExt;

async fn example<R: agnostic_io::AsyncBufRead + Unpin>(reader: &mut R) {
    // Read lines
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        println!("Line: {}", line);
    }

    // Read until delimiter
    let mut buf = Vec::new();
    reader.read_until(b'\n', &mut buf).await.unwrap();

    // Split on delimiter
    let mut split = reader.split(b',');
    while let Some(chunk) = split.next_segment().await.unwrap() {
        println!("Chunk: {:?}", chunk);
    }
}
```

## Testing with Sans-I/O

The Sans-I/O approach makes testing trivial:

```rust
use agnostic_io::{AsyncRead, AsyncWrite};
use std::io::Cursor;

#[tokio::test]
async fn test_protocol() {
    // Mock I/O with Cursor (implements AsyncRead/AsyncWrite)
    let mock_data = b"test data";
    let mut reader = Cursor::new(mock_data);

    let mut output = Vec::new();
    let mut writer = Cursor::new(&mut output);

    // Test your protocol without real I/O
    my_protocol_handler(&mut reader, &mut writer).await.unwrap();

    assert_eq!(output, b"expected output");
}

async fn my_protocol_handler<R, W>(
    reader: &mut R,
    writer: &mut W,
) -> std::io::Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    // Your protocol logic here
    Ok(())
}
```

## Implementing Custom I/O Types

You can implement `agnostic-io` traits for your own types:

```rust
use agnostic_io::AsyncRead;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MyReader {
    data: Vec<u8>,
    pos: usize,
}

impl AsyncRead for MyReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let remaining = &self.data[self.pos..];
        let to_read = remaining.len().min(buf.len());

        buf[..to_read].copy_from_slice(&remaining[..to_read]);
        self.pos += to_read;

        Poll::Ready(Ok(to_read))
    }
}
```

## Tokio Compatibility

When using the `tokio` feature, `agnostic-io` provides compatibility with `tokio::io` traits:

```toml
[dependencies]
agnostic-io = { version = "0.1", features = ["tokio"] }
```

```rust
use agnostic_io::AsyncReadExt;
use tokio::io::AsyncRead as TokioAsyncRead;

async fn works_with_tokio<R: TokioAsyncRead + Unpin>(reader: &mut R) {
    // Can use agnostic-io extension methods with tokio types
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await.unwrap();
}
```

## Benefits of Sans-I/O

### 1. Runtime Independence

Your protocol works with any runtime:

```rust
// Same code works with tokio, async-std, or smol
async fn my_protocol<S>(stream: S) -> std::io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // Protocol implementation
    Ok(())
}
```

### 2. Easy Testing

No need for real sockets:

```rust
#[test]
fn test_without_io() {
    let mock = Cursor::new(vec![]);
    // Test your protocol
}
```

### 3. Composability

Build complex I/O from simple parts:

```rust
let reader = TcpStream::connect("...").await?;
let buffered = BufReader::new(reader);
let limited = buffered.take(1024);
let codec = FrameCodec::new(limited);
```

### 4. Clear Separation

Protocol logic is separate from I/O:

```rust
// Protocol layer - pure logic
fn parse_message(data: &[u8]) -> Result<Message, Error> { ... }

// I/O layer - runtime specific
async fn read_message<R: AsyncRead>(reader: &mut R) -> Result<Message, Error> {
    let data = read_frame(reader).await?;
    parse_message(&data)
}
```

## Feature Flags

- `std` (default): Standard library support
- `alloc`: Allocation support without std
- `tokio`: Compatibility with tokio::io traits

#### License

`agnostic-io` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2025 Al Liu.

[Github-url]: https://github.com/al8n/agnostic/
[CI-url]: https://github.com/al8n/agnostic/actions/workflows/ci.yml
[doc-url]: https://docs.rs/agnostic-io
[crates-url]: https://crates.io/crates/agnostic-io
[codecov-url]: https://app.codecov.io/gh/al8n/agnostic/
[Sans-I/O]: https://sans-io.readthedocs.io/en/latest/
