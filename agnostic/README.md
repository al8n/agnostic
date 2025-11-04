<div align="center">

<!-- <img src="https://raw.githubusercontent.com/al8n/agnostic/main/art/logo.png" height = "200px"> -->

<h1>Agnostic</h1>

</div>
<div align="center">

`agnostic` is an agnostic abstraction layer for any async runtime.

If you want a light weight crate, see `agnostic-lite`.

[<img alt="github" src="https://img.shields.io/badge/github-al8n/agnostic-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Fagnostic" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/agnostic/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/agnostic?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-agnostic-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/agnostic?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/agnostic?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

</div>

## Introduction

`agnostic` is a comprehensive, runtime-agnostic abstraction layer for async Rust. It provides a unified API for task spawning, networking, DNS resolution, process management, and QUIC protocol support - all working seamlessly with tokio, async-std, or smol.

**Looking for a lightweight option?** Check out [`agnostic-lite`](../agnostic-lite/) for a minimal, `no_std`-compatible core.

## Features

- **Task Management**: Spawn tasks globally or locally
- **Time Operations**: Sleep, intervals, timeouts, and delays
- **Networking**: TCP listeners/streams and UDP sockets
- **DNS Resolution**: Multiple transports (DoH, DoT, DoQ, DoH3) with DNSSEC
- **Process Management**: Spawn and manage subprocesses
- **QUIC Support**: Quinn protocol integration
- **Runtime Agnostic**: Switch runtimes with a single feature flag
- **Zero-Cost**: Compiles to runtime-specific code

## Installation

```toml
[dependencies]
agnostic = "0.7"
```

### Runtime Selection

Choose one runtime feature:

```toml
# For tokio
agnostic = { version = "0.7", features = ["tokio"] }

# For async-std
agnostic = { version = "0.7", features = ["async-std"] }

# For smol
agnostic = { version = "0.7", features = ["smol"] }
```

### Optional Features

```toml
# Enable networking
agnostic = { version = "0.7", features = ["tokio", "net"] }

# Enable DNS resolution
agnostic = { version = "0.7", features = ["tokio", "dns"] }

# Enable DNS over HTTPS with rustls
agnostic = { version = "0.7", features = ["tokio", "dns-over-https-rustls"] }

# Enable DNS over QUIC
agnostic = { version = "0.7", features = ["tokio", "dns-over-quic"] }

# Enable DNSSEC
agnostic = { version = "0.7", features = ["tokio", "dnssec-ring"] }

# Enable process management
agnostic = { version = "0.7", features = ["tokio", "process"] }

# Enable Quinn QUIC
agnostic = { version = "0.7", features = ["tokio", "quinn"] }
```

## Quick Start

### Task Spawning

```rust
use agnostic::Runtime;

#[tokio::main]
async fn main() {
    // Spawn a task
    let handle = Runtime::spawn(async {
        println!("Hello from a spawned task!");
        42
    });

    let result = handle.await.unwrap();
    println!("Task returned: {}", result);
}
```

### Sleep and Timeouts

```rust
use agnostic::time::{timeout, Duration};
use agnostic::Runtime;

#[tokio::main]
async fn main() {
    // Sleep for 1 second
    Runtime::sleep(Duration::from_secs(1)).await;

    // Timeout an operation
    let result = timeout(Duration::from_secs(2), async {
        Runtime::sleep(Duration::from_secs(1)).await;
        "completed"
    }).await;

    match result {
        Ok(val) => println!("Operation completed: {}", val),
        Err(_) => println!("Operation timed out"),
    }
}
```

### TCP Server

```rust
use agnostic::net::TcpListener;
use agnostic_io::AsyncReadExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on port 8080");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        agnostic::Runtime::spawn(async move {
            let mut buf = vec![0u8; 1024];
            match stream.read(&mut buf).await {
                Ok(n) => {
                    println!("Received {} bytes", n);
                }
                Err(e) => eprintln!("Error reading: {}", e),
            }
        });
    }
}
```

### TCP Client

```rust
use agnostic::net::TcpStream;
use agnostic_io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    // Write data
    stream.write_all(b"Hello, server!").await?;
    stream.flush().await?;

    // Read response
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
```

### UDP Socket

```rust
use agnostic::net::UdpSocket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;

    let mut buf = [0u8; 1024];
    let (len, addr) = socket.recv_from(&mut buf).await?;

    println!("Received {} bytes from {}", len, addr);

    // Echo back
    socket.send_to(&buf[..len], addr).await?;

    Ok(())
}
```

### DNS Resolution

```rust
use agnostic::dns::{Dns, ResolverConfig, ResolverOpts};
use agnostic::net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DNS resolver
    let dns = Dns::<Net>::new(
        ResolverConfig::default(),
        ResolverOpts::default()
    )?;

    // Lookup A records
    let response = dns.lookup_ip("example.com").await?;

    for ip in response.iter() {
        println!("IP: {}", ip);
    }

    Ok(())
}
```

### Process Spawning

```rust
use agnostic::process::Command;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let output = Command::new("echo")
        .arg("Hello from subprocess!")
        .output()
        .await?;

    println!("Output: {}", String::from_utf8_lossy(&output.stdout));
    println!("Exit status: {}", output.status);

    Ok(())
}
```

### Intervals

```rust
use agnostic::time::{interval, Duration};
use agnostic::Runtime;

#[tokio::main]
async fn main() {
    let mut ticker = interval(Duration::from_secs(1));

    for i in 0..5 {
        ticker.tick().await;
        println!("Tick {}", i);
    }
}
```

## Switching Runtimes

One of the key benefits of `agnostic` is the ability to switch runtimes without changing your code. Simply change the feature flag and runtime initialization:

```rust
// Your application code remains the same
use agnostic::Runtime;

async fn my_app() {
    Runtime::spawn(async {
        println!("This works with any runtime!");
    }).await.unwrap();
}

// Just change the main function:

// With tokio:
#[tokio::main]
async fn main() {
    my_app().await;
}

// With async-std:
#[async_std::main]
async fn main() {
    my_app().await;
}

// With smol:
fn main() {
    smol::block_on(my_app());
}
```

## Feature Flags

### Core Features

- `std` (default): Standard library support
- `alloc`: Allocation support

### Runtime Features (choose one)

- `tokio`: Tokio runtime support
- `async-std`: Async-std runtime support
- `smol`: Smol runtime support

### Component Features

- `net`: Network abstractions (TCP, UDP)
- `dns`: DNS resolution support
- `process`: Process spawning and management
- `quinn`: Quinn QUIC protocol support
- `tokio-io`: Tokio I/O trait compatibility

### DNS Transport Features

- `dns-over-quic`: DNS over QUIC (RFC 9250)
- `dns-over-h3`: DNS over HTTP/3
- `dns-over-https-rustls`: DNS over HTTPS with rustls
- `dns-over-rustls`: DNS over TLS with rustls
- `dns-over-openssl`: DNS over TLS with OpenSSL
- `dns-over-native-tls`: DNS over TLS with native-tls
- `dns-webpki-roots`: Use webpki root certificates
- `dns-native-certs`: Use OS native certificates

### DNSSEC Features

- `dnssec`: Basic DNSSEC support
- `dnssec-openssl`: DNSSEC with OpenSSL
- `dnssec-ring`: DNSSEC with ring crypto library

## Comparison: agnostic vs agnostic-lite

| Feature | agnostic | agnostic-lite |
|---------|----------|---------------|
| Task spawning | ✅ | ✅ |
| Time operations | ✅ | ✅ |
| Networking | ✅ | ❌ |
| DNS resolution | ✅ | ❌ |
| Process management | ✅ | ❌ |
| QUIC support | ✅ | ❌ |
| `no_std` support | ❌ | ✅ |
| Alloc-free | ❌ | ✅ |
| No unsafe code | ❌ | ✅ |

Use `agnostic-lite` when:
- You need `no_std` or embedded support
- You want minimal dependencies
- You only need basic async primitives

Use `agnostic` when:
- You need networking, DNS, or process capabilities
- You're building standard applications
- You want a batteries-included experience

#### License

`agnostic` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2025 Al Liu.

[Github-url]: https://github.com/al8n/agnostic/
[CI-url]: https://github.com/al8n/agnostic/actions/workflows/ci.yml
[doc-url]: https://docs.rs/agnostic
[crates-url]: https://crates.io/crates/agnostic
[codecov-url]: https://app.codecov.io/gh/al8n/agnostic/

