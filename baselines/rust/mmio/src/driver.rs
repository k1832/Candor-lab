//! The driver algorithm, binding at the register-access level (spec-mmio.md §3).
//! Every access is issued in the exact order §3 prescribes; all failure modes are
//! returned as values, never panics (§1.4, §3.6).

use crate::constants::*;
use crate::hal::{Mmio, Reg};

/// Result of `poll_ready` / `init` (§3.1-§3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitResult {
    Ok,
    Timeout,
}

/// Result of `recover` (§3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverResult {
    Ok,
    Fatal,
}

/// Result of `transfer` (§3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferResult {
    Ok,
    Error,
    Timeout,
}

/// Top-level result of `run` (§3.5); `DevFatal` is a returned value (§3.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunResult {
    Ok,
    DevFatal,
}

/// A driver bound to a memory-mapped device.
pub struct Driver<M> {
    hal: M,
}

impl<M: Mmio> Driver<M> {
    pub fn new(hal: M) -> Self {
        Self { hal }
    }

    /// Borrow the underlying HAL (e.g. to read a captured trace).
    pub fn hal(&self) -> &M {
        &self.hal
    }

    /// Consume the driver, returning the underlying HAL.
    pub fn into_hal(self) -> M {
        self.hal
    }

    /// §3.1: poll STATUS up to `MAX_POLLS` times for READY, surfacing TIMEOUT.
    fn poll_ready(&mut self) -> InitResult {
        for _ in 0..MAX_POLLS {
            let st = self.hal.read(Reg::Status);
            if st & READY != 0 {
                return InitResult::Ok;
            }
            if st & TIMEOUT != 0 {
                return InitResult::Timeout;
            }
        }
        InitResult::Timeout
    }

    /// §3.2: reset, wait for ready, enable.
    pub fn init(&mut self) -> InitResult {
        self.hal.write(Reg::Ctrl, RESET);
        if self.poll_ready() != InitResult::Ok {
            return InitResult::Timeout;
        }
        self.hal.write(Reg::Ctrl, ENABLE);
        InitResult::Ok
    }

    /// §3.3: reset and re-enable; a failed poll is fatal.
    pub fn recover(&mut self) -> RecoverResult {
        self.hal.write(Reg::Ctrl, RESET);
        if self.poll_ready() != InitResult::Ok {
            return RecoverResult::Fatal;
        }
        self.hal.write(Reg::Ctrl, ENABLE);
        RecoverResult::Ok
    }

    /// §3.4: run one transfer. Precondition: device READY. For `CMD_WRITE`, `buf`
    /// supplies the first `n` words; for `CMD_READ`, the first `n` words are filled.
    pub fn transfer(&mut self, cmd: u32, n: usize, buf: &mut [u32]) -> TransferResult {
        self.hal.write(Reg::Len, n as u32);
        self.hal.write(Reg::Cmd, cmd);
        self.hal.write(Reg::Ctrl, ENABLE | START);

        let mut transferred = 0usize;
        let mut stall = 0u32;
        loop {
            if cmd == CMD_WRITE && transferred < n {
                self.hal.write(Reg::Data, buf[transferred]);
                transferred += 1;
                stall = 0;
            }

            let st = self.hal.read(Reg::Status);
            if st & ERROR != 0 {
                return TransferResult::Error;
            }
            if st & TIMEOUT != 0 {
                return TransferResult::Timeout;
            }
            if st & DONE != 0 {
                break;
            }

            // Otherwise BUSY holds.
            if cmd == CMD_READ {
                buf[transferred] = self.hal.read(Reg::Data);
                transferred += 1;
                stall = 0;
            } else {
                stall += 1;
                if stall >= MAX_POLLS {
                    return TransferResult::Timeout;
                }
            }
        }

        self.hal.write(Reg::Ctrl, IRQ_ACK);
        TransferResult::Ok
    }

    /// §3.5: init (recover on timeout), transfer, one recovery+retry on fault.
    pub fn run(&mut self, cmd: u32, n: usize, buf: &mut [u32]) -> RunResult {
        if self.init() == InitResult::Timeout && self.recover() == RecoverResult::Fatal {
            return RunResult::DevFatal;
        }

        let mut t = self.transfer(cmd, n, buf);
        if t == TransferResult::Error || t == TransferResult::Timeout {
            if self.recover() == RecoverResult::Fatal {
                return RunResult::DevFatal;
            }
            t = self.transfer(cmd, n, buf);
        }

        if t == TransferResult::Ok {
            RunResult::Ok
        } else {
            RunResult::DevFatal
        }
    }
}
