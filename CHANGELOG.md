# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### agnostic [0.8.0] - 2025-11-04

#### Added
- Quick start examples for: task spawning, sleep/timeouts, TCP server/client, UDP, DNS resolution, process spawning, intervals
- Complete feature flags reference with descriptions for all 20+ features
- Runtime switching guide with examples for tokio, async-std, and smol
- Comparison table: agnostic vs agnostic-lite showing capabilities and use cases
- Feature documentation for DNS transports (DoH, DoT, DoQ, DoH3) and DNSSEC
- Installation instructions for all feature combinations
- "Why Agnostic?" section explaining benefits and use cases

#### Changed
- Improved introduction with clearer value proposition
- Enhanced feature categorization (Core, Runtime, Component, DNS Transport, DNSSEC)

### agnostic-lite [0.6.0] - 2025-11-04

#### Added
- WASM/WebAssembly usage example with wasm-bindgen
- Embedded systems example for no_std environments
- Runtime-agnostic library example showing generic code patterns
- Complete trait signatures for all 6 core traits (AsyncSpawner, AsyncLocalSpawner, AsyncSleep, AsyncInterval, AsyncTimeout, RuntimeLite)
- Conditional compilation helpers documentation (cfg_tokio!, cfg_smol!, etc.)
- Performance section explaining zero-cost abstractions
- Examples for: spawning tasks, sleep, intervals, timeouts, local tasks, generic code
- "Why agnostic-lite?" decision guide comparing with agnostic

#### Changed
- Clarified the distinction between agnostic-lite and agnostic
- Improved feature flags documentation
- Enhanced embedded and no_std documentation

### agnostic-io [0.2.0] - 2025-11-04

#### Added
- Comprehensive Sans-I/O philosophy explanation with detailed benefits
- "What is Sans-I/O?" section explaining the design pattern
- Complete protocol implementation example (FrameCodec with length-prefixed framing)
- Extension traits documentation (AsyncReadExt, AsyncWriteExt, AsyncBufReadExt)
- Custom I/O type implementation example
- Examples for: reading data, writing data, building protocols, testing, composability
- Tokio compatibility layer documentation

#### Changed
- Reorganized documentation to emphasize Sans-I/O benefits
- Enhanced feature flags explanation
- Improved API reference with trait signatures

### agnostic-net [0.3.0] - 2025-11-04

#### Added

- Complete API reference for TcpListener, TcpStream, and UdpSocket
- Platform support details (Unix/Linux, Windows, macOS, BSD)
- Performance considerations section

#### Changed

- Improved introduction with clearer feature list
- Enhanced key features section

### agnostic-dns [0.3.0] - 2025-11-04

#### Added

- Feature matrix table documenting all 18+ features
- Performance tips for production use

#### Changed
- Reorganized features into clear categories (Core, Runtimes, Transport Protocols, Certificates, DNSSEC, Other)
- Improved introduction with clearer feature list

#### Fixed
- Corrected version numbers in installation instructions

### agnostic-process [0.3.0] - 2025-11-04

#### Added

- API reference for Command, Child, and Process trait
- Runtime switching guide

#### Changed

- Improved introduction with clearer feature list

#### Fixed

- Corrected version numbers in installation instructions

### Project-wide Changes

#### Added

- Main project README with:
  - Architecture diagram showing crate relationships
  - Decision guide: "Which Crate Should I Use?"
  - Crates comparison table with versions and use cases
  - Runtime selection guide
  - Platform support matrix
  - "Why Agnostic?" value proposition
- CHANGELOG.md with complete version history
- Migration guide for async-std removal

#### Changed

- Consistent README structure across all crates
- Comprehensive documentation coverage (7 READMEs enhanced)

#### Fixed

- Version numbers corrected across all crate READMEs

## [0.7.2] - 2025-01-04

### Changed
- Updated dependencies to latest versions
- Improved documentation across all crates

## [0.7.0] - 2025-01-04

### Removed
- **BREAKING**: Removed async-std runtime support across all crates due to async-std being discontinued
  - Removed from `agnostic` (0.7.0)
  - Removed from `agnostic-lite` (0.5.6)
  - Removed from `agnostic-net` (0.2.3)
  - Removed from `agnostic-dns` (0.2.2)
  - Removed from `agnostic-process` (0.2.2)
  - Removed `feature-extension-for-async-std` crate

### Changed
- Supported runtimes are now: tokio, smol, and wasm-bindgen-futures (for agnostic-lite)
- Updated all documentation to reflect async-std removal
- Updated workspace dependencies

### Added
- Comprehensive README documentation with examples for all crates
- Feature matrices for complex crates (agnostic-dns)
- Sans-I/O philosophy documentation in agnostic-io
- Runtime switching guides
- Advanced examples (TCP/UDP servers, DNS protocols, etc.)

## agnostic [0.6.0] - 2024-12-15

### Added
- Enhanced feature flags documentation
- Comparison table between agnostic and agnostic-lite

### Changed
- Improved error handling across networking components
- Optimized DNS resolution performance

### Fixed
- Platform-specific build issues on Windows
- Memory leaks in long-lived DNS resolvers

## agnostic-lite [0.5.0] - 2024-12-10

### Added
- WASM support via wasm-bindgen-futures
- Conditional compilation helper macros (cfg_tokio!, cfg_smol!, etc.)
- Yielder trait for cooperative task yielding

### Changed
- Split Runtime trait into smaller, focused traits
- Improved no_std compatibility
- Enhanced embedded systems support

### Fixed
- Timer precision issues in some runtime configurations
- Memory safety issues in local task spawning

## agnostic-net [0.2.0] - 2024-12-01

### Added
- UDP socket support
- Platform-specific optimizations for Unix and Windows
- TCP connection pooling examples
- Broadcast UDP server example

### Changed
- Improved API consistency with std::net
- Enhanced error messages
- Better cross-platform support

### Fixed
- Connection timeout issues
- Socket option setting failures on some platforms

## agnostic-dns [0.2.0] - 2024-11-25

### Added
- DNS over QUIC (DoQ) support (RFC 9250)
- DNS over HTTP/3 (DoH3) support
- DNSSEC validation with ring crypto library
- Comprehensive feature matrix
- Examples for all DNS transport protocols

### Changed
- Updated to hickory-dns 0.24
- Improved caching mechanism
- Better configuration options

### Fixed
- DNSSEC validation errors on some domains
- Memory leaks in long-running resolvers
- Timeout handling in DoH/DoT connections

## agnostic-process [0.2.0] - 2024-11-20

### Added
- Stdio redirection support
- Child process management improvements
- Exit status handling

### Changed
- Better error propagation
- Improved cross-platform compatibility

### Fixed
- Process zombie prevention
- Stdio deadlock issues

## agnostic-io [0.1.2] - 2024-11-15

### Added
- Comprehensive Sans-I/O documentation
- Protocol implementation examples
- AsyncBufRead trait extensions

### Changed
- Improved tokio compatibility layer
- Better no_std support

### Fixed
- Trait bound issues in some configurations

## agnostic-lite [0.4.0] - 2024-11-01

### Added
- Time abstractions (AsyncSleep, AsyncInterval, AsyncTimeout)
- Local task spawning support
- RuntimeLite trait combining all capabilities

### Changed
- Moved from monolithic Runtime trait to focused traits
- Improved documentation

## agnostic [0.5.0] - 2024-10-15

### Added
- Quinn QUIC protocol support
- Process management abstraction
- DNS resolution with multiple transport protocols

### Changed
- Restructured crate organization into workspace
- Split into specialized sub-crates

## agnostic-net [0.1.0] - 2024-10-01

### Added
- Initial release
- TCP listener and stream abstractions
- Runtime-agnostic networking for tokio, async-std, and smol

## agnostic-lite [0.3.0] - 2024-09-20

### Added
- Allocation-free operation mode
- Embedded systems support
- No unsafe code guarantee

### Changed
- Improved no_std compatibility
- Reduced memory footprint

## agnostic [0.4.0] - 2024-09-01

### Added
- Support for smol runtime
- Comprehensive test suite
- CI/CD pipeline

### Changed
- Improved error handling
- Better documentation

## agnostic [0.3.0] - 2024-08-15

### Added
- Support for async-std runtime (now removed in 0.7.0)
- Configuration macros for conditional compilation

### Changed
- Improved API ergonomics
- Better runtime detection

## agnostic [0.2.0] - 2024-08-01

### Added
- Basic networking abstractions
- Initial DNS support

### Changed
- Refactored internal architecture
- Improved performance

### Fixed
- Various runtime compatibility issues

## agnostic-lite [0.2.0] - 2024-07-20

### Added
- Initial time abstractions
- Task spawning traits

### Changed
- Simplified API surface

## agnostic [0.1.0] - 2024-07-01

### Added
- Initial release
- Basic tokio support
- Core runtime abstractions
- Task spawning capabilities

## agnostic-lite [0.1.0] - 2024-07-01

### Added
- Initial release
- Core trait definitions
- no_std support
- Zero unsafe code

---

## Migration Guide: Removing async-std Support

### From 0.6.x to 0.7.x (agnostic)
### From 0.4.x to 0.5.x (agnostic-lite)
### From 0.1.x to 0.2.x (agnostic-net, agnostic-dns, agnostic-process)

If you were using async-std, you need to migrate to either tokio or smol:

#### Option 1: Migrate to tokio

```toml
# Before
[dependencies]
agnostic = { version = "0.6", features = ["async-std"] }

# After
[dependencies]
agnostic = { version = "0.7", features = ["tokio"] }
tokio = { version = "1", features = ["full"] }
```

```rust
// Before
#[async_std::main]
async fn main() {
    // your code
}

// After
#[tokio::main]
async fn main() {
    // your code remains the same
}
```

#### Option 2: Migrate to smol

```toml
# Before
[dependencies]
agnostic = { version = "0.6", features = ["async-std"] }

# After
[dependencies]
agnostic = { version = "0.7", features = ["smol"] }
smol = "2"
```

```rust
// Before
#[async_std::main]
async fn main() {
    // your code
}

// After
fn main() {
    smol::block_on(async {
        // your code remains the same
    });
}
```

### Why was async-std removed?

The async-std project has been discontinued and is no longer actively maintained. To ensure the long-term sustainability and security of the agnostic ecosystem, we've removed support for async-std in favor of actively maintained runtimes:

- **tokio**: The most widely used async runtime with excellent ecosystem support
- **smol**: A lightweight, minimal-dependency alternative

Your application code using agnostic abstractions remains unchanged - only the runtime initialization needs to be updated.

---

## Links

- [Repository](https://github.com/al8n/agnostic)
- [Documentation](https://docs.rs/agnostic)
- [Issue Tracker](https://github.com/al8n/agnostic/issues)

[Unreleased]: https://github.com/al8n/agnostic/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/al8n/agnostic/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/al8n/agnostic/compare/v0.7.0...v0.7.2
[0.7.0]: https://github.com/al8n/agnostic/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/al8n/agnostic/releases/tag/v0.6.0
