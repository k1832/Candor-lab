//! Idiomatic-Rust baseline for the Bet 5 MMIO driver state machine.
//!
//! Implements the driver algorithm (spec-mmio.md §3) over the simulated device
//! model (§2). The driver is generic over the [`Mmio`](hal::Mmio) HAL trait, so the
//! deterministic simulated device drives the frozen trace vectors while a real
//! volatile-address path remains available for hardware.

pub mod constants;
pub mod device;
pub mod driver;
pub mod hal;
