<div align="center">

<!-- <img src="https://raw.githubusercontent.com/al8n/agnostic/main/art/logo.png" height = "200px"> -->

<h1>Agnostic</h1>

</div>
<div align="center">

[<img alt="github" src="https://img.shields.io/badge/github-al8n/agnostic-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Fagnostic-total" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/agnostic/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/agnostic?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

Write once, run on any async runtime.

</div>

## Overview

**Agnostic** is a Rust ecosystem that provides runtime-agnostic abstractions for async operations. Write your async code once and run it seamlessly with tokio, smol, or even in WebAssembly - without code changes.

### Why Agnostic?

- **Runtime Independence**: Switch between tokio, smol with a single feature flag
- **Zero-Cost Abstractions**: Generic implementations compile to specialized code per runtime
- **Incremental Adoption**: Use lightweight `agnostic-lite` or the full-featured `agnostic` crate
- **no_std Support**: Core abstractions work in embedded and constrained environments
- **Memory Safe**: No unsafe code in core abstractions (`agnostic-lite`)
- **Comprehensive**: Covers spawning, networking, DNS, process management, and more

## Architecture

The Agnostic ecosystem is organized into modular crates:

```
agnostic (facade - full-featured)
├── agnostic-lite (core - no_std, alloc-free)
│   ├── Task spawning (AsyncSpawner, AsyncLocalSpawner)
│   ├── Time abstractions (AsyncSleep, AsyncInterval, AsyncTimeout)
│   └── Runtime implementations (tokio, smol, wasm)
├── agnostic-io (Sans-I/O trait definitions)
│   └── AsyncRead, AsyncWrite, etc.
├── agnostic-net (networking)
│   ├── TCP (TcpListener, TcpStream)
│   ├── UDP (UdpSocket)
│   └── Depends on: agnostic-lite, agnostic-io
├── agnostic-dns (DNS resolution)
│   ├── Multiple transports (DoH, DoT, DoQ, DoH3)
│   ├── DNSSEC support
│   └── Depends on: agnostic-net, hickory-dns
└── agnostic-process (subprocess management)
    ├── Command execution
    ├── Stdio redirection
    └── Depends on: agnostic-io
```

## Crates

| Crate | Version | Description | Use When |
|-------|---------|-------------|----------|
| [`agnostic`](./agnostic/) | 0.7.2 | Full-featured facade with networking, DNS, process management, QUIC | You need comprehensive async abstractions |
| [`agnostic-lite`](./agnostic-lite/) | 0.5.6 | Lightweight core (no_std, alloc-free, no unsafe code) | You need minimal abstractions or embedded/no_std support |
| [`agnostic-io`](./agnostic-io/) | 0.1.2 | Sans-I/O trait definitions | You're building protocol implementations |
| [`agnostic-net`](./agnostic-net/) | 0.2.3 | TCP/UDP networking abstractions | You need runtime-agnostic networking |
| [`agnostic-dns`](./agnostic-dns/) | 0.2.2 | DNS resolution with DoH, DoT, DoQ, DNSSEC | You need advanced DNS capabilities |
| [`agnostic-process`](./agnostic-process/) | 0.2.2 | Subprocess spawning and management | You need to spawn external processes |

## Decision Guide

### Which Crate Should I Use?

**Use `agnostic` if:**

- You need full-featured async abstractions (networking, DNS, processes)
- You're building applications with multiple async capabilities
- You want a batteries-included experience

**Use `agnostic-lite` if:**

- You only need task spawning and time abstractions
- You're working in `no_std` or embedded environments
- You want to minimize dependencies
- You're building a library and want minimal footprint

**Use individual crates (`agnostic-net`, `agnostic-dns`, etc.) if:**

- You need specific functionality (e.g., just networking)
- You want fine-grained dependency control
- You're building specialized abstractions

### Runtime Selection

Choose your runtime based on your needs:

- **tokio**: Mature, widely used, excellent ecosystem
- **smol**: Lightweight, minimal dependencies
- **wasm-bindgen-futures**: WebAssembly support

All runtimes work identically with Agnostic abstractions.

## Supported Runtimes

- **tokio** - Enable with `features = ["tokio"]`
- **smol** - Enable with `features = ["smol"]`
- **wasm-bindgen-futures** - Enable with `features = ["wasm"]` (agnostic-lite only)

## Platform Support

- **Unix/Linux**: Full support via `rustix`
- **Windows**: Full support via `windows-sys`
- **WebAssembly**: Supported via `wasm-bindgen-futures` (agnostic-lite)
- **Embedded**: Supported via `no_std` (agnostic-lite)

#### License

`agnostic` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2024 Al Liu.

[Github-url]: https://github.com/al8n/agnostic/
[CI-url]: https://github.com/al8n/agnostic/actions/workflows/ci.yml
[doc-url]: https://docs.rs/agnostic
[agnostic-lite-doc-url]: https://docs.rs/agnostic-lite
[crates-url]: https://crates.io/crates/agnostic
[agnostic-lite-crates-url]: https://crates.io/crates/agnostic-lite
[codecov-url]: https://app.codecov.io/gh/al8n/agnostic/
