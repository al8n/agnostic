<div align="center">

<!-- <img src="https://raw.githubusercontent.com/al8n/agnostic/main/art/logo.png" height = "200px"> -->

<h1>Agnostic DNS</h1>

</div>
<div align="center">

`agnostic-dns` is an agnostic abstraction layer over [`hickory-dns`](https://github.com/hickory-dns/hickory-dns).

[<img alt="github" src="https://img.shields.io/badge/github-al8n/agnostic--net-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Fagnostic-dns" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/agnostic/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/agnostic?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-agnostic--net-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/agnostic-dns?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/agnostic-dns?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

</div>

## Introduction

`agnostic-dns` provides runtime-agnostic DNS resolution built on [`hickory-dns`](https://github.com/hickory-dns/hickory-dns). It supports multiple transport protocols (UDP, TCP, DoT, DoH, DoQ, DoH3) and DNSSEC validation - all working seamlessly across tokio, async-std, and smol runtimes.

### Key Features

- **Multiple Transport Protocols**:
  - DNS over UDP/TCP (standard)
  - DNS over TLS (DoT)
  - DNS over HTTPS (DoH)
  - DNS over QUIC (DoQ)
  - DNS over HTTP/3 (DoH3)
- **DNSSEC Support**: Validate DNS responses with OpenSSL or ring
- **Runtime Agnostic**: Works with tokio, async-std, and smol
- **Flexible Configuration**: Use system settings or custom resolvers
- **Comprehensive**: Built on the mature hickory-dns library

### Supported Runtimes

- **tokio** - Enable with `features = ["tokio"]`
- **async-std** - Enable with `features = ["async-std"]`
- **smol** - Enable with `features = ["smol"]`

## Installation

```toml
[dependencies]
agnostic-dns = "0.2"
```

- `tokio`

  ```toml
  agnostic-dns = { version = "0.2", features = ["tokio"] }
  ```

- `smol`

  ```toml
  agnostic-dns = { version = "0.2", features = ["smol"] }
  ```

- `async-std`

  ```toml
  agnostic-dns = { version = "0.2", features = ["async-std"] }
  ```

## Feature Matrix

| Feature | Description | Enable With |
|---------|-------------|-------------|
| **Core** | | |
| `dns` | Basic DNS resolution | Default |
| **Runtimes** | | |
| `tokio` | Tokio runtime support | `features = ["tokio"]` |
| `async-std` | Async-std support | `features = ["async-std"]` |
| `smol` | Smol runtime support | `features = ["smol"]` |
| **Transport Protocols** | | |
| `dns-over-rustls` | DNS over TLS with rustls | `features = ["dns-over-rustls"]` |
| `dns-over-openssl` | DNS over TLS with OpenSSL | `features = ["dns-over-openssl"]` |
| `dns-over-native-tls` | DNS over TLS with native-tls | `features = ["dns-over-native-tls"]` |
| `dns-over-https-rustls` | DNS over HTTPS with rustls | `features = ["dns-over-https-rustls"]` |
| `dns-over-quic` | DNS over QUIC (RFC 9250) | `features = ["dns-over-quic"]` |
| `dns-over-h3` | DNS over HTTP/3 | `features = ["dns-over-h3"]` |
| **Certificates** | | |
| `dns-webpki-roots` | Use webpki root certificates | `features = ["dns-webpki-roots"]` |
| `dns-native-certs` | Use OS native certificates | `features = ["dns-native-certs"]` |
| **DNSSEC** | | |
| `dnssec` | Basic DNSSEC validation | `features = ["dnssec"]` |
| `dnssec-openssl` | DNSSEC with OpenSSL crypto | `features = ["dnssec-openssl"]` |
| `dnssec-ring` | DNSSEC with ring crypto | `features = ["dnssec-ring"]` |
| **Other** | | |
| `tracing` | Distributed tracing support | `features = ["tracing"]` |

## Quick Start

### Basic DNS Resolution

```rust
use agnostic_dns::{Dns, ResolverConfig, ResolverOpts};
use agnostic_net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DNS resolver with default config
    let dns = Dns::<Net>::new(
        ResolverConfig::default(),
        ResolverOpts::default()
    )?;

    // Lookup IP addresses
    let response = dns.lookup_ip("example.com").await?;

    for ip in response.iter() {
        println!("IP: {}", ip);
    }

    Ok(())
}
```

### Using System Configuration

```rust
use agnostic_dns::{Dns, system_conf};
use agnostic_net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use system's DNS configuration
    let (config, opts) = system_conf::read_system_conf()?;
    let dns = Dns::<Net>::new(config, opts)?;

    let response = dns.lookup_ip("github.com").await?;

    for ip in response.iter() {
        println!("GitHub IP: {}", ip);
    }

    Ok(())
}
```

### Custom DNS Server

```rust
use agnostic_dns::{Dns, ResolverConfig, ResolverOpts, Name, NameServerConfig};
use agnostic_net::Net;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use Google DNS (8.8.8.8)
    let mut config = ResolverConfig::new();
    let nameserver = NameServerConfig {
        socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53),
        protocol: hickory_proto::rr::dnssec::Protocol::Udp,
        tls_dns_name: None,
        trust_nx_responses: false,
        bind_addr: None,
    };
    config.add_name_server(nameserver);

    let dns = Dns::<Net>::new(config, ResolverOpts::default())?;

    let response = dns.lookup_ip("cloudflare.com").await?;

    for ip in response.iter() {
        println!("IP: {}", ip);
    }

    Ok(())
}
```

## DNS over HTTPS (DoH)

Enable with `features = ["dns-over-https-rustls", "dns-webpki-roots"]`:

```rust
use agnostic_dns::{Dns, ResolverConfig, ResolverOpts, Name, NameServerConfig};
use agnostic_net::Net;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ResolverConfig::new();

    // Cloudflare DoH
    let nameserver = NameServerConfig {
        socket_addr: "1.1.1.1:443".parse()?,
        protocol: hickory_proto::rr::dnssec::Protocol::Https,
        tls_dns_name: Some("cloudflare-dns.com".to_string()),
        trust_nx_responses: true,
        bind_addr: None,
    };
    config.add_name_server(nameserver);

    let dns = Dns::<Net>::new(config, ResolverOpts::default())?;

    let response = dns.lookup_ip("example.com").await?;

    println!("DoH Resolution:");
    for ip in response.iter() {
        println!("  IP: {}", ip);
    }

    Ok(())
}
```

## DNS over TLS (DoT)

Enable with `features = ["dns-over-rustls", "dns-webpki-roots"]`:

```rust
use agnostic_dns::{Dns, ResolverConfig, ResolverOpts, NameServerConfig};
use agnostic_net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ResolverConfig::new();

    // Cloudflare DoT
    let nameserver = NameServerConfig {
        socket_addr: "1.1.1.1:853".parse()?,
        protocol: hickory_proto::rr::dnssec::Protocol::Tls,
        tls_dns_name: Some("cloudflare-dns.com".to_string()),
        trust_nx_responses: true,
        bind_addr: None,
    };
    config.add_name_server(nameserver);

    let dns = Dns::<Net>::new(config, ResolverOpts::default())?;

    let response = dns.lookup_ip("example.com").await?;

    println!("DoT Resolution:");
    for ip in response.iter() {
        println!("  IP: {}", ip);
    }

    Ok(())
}
```

## DNS over QUIC (DoQ)

Enable with `features = ["dns-over-quic"]`:

```rust
use agnostic_dns::{Dns, ResolverConfig, ResolverOpts, NameServerConfig};
use agnostic_net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = ResolverConfig::new();

    // AdGuard DoQ
    let nameserver = NameServerConfig {
        socket_addr: "94.140.14.14:853".parse()?,
        protocol: hickory_proto::rr::dnssec::Protocol::Quic,
        tls_dns_name: Some("dns.adguard.com".to_string()),
        trust_nx_responses: true,
        bind_addr: None,
    };
    config.add_name_server(nameserver);

    let dns = Dns::<Net>::new(config, ResolverOpts::default())?;

    let response = dns.lookup_ip("example.com").await?;

    println!("DoQ Resolution:");
    for ip in response.iter() {
        println!("  IP: {}", ip);
    }

    Ok(())
}
```

## DNSSEC Validation

Enable with `features = ["dnssec-ring"]`:

```rust
use agnostic_dns::{Dns, ResolverConfig, ResolverOpts};
use agnostic_net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut opts = ResolverOpts::default();
    opts.validate = true; // Enable DNSSEC validation

    let dns = Dns::<Net>::new(
        ResolverConfig::default(),
        opts
    )?;

    // Query a DNSSEC-signed domain
    let response = dns.lookup_ip("cloudflare.com").await?;

    println!("DNSSEC-validated IPs:");
    for ip in response.iter() {
        println!("  {}", ip);
    }

    Ok(())
}
```

## Advanced Usage

### Multiple Record Types

```rust
use agnostic_dns::{Dns, Name, RecordType};
use agnostic_net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dns = Dns::<Net>::system_conf()?;

    // A records
    let a_records = dns.lookup(
        Name::from_utf8("example.com")?,
        RecordType::A
    ).await?;

    // AAAA records (IPv6)
    let aaaa_records = dns.lookup(
        Name::from_utf8("example.com")?,
        RecordType::AAAA
    ).await?;

    // MX records (mail servers)
    let mx_records = dns.lookup(
        Name::from_utf8("example.com")?,
        RecordType::MX
    ).await?;

    // TXT records
    let txt_records = dns.lookup(
        Name::from_utf8("example.com")?,
        RecordType::TXT
    ).await?;

    Ok(())
}
```

### Reverse DNS Lookup

```rust
use agnostic_dns::Dns;
use agnostic_net::Net;
use std::net::IpAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dns = Dns::<Net>::system_conf()?;

    let ip: IpAddr = "8.8.8.8".parse()?;
    let response = dns.reverse_lookup(ip).await?;

    println!("Reverse DNS for {}:", ip);
    for name in response.iter() {
        println!("  {}", name);
    }

    Ok(())
}
```

### Caching

```rust
use agnostic_dns::{Dns, ResolverOpts};
use agnostic_net::Net;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut opts = ResolverOpts::default();
    opts.cache_size = 1024; // Cache 1024 entries
    opts.positive_min_ttl = Some(std::time::Duration::from_secs(60));
    opts.negative_min_ttl = Some(std::time::Duration::from_secs(10));

    let dns = Dns::<Net>::new(
        ResolverConfig::default(),
        opts
    )?;

    // First lookup (not cached)
    let start = std::time::Instant::now();
    dns.lookup_ip("example.com").await?;
    println!("First lookup: {:?}", start.elapsed());

    // Second lookup (cached)
    let start = std::time::Instant::now();
    dns.lookup_ip("example.com").await?;
    println!("Cached lookup: {:?}", start.elapsed());

    Ok(())
}
```

## Popular DNS Providers

### Cloudflare (1.1.1.1)

```toml
# For DoH
agnostic-dns = { version = "0.2", features = ["tokio", "dns-over-https-rustls", "dns-webpki-roots"] }
```

- UDP/TCP: `1.1.1.1:53`
- DoT: `1.1.1.1:853` (cloudflare-dns.com)
- DoH: `1.1.1.1:443` (cloudflare-dns.com/dns-query)

### Google Public DNS (8.8.8.8)

- UDP/TCP: `8.8.8.8:53` / `8.8.4.4:53`
- DoT: `8.8.8.8:853` (dns.google)
- DoH: `8.8.8.8:443` (dns.google/dns-query)

### Quad9 (9.9.9.9)

- UDP/TCP: `9.9.9.9:53`
- DoT: `9.9.9.9:853` (dns.quad9.net)
- DoH: `9.9.9.9:443` (dns.quad9.net/dns-query)

## Performance Tips

1. **Enable caching** for frequently queried domains
2. **Use DoH/DoT** for privacy, but note the overhead
3. **Configure timeouts** appropriately for your use case
4. **Reuse resolver instances** - they're designed for long-term use
5. **Use system configuration** when possible for optimal defaults

## Switching Runtimes

Your DNS code remains the same across runtimes:

```rust
use agnostic_dns::Dns;
use agnostic_net::Net;

async fn resolve_domain(domain: &str) -> Result<(), Box<dyn std::error::Error>> {
    let dns = Dns::<Net>::system_conf()?;
    let response = dns.lookup_ip(domain).await?;

    for ip in response.iter() {
        println!("{}", ip);
    }

    Ok(())
}

// Just change the runtime:

// Tokio:
#[tokio::main]
async fn main() {
    resolve_domain("example.com").await.unwrap();
}

// Async-std:
#[async_std::main]
async fn main() {
    resolve_domain("example.com").await.unwrap();
}

// Smol:
fn main() {
    smol::block_on(resolve_domain("example.com")).unwrap();
}
```

#### License

`agnostic-dns` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2025 Al Liu.

[Github-url]: https://github.com/al8n/agnostic/
[CI-url]: https://github.com/al8n/agnostic/actions/workflows/ci.yml
[doc-url]: https://docs.rs/agnostic-dns
[crates-url]: https://crates.io/crates/agnostic-dns
[codecov-url]: https://app.codecov.io/gh/al8n/agnostic/
