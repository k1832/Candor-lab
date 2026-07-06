//! The flat byte-addressable memory (design 0001 §4.2). All runtime values live
//! here at defined addresses; a `rawptr` is a plain `u64` address into it.

/// Reserved null address.
pub const NULL: u64 = 0;
/// Static storage (string literals, top-level `static` values) grows from here.
pub const STATIC_BASE: u64 = 0x1000;
/// The call stack (locals + temporaries) bumps upward from here.
pub const STACK_BASE: u64 = 0x0010_0000;
/// The model's cap; reads/writes beyond it fault (§7.2 "beyond the model").
pub const MAX_ADDR: u64 = 0x1000_0000; // 256 MiB

pub fn round_up(x: u64, align: u64) -> u64 {
    let a = align.max(1);
    x.div_ceil(a) * a
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemErr {
    /// Address (range) is beyond the memory model.
    Oob,
    /// The init-byte guard tripped: reading a never-written byte.
    Uninit,
}

pub struct Mem {
    bytes: Vec<u8>,
    init: Vec<bool>,
    pub static_bump: u64,
    pub stack_bump: u64,
}

impl Default for Mem {
    fn default() -> Self {
        Self::new()
    }
}

impl Mem {
    pub fn new() -> Mem {
        Mem {
            bytes: Vec::new(),
            init: Vec::new(),
            static_bump: STATIC_BASE,
            stack_bump: STACK_BASE,
        }
    }

    fn ensure(&mut self, addr: u64, len: u64) -> Result<(), MemErr> {
        let end = addr.checked_add(len).ok_or(MemErr::Oob)?;
        if end > MAX_ADDR {
            return Err(MemErr::Oob);
        }
        if end as usize > self.bytes.len() {
            self.bytes.resize(end as usize, 0);
            self.init.resize(end as usize, false);
        }
        Ok(())
    }

    /// Reserve `size` bytes on the stack at natural `align`; returns the address.
    pub fn stack_alloc(&mut self, size: u64, align: u64) -> u64 {
        let a = round_up(self.stack_bump, align.max(1));
        self.stack_bump = a + size;
        a
    }

    /// Reserve `size` bytes of static storage.
    pub fn static_alloc(&mut self, size: u64, align: u64) -> u64 {
        let a = round_up(self.static_bump, align.max(1));
        self.static_bump = a + size;
        a
    }

    /// Write raw bytes, growing (and marking initialized) as needed.
    pub fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), MemErr> {
        if data.is_empty() {
            return Ok(());
        }
        self.ensure(addr, data.len() as u64)?;
        let start = addr as usize;
        self.bytes[start..start + data.len()].copy_from_slice(data);
        for b in &mut self.init[start..start + data.len()] {
            *b = true;
        }
        Ok(())
    }

    /// Read `len` bytes. When `guard` is set, faults on any never-written byte
    /// (the prototype diagnostic aid, see the module docs).
    pub fn read(&mut self, addr: u64, len: u64, guard: bool) -> Result<Vec<u8>, MemErr> {
        if len == 0 {
            return Ok(Vec::new());
        }
        self.ensure(addr, len)?;
        let start = addr as usize;
        let end = start + len as usize;
        if guard && self.init[start..end].iter().any(|b| !*b) {
            return Err(MemErr::Uninit);
        }
        Ok(self.bytes[start..end].to_vec())
    }

    /// Copy `len` bytes from `src` to `dst` (a move or copy). Guards the source.
    pub fn copy(&mut self, dst: u64, src: u64, len: u64, guard: bool) -> Result<(), MemErr> {
        let data = self.read(src, len, guard)?;
        self.write(dst, &data)
    }

    // ---- typed scalar helpers (little-endian) ----

    pub fn write_uint(&mut self, addr: u64, value: u128, size: u64) -> Result<(), MemErr> {
        let bytes = value.to_le_bytes();
        self.write(addr, &bytes[..size as usize])
    }

    pub fn read_uint(&mut self, addr: u64, size: u64, guard: bool) -> Result<u128, MemErr> {
        let raw = self.read(addr, size, guard)?;
        let mut buf = [0u8; 16];
        buf[..raw.len()].copy_from_slice(&raw);
        Ok(u128::from_le_bytes(buf))
    }

    pub fn write_u64(&mut self, addr: u64, value: u64) -> Result<(), MemErr> {
        self.write(addr, &value.to_le_bytes())
    }

    pub fn read_u64(&mut self, addr: u64, guard: bool) -> Result<u64, MemErr> {
        Ok(self.read_uint(addr, 8, guard)? as u64)
    }
}
