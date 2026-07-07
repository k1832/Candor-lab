# MMIO Driver State Machine — idiomatic-Rust baseline

Rust baseline for the Bet 5 validation basket program **(c) Driver-like state
machine over MMIO**, implementing the frozen functional specification
`docs/basket/spec-mmio.md` (device model §2, driver algorithm §3, trace vectors §4).

## Provenance

- **Status:** COMMISSIONED. Written fresh against the frozen spec for the Bet 5
  comparison (per `BET5_CRITERION.md` §2.5, an independently sourced baseline is
  preferred; this one is commissioned — recorded honestly for the adjudicator).
- **Authoring blindness:** produced reading only `docs/basket/spec-mmio.md`,
  `docs/basket/README.md`, and `BET5_CRITERION.md` §2.5. No Candor design docs,
  prototype code, review docs, or other baselines were read.
- **Author:** Claude (Opus model family) session.
- **Date:** 2026-07-07.

## Design

- `src/hal.rs` — the `Mmio` HAL trait (ordered, non-elidable register accesses,
  §1.1). `TracingMmio<M>` is a decorator that captures the access trace the suite
  checks (§4.1). `VolatileMmio` is the real-hardware path (`read_volatile` /
  `write_volatile`); it is the only `unsafe` in the crate and is *not* exercised by
  the tests — the simulated device drives them.
- `src/device.rs` — `SimDevice`, the deterministic simulated device (§2), with the
  scenario flags `init_stuck_first`, `stuck_persists`, and a per-transfer
  `err_schedule` (§2.5). It is the test double behind the `Mmio` trait.
- `src/driver.rs` — `Driver<M>`, the state machine: `poll_ready`, `init`, `recover`,
  `transfer`, `run` (§3). Every failure mode is a returned value; nothing panics
  (§1.4, §3.6).
- `tests/vectors.rs` — every numbered vector M1–M10 as a test named by vector ID,
  asserting byte-exact trace equality against the spec's frozen expected traces.
  M11–M14 (state coverage, register-subset, determinism, data-word equality) are
  covered by assertions inside those tests plus `m13_determinism`.

## How to run

```sh
cd baselines/rust/mmio
cargo test        # runs M1..M10 + cross-checks; all green
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
