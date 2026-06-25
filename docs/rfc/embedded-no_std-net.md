# RFC: `no_std` / embedded networking via bound-minimization

> **Status:** Draft (P0) — design proposal for review. Tracks [#46](https://github.com/al8n/agnostic/issues/46).

The [#46 follow-up](https://github.com/al8n/agnostic/issues/46) asks that `agnostic-net` itself
support embedding, so users don't reach for `embassy-net` / `embassy-time` directly. This RFC
proposes how.

## Where we already are

- **`embassy-time` is already covered.** `agnostic-lite`'s `embassy` backend wraps it (`sleep` /
  `timeout` / `interval` / `delay` / `Instant`), so `RuntimeLite` already abstracts time on bare
  metal.
- The gap is **networking** — `agnostic-net` is std-only and its traits hard-require `Send + Sync`.

## The obstruction

`embassy-net` (0.7.1) is irreducibly **`!Send + !Sync`**: `TcpSocket<'a>` and `Stack<'d>` are both
`!Send`/`!Sync` (single-core embedded), they borrow a static stack + const-generic buffers
(`TcpClientState<N, TX, RX>`), speak `embedded-io-async`, and address via `core::net::SocketAddr`.
`agnostic-net`'s `Net` / `TcpStream` / `TcpListener` / `UdpSocket` all require `Send + Sync + 'static`
with `+ Send` futures. So `embassy-net` **cannot** implement them as written.

## Approach: bound-minimization, not a `!Send` trait fork

Rather than duplicate the API into a parallel `LocalNet` family, **relax the blanket `Send + Sync`
off the traits and require `Send` only where it's genuinely needed** — at spawn boundaries. Loosening
a supertrait is non-breaking for implementors (tokio/smol still satisfy it; embassy now can too).

### Evidence — strip-and-measure

Removing `Send + Sync` from the six net traits and rebuilding `--workspace --all-features --tests`:

- Baseline: green. After the strip: **81 errors across 44 distinct sites.**
- **76 of 79 errors are "required by a bound in `RuntimeLite::spawn` / `spawn_detach`"** — i.e.
  `spawn(async move { …socket… })` whose future is now `!Send`. That is the *only* place `Send` is
  genuinely needed.
- The trait definitions and all non-spawn library logic compiled **fine without `Send + Sync`** →
  the bound is over-applied.
- Production-library cost ≈ 2 sites (the `into_incoming` impls that kept `+ Send` while the trait
  dropped it — an artifact of the partial strip, gone once both sides relax). **42 of 44 sites are
  the test harness**, all the identical spawn-a-task-holding-a-socket shape.

**Verdict: tractable** — the need is localized to spawn boundaries, not smeared across the API.

## Design

**1. Named futures (`tower::Service` style).** Replace `async fn` / RPITIT-`+Send` with named
associated futures (`type Connect: Future<…>; fn connect(..) -> Self::Connect`). This makes each
future's `Send`-ness **structural** (auto-derived from its fields) and **boundable at the use site**
— no `unsafe`, no `trait_variant` required for the core. Prototyped on both cases:
   - *easy* — `recv_from`: one shared `RecvFrom<'a, S>` wrapping the existing poll primitive (nearly
     free, since `agnostic-net` is already poll-based for UDP/`AsyncRead`/`AsyncWrite`);
   - *hard* — `connect`: a hand-rolled per-backend future (the runtime's own connect future is
     unnameable).

   The prototype compiles `no_std`, and a `where S::Connect: Send` use-site correctly **accepts** the
   tokio-like backend while **rejecting** the `!Send` embassy-like one (`error[E0277]: *const ()
   cannot be sent between threads safely`).

**2. Relax the supertraits.** Drop `Send + Sync` from `Net` / `TcpStream` / `TcpListener` /
`UdpSocket` / `OwnedReadHalf` / `OwnedWriteHalf`. Keep `Unpin` / `'static` — embassy satisfies
`'static` via a `StaticCell` stack (`TcpSocket<'static>`).

**3. Keep std friction-free with a `Send` variant.** Use `trait_variant` (embassy-rs's own tool) —
or a blanket `trait SendNet: Net where Self::TcpStream: Send, …` — so tokio/smol consumers use the
`Send` variant and annotate **nothing**; embassy uses the relaxed core + `spawn_local`. (The 42 test
sites bind on the `Send` variant once, rather than per-spawn.)

**4. The embassy backend.** A `Stack` + static buffer pool registered through an `init()` global
(mirroring the `embassy-executor` backend's `SendSpawner` `OnceLock`); `core::net::SocketAddr` ↔
embassy addressing (maps via `embedded-nal-async`); an `embedded-io-async` ↔ `futures-io` /
`tokio-io` bridge for the read/write traits.

## Does this apply to the rest of the workspace?

Targeted, not uniform:

| Crate | Applies? |
|---|---|
| **agnostic-lite** | **Already done.** The `embassy` backend implements `RuntimeLite` (executor via `SendSpawner` + `embassy-time`). `spawn`'s `Send` is correct there (cross-context send); the `Send` / `spawn_local` split is intrinsic, not over-binding. |
| **agnostic-net** | **Primary target** — relax + named futures + embassy backend (this RFC). |
| **agnostic-dns** | The `Send`-relaxation applies, **but no_std DNS is gated by hickory** (std + `Send`-only `RuntimeProvider`). Embedded DNS would use embassy-net's own `DnsSocket` — a *separate* backend, not a hickory relaxation. Bound-minimization helps the `Send` story but doesn't deliver no_std DNS by itself. |
| **agnostic-process** | **N/A on embedded** — bare metal has no subprocesses; nothing to back an embassy impl. Std-only by nature. |
| **agnostic** (facade) | Benefits **transitively** as the components relax; the work lives in the components. |

Separately, the bound-minimization *audit* (a bound only where used) is a clean win for any crate —
but for the std-only crates it's largely *theoretical* composability (no non-`Send` runtime in
practice), so it should follow the embassy-enabling work, not lead it.

## Phased plan

- **P0** (this RFC): agree on the trait shape — named futures + relaxed bounds + `Send`-variant.
- **P1**: `no_std`-ify `agnostic-net` (`core::net`, a portable I/O error, gate `std::net`) behind a
  feature — additive.
- **P2**: relax bounds + named futures + `Send`-variant; re-verify tokio/smol (the ~44 sites adopt
  the `Send` variant).
- **P3**: the `embassy-net` backend (`Stack` / buffer `init()`, io bridge, addressing) + a
  `thumbv7em-none-eabi` cross-compile gate.
- **P4** (later): embedded DNS via `embassy-net`'s `DnsSocket`, separate from hickory.

## Open questions / risks

- **Std ergonomics:** confirm `trait_variant` yields a `Send`-variant clean enough that existing
  tokio/smol code needs no per-spawn annotation (the 42 test sites are the canary).
- **Named-future boilerplate:** the async *constructors* (`connect` / `accept` / `bind`) need
  hand-rolled poll futures on stable (TAIT-in-associated-type would remove that, but it's nightly).
- **SemVer:** relaxing the public supertrait is a source-break for downstream code relying on
  *implied* `Send` (it must add an explicit bound) → a `0.x` minor bump.
- **no_std error type:** a portable I/O error (custom vs `embedded-io::Error`).
- **Stack/buffer API:** how much to hide vs. hand in (the user owns the `Stack` + static buffers
  regardless on embedded).

---

*Filed as a design proposal — feedback on the trait shape (named futures + relaxed bounds +
`Send`-variant) welcome before P1 starts.*
