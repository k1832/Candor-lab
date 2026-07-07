//! Minimal MMIO HAL: a `Mmio` trait so the tested driver can run over either the
//! simulated device (§2) or a real volatile-address block, plus a tracing decorator
//! that captures the ordered access trace the suite checks (§4.1).

use core::fmt;

/// The five device registers (§2.1), addressed by name so the trait is
/// substitutable; byte offsets are recovered only for the real hardware path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reg {
    Ctrl,
    Status,
    Cmd,
    Data,
    Len,
}

impl Reg {
    /// Fixed byte offset of the register (§2.1).
    pub const fn offset(self) -> usize {
        match self {
            Reg::Ctrl => 0x00,
            Reg::Status => 0x04,
            Reg::Cmd => 0x08,
            Reg::Data => 0x0C,
            Reg::Len => 0x10,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Reg::Ctrl => "CTRL",
            Reg::Status => "STATUS",
            Reg::Cmd => "CMD",
            Reg::Data => "DATA",
            Reg::Len => "LEN",
        }
    }
}

/// A memory-mapped register block: reads and writes are ordered, non-elidable
/// external effects (§1.1).
pub trait Mmio {
    fn read(&mut self, reg: Reg) -> u32;
    fn write(&mut self, reg: Reg, value: u32);
}

/// One observed MMIO access (§4.1): `(op, register, value)`; for reads `value` is
/// the device-returned word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Access {
    pub op: Op,
    pub reg: Reg,
    pub val: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    R,
    W,
}

impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self.op {
            Op::R => 'R',
            Op::W => 'W',
        };
        write!(f, "{} {}=0x{:02X}", op, self.reg.name(), self.val)
    }
}

/// Decorator that records every access it forwards, in program order (§4.1).
pub struct TracingMmio<M> {
    inner: M,
    pub trace: Vec<Access>,
}

impl<M: Mmio> TracingMmio<M> {
    pub fn new(inner: M) -> Self {
        Self {
            inner,
            trace: Vec::new(),
        }
    }
}

impl<M: Mmio> Mmio for TracingMmio<M> {
    fn read(&mut self, reg: Reg) -> u32 {
        let val = self.inner.read(reg);
        self.trace.push(Access {
            op: Op::R,
            reg,
            val,
        });
        val
    }

    fn write(&mut self, reg: Reg, value: u32) {
        self.inner.write(reg, value);
        self.trace.push(Access {
            op: Op::W,
            reg,
            val: value,
        });
    }
}

/// Real-hardware access path: volatile reads/writes against a fixed register base.
/// Not exercised by the suite (the simulated device drives the tests), but present
/// so the driver can bind to a real device without changing the tested path.
pub struct VolatileMmio {
    base: *mut u8,
}

impl VolatileMmio {
    /// # Safety
    /// `base` must point to the start of a valid device register block that stays
    /// mapped for the lifetime of this handle and permits volatile 32-bit access at
    /// every offset in [`Reg::offset`].
    pub unsafe fn new(base: *mut u8) -> Self {
        Self { base }
    }
}

impl Mmio for VolatileMmio {
    fn read(&mut self, reg: Reg) -> u32 {
        // SAFETY: `base` upholds the contract from `new`; the offset is in-block.
        unsafe { self.base.add(reg.offset()).cast::<u32>().read_volatile() }
    }

    fn write(&mut self, reg: Reg, value: u32) {
        // SAFETY: `base` upholds the contract from `new`; the offset is in-block.
        unsafe {
            self.base
                .add(reg.offset())
                .cast::<u32>()
                .write_volatile(value)
        }
    }
}
