//! Register map, bit/code constants, and timing constants from spec-mmio.md §2.1-§2.2.

// STATUS bits (§2.2).
pub const READY: u32 = 0x01;
pub const BUSY: u32 = 0x02;
pub const DONE: u32 = 0x04;
pub const ERROR: u32 = 0x08;
pub const TIMEOUT: u32 = 0x10;

// CTRL bits (§2.2).
pub const ENABLE: u32 = 0x01;
pub const START: u32 = 0x02;
pub const RESET: u32 = 0x04;
pub const IRQ_ACK: u32 = 0x08;

// CMD codes (§2.2).
pub const CMD_READ: u32 = 0x01;
pub const CMD_WRITE: u32 = 0x02;

// Timing (§2.2).
pub const RESET_DELAY: u32 = 1;
pub const MAX_POLLS: u32 = 8;
