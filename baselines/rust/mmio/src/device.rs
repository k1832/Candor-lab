//! The simulated memory-mapped device (spec-mmio.md §2). It is a deterministic
//! function of its scenario configuration (§2.5) plus the driver's access sequence,
//! and serves as the test double behind the `Mmio` trait.

use crate::constants::*;
use crate::hal::{Mmio, Reg};

/// Device FSM states (§2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeviceState {
    Uninit,
    Resetting,
    Inited,
    Ready,
    Active,
    Complete,
    Fault,
}

/// Scenario configuration set by the harness, not via MMIO (§2.5).
#[derive(Debug, Clone, Default)]
pub struct Scenario {
    /// Arm the first RESET to leave the device stuck in RESETTING (§2.4).
    pub init_stuck_first: bool,
    /// When set, a RESET does not clear an existing `stuck` (§2.4); an init timeout
    /// cannot be recovered (M10).
    pub stuck_persists: bool,
    /// Fault index applied to each successive transfer, indexed by transfer count;
    /// `None` (or past the end) means that transfer does not fault (§2.5).
    pub err_schedule: Vec<Option<u32>>,
}

/// The simulated device (§2.4).
pub struct SimDevice {
    state: DeviceState,
    // Latched register values written by the driver.
    len: u32,
    cmd: u32,
    // Internal fields (§2.3).
    rem: u32,
    data_out: u32,
    reset_ctr: u32,
    err_at: Option<u32>,
    stuck: bool,
    latched: u32,
    // Scenario (§2.5).
    scenario: Scenario,
    transfer_count: usize,
}

impl SimDevice {
    pub fn new(scenario: Scenario) -> Self {
        // Initial: UNINIT, all regs 0, no latched bits (§2.4).
        Self {
            state: DeviceState::Uninit,
            len: 0,
            cmd: 0,
            rem: 0,
            data_out: 0,
            reset_ctr: 0,
            err_at: None,
            stuck: false,
            latched: 0,
            scenario,
            transfer_count: 0,
        }
    }

    /// The configured fault index for the current transfer, consumed on START (§2.4).
    fn next_err_at(&mut self) -> Option<u32> {
        let err = self
            .scenario
            .err_schedule
            .get(self.transfer_count)
            .copied()
            .flatten();
        self.transfer_count += 1;
        err
    }

    fn write_ctrl(&mut self, value: u32) {
        if value & RESET != 0 {
            if self.scenario.init_stuck_first {
                // First RESET while armed: latch stuck, enter RESETTING, consume flag.
                self.stuck = true;
                self.state = DeviceState::Resetting;
                self.scenario.init_stuck_first = false;
            } else {
                self.state = DeviceState::Resetting;
                self.reset_ctr = RESET_DELAY;
                self.latched = 0;
                self.err_at = None;
                if !self.scenario.stuck_persists {
                    self.stuck = false;
                }
            }
        }
        if value & ENABLE != 0 && self.state == DeviceState::Inited {
            self.state = DeviceState::Ready;
        }
        if value & START != 0 && self.state == DeviceState::Ready {
            self.state = DeviceState::Active;
            self.rem = self.len;
            self.data_out = 0;
            self.err_at = self.next_err_at();
        }
        if value & IRQ_ACK != 0 {
            self.latched &= !(DONE | ERROR);
            if self.state == DeviceState::Complete {
                self.state = DeviceState::Ready;
            }
        }
    }

    fn read_status(&mut self) -> u32 {
        match self.state {
            DeviceState::Uninit => 0x00,
            DeviceState::Resetting => {
                if self.stuck {
                    BUSY
                } else if self.reset_ctr > 0 {
                    self.reset_ctr -= 1;
                    BUSY
                } else {
                    self.state = DeviceState::Inited;
                    READY
                }
            }
            DeviceState::Inited | DeviceState::Ready => READY,
            DeviceState::Active => {
                let idx = self.len - self.rem;
                if self.err_at == Some(idx) {
                    self.state = DeviceState::Fault;
                    self.latched |= ERROR;
                    ERROR
                } else if self.rem > 0 {
                    self.rem -= 1;
                    BUSY
                } else {
                    self.state = DeviceState::Complete;
                    self.latched |= DONE;
                    READY | DONE
                }
            }
            DeviceState::Complete => READY | DONE,
            DeviceState::Fault => ERROR,
        }
    }

    fn read_data(&mut self) -> u32 {
        if self.state == DeviceState::Active && self.cmd == CMD_READ {
            let word = self.data_out;
            self.data_out += 1;
            word
        } else {
            0
        }
    }
}

impl Mmio for SimDevice {
    fn read(&mut self, reg: Reg) -> u32 {
        match reg {
            Reg::Status => self.read_status(),
            Reg::Data => self.read_data(),
            // CTRL/CMD/LEN are write registers (§2.1); the driver never reads them.
            Reg::Ctrl | Reg::Cmd | Reg::Len => 0,
        }
    }

    fn write(&mut self, reg: Reg, value: u32) {
        match reg {
            Reg::Ctrl => self.write_ctrl(value),
            Reg::Len => self.len = value,
            Reg::Cmd => self.cmd = value,
            // DATA while ACTIVE and CMD_WRITE is accepted with no state advance (§2.4).
            Reg::Data => {}
            // STATUS is read-only (§2.1).
            Reg::Status => {}
        }
    }
}
