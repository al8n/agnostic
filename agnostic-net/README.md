<div align="center">

<!-- <img src="https://raw.githubusercontent.com/al8n/agnostic/main/art/logo.png" height = "200px"> -->

<h1>Agnostic Network</h1>

</div>
<div align="center">

Agnostic abstraction layer of `std::net` for any async runtime.

[<img alt="github" src="https://img.shields.io/badge/github-al8n/agnostic--net-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Fagnostic-net" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/agnostic/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/agnostic?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-agnostic--net-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/agnostic-net?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/agnostic-net?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

</div>

## Introduction

`agnostic-net` provides runtime-agnostic abstractions for TCP and UDP networking. Write your network code once and run it with tokio, async-std, or smol - without any code changes.

### Key Features

- **TCP Support**: Async TCP listeners and streams
- **UDP Support**: Async UDP sockets
- **Cross-Platform**: Works on Unix, Windows, and other platforms
- **Runtime Agnostic**: Seamlessly switch between tokio, async-std, and smol
- **Zero-Cost**: Compiles to runtime-specific code
- **Familiar API**: Similar to `std::net` but async

### Supported Runtimes

- **tokio** - Enable with `features = ["tokio"]`
- **async-std** - Enable with `features = ["async-std"]`
- **smol** - Enable with `features = ["smol"]`

## Installation

```toml
[dependencies]
agnostic-net = "0.2"
```

- `tokio`

  ```toml
  agnostic-net = { version = "0.2", features = ["tokio"] }
  ```

- `smol`

  ```toml
  agnostic-net = { version = "0.2", features = ["smol"] }
  ```

- `async-std`

  ```toml
  agnostic-net = { version = "0.2", features = ["async-std"] }
  ```

## Quick Start

### TCP Echo Server

```rust
use agnostic_net::TcpListener;
use agnostic_io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on port 8080");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];

            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => break, // Connection closed
                    Ok(n) => {
                        // Echo back
                        if stream.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }

            println!("Connection closed: {}", addr);
        });
    }
}
```

### TCP Client

```rust
use agnostic_net::TcpStream;
use agnostic_io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Connect to server
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    // Send data
    stream.write_all(b"Hello, server!").await?;
    stream.flush().await?;

    // Read response
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;

    println!("Received: {}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
```

### UDP Echo Server

```rust
use agnostic_net::UdpSocket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    println!("UDP server listening on port 8080");

    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        println!("Received {} bytes from {}", len, addr);

        // Echo back
        socket.send_to(&buf[..len], addr).await?;
    }
}
```

### UDP Client

```rust
use agnostic_net::UdpSocket;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    // Send data
    socket.send_to(b"Hello, UDP!", "127.0.0.1:8080").await?;

    // Receive response
    let mut buf = [0u8; 1024];
    let (len, addr) = socket.recv_from(&mut buf).await?;

    println!("Received {} bytes from {}: {}",
        len, addr, String::from_utf8_lossy(&buf[..len]));

    Ok(())
}
```

## Advanced Examples

### HTTP Server (Simplified)

```rust
use agnostic_net::TcpListener;
use agnostic_io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("HTTP server running on http://127.0.0.1:3000");

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];

            if let Ok(n) = stream.read(&mut buf).await {
                let request = String::from_utf8_lossy(&buf[..n]);
                println!("Request: {}", request);

                let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
    }
}
```

### TCP Connection Pool

```rust
use agnostic_net::TcpStream;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ConnectionPool {
    address: String,
    semaphore: Arc<Semaphore>,
}

impl ConnectionPool {
    pub fn new(address: String, max_connections: usize) -> Self {
        Self {
            address,
            semaphore: Arc::new(Semaphore::new(max_connections)),
        }
    }

    pub async fn get_connection(&self) -> std::io::Result<TcpStream> {
        let _permit = self.semaphore.acquire().await.unwrap();
        TcpStream::connect(&self.address).await
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let pool = Arc::new(ConnectionPool::new(
        "127.0.0.1:8080".to_string(),
        10,
    ));

    let mut handles = vec![];

    for i in 0..20 {
        let pool = Arc::clone(&pool);
        let handle = tokio::spawn(async move {
            let mut stream = pool.get_connection().await.unwrap();
            println!("Connection {} established", i);
            // Use the connection...
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
```

### Broadcast UDP Server

```rust
use agnostic_net::UdpSocket;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:8080").await?);
    let clients = Arc::new(Mutex::new(HashSet::<SocketAddr>::new()));

    println!("Broadcast server running on 127.0.0.1:8080");

    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let message = &buf[..len];

        // Add client
        clients.lock().await.insert(addr);

        println!("Broadcasting message from {}", addr);

        // Broadcast to all clients
        let clients_list: Vec<_> = clients.lock().await.iter().copied().collect();
        for client in clients_list {
            if client != addr {
                let _ = socket.send_to(message, client).await;
            }
        }
    }
}
```

### Chat Server

```rust
use agnostic_net::TcpListener;
use agnostic_io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    println!("Chat server running on 127.0.0.1:8080");

    loop {
        let (mut stream, addr) = listener.accept().await?;
        let tx = Arc::clone(&tx);
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            println!("New client: {}", addr);

            let (mut reader, mut writer) = stream.split();

            // Spawn task to send messages
            let addr_clone = addr;
            tokio::spawn(async move {
                while let Ok(msg) = rx.recv().await {
                    let _ = writer.write_all(msg.as_bytes()).await;
                    let _ = writer.write_all(b"\n").await;
                }
            });

            // Read and broadcast messages
            let mut buf = vec![0u8; 1024];
            loop {
                match reader.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let msg = format!(
                            "{}: {}",
                            addr_clone,
                            String::from_utf8_lossy(&buf[..n])
                        );
                        let _ = tx.send(msg);
                    }
                    Err(_) => break,
                }
            }

            println!("Client disconnected: {}", addr_clone);
        });
    }
}
```

## API Reference

### TcpListener

```rust
impl TcpListener {
    // Bind to an address
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener>;

    // Accept a new connection
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)>;

    // Get the local address
    pub fn local_addr(&self) -> io::Result<SocketAddr>;
}
```

### TcpStream

```rust
impl TcpStream {
    // Connect to an address
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream>;

    // Get local address
    pub fn local_addr(&self) -> io::Result<SocketAddr>;

    // Get peer address
    pub fn peer_addr(&self) -> io::Result<SocketAddr>;

    // Set TCP_NODELAY
    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()>;

    // Set TTL
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()>;

    // Shutdown read, write, or both
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()>;

    // Split into read and write halves
    pub fn split(&mut self) -> (ReadHalf, WriteHalf);
}
```

### UdpSocket

```rust
impl UdpSocket {
    // Bind to an address
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<UdpSocket>;

    // Send data to an address
    pub async fn send_to<A: ToSocketAddrs>(
        &self,
        buf: &[u8],
        target: A
    ) -> io::Result<usize>;

    // Receive data from any address
    pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)>;

    // Connect to a remote address
    pub async fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<()>;

    // Send data (requires connected socket)
    pub async fn send(&self, buf: &[u8]) -> io::Result<usize>;

    // Receive data (requires connected socket)
    pub async fn recv(&self, buf: &mut [u8]) -> io::Result<usize>;

    // Get local address
    pub fn local_addr(&self) -> io::Result<SocketAddr>;

    // Set broadcast
    pub fn set_broadcast(&self, broadcast: bool) -> io::Result<()>;

    // Set TTL
    pub fn set_ttl(&self, ttl: u32) -> io::Result<()>;
}
```

## Switching Runtimes

One of the key benefits is runtime portability. Just change the feature flag:

```rust
// Your code stays the same
use agnostic_net::TcpListener;

async fn start_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    // ... rest of your code
    Ok(())
}

// Only change the runtime initialization:

// With tokio:
#[tokio::main]
async fn main() {
    start_server().await.unwrap();
}

// With async-std:
#[async_std::main]
async fn main() {
    start_server().await.unwrap();
}

// With smol:
fn main() {
    smol::block_on(start_server()).unwrap();
}
```

## Platform Support

`agnostic-net` works on all major platforms:

- **Unix/Linux**: Full support via `rustix`
- **Windows**: Full support via `windows-sys` and `socket2`
- **macOS**: Full support
- **BSD**: Full support

## Feature Flags

- `std` (default): Standard library support
- `tokio`: Tokio runtime support
- `tokio-io`: Tokio I/O trait implementations
- `async-std`: Async-std runtime support
- `smol`: Smol runtime support

## Performance Considerations

- **Zero-cost abstractions**: Compiles to runtime-specific code
- **No runtime overhead**: Direct delegation to underlying runtime
- **Efficient I/O**: Uses platform-specific optimizations

#### License

`agnostic-net` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2025 Al Liu.

[Github-url]: https://github.com/al8n/agnostic/
[CI-url]: https://github.com/al8n/agnostic/actions/workflows/ci.yml
[doc-url]: https://docs.rs/agnostic-net
[crates-url]: https://crates.io/crates/agnostic-net
[codecov-url]: https://app.codecov.io/gh/al8n/agnostic/
