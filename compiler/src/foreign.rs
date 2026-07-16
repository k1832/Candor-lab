//! The foreign-call shim registry (design 0011 §5).
//!
//! The tree-walker cannot execute the C ABI, and no native backend exists yet
//! (0010 is a forward dependency). So a foreign (`extern`) call is dispatched
//! through a **test-only shim registry**: a process-global map from a foreign
//! symbol name to a Rust closure standing in for the C implementation. Both
//! execution engines (the tree-walker and the MIR interpreter) consult this one
//! registry, so a shim-backed extern call produces identical traces on both — the
//! engine-equality the harness tests.
//!
//! A call whose symbol is **not** registered raises the defined
//! [`crate::interp::FaultKind::NoForeignRuntime`] fault (never undefined behavior,
//! never a silent no-op). The registry ships no C and is for harness use only; the
//! shim/real per-symbol differential obligation (0011 §5, 0010 §4) is recorded for
//! when FFI enters the differential corpus.
//!
//! A shim receives the call's scalar arguments **and a handle to the engine's
//! flat memory** (`crate::interp::mem::Mem`, shared by both engines). The memory
//! handle is what lets an I/O shim honor a C pointer-and-length signature: a
//! `write(fd, buf, n)` shim reads `n` bytes at the Candor `buf` address, a
//! `read(fd, buf, n)` shim deposits bytes there (design 0013 std I/O layer).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::interp::mem::Mem;

/// A shim: takes the foreign call's scalar arguments (each widened to `i128`) and
/// a handle to the calling engine's flat memory, and returns its scalar result (a
/// word: pointer, integer, or unit-as-0). The memory handle lets a shim dereference
/// a Candor `rawptr` argument — the byte-buffer half of a POSIX I/O signature.
type ShimFn = Box<dyn Fn(&[i128], &mut Mem) -> i128 + Send + Sync + 'static>;

fn registry() -> &'static Mutex<HashMap<String, ShimFn>> {
    static R: OnceLock<Mutex<HashMap<String, ShimFn>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register (or replace) a shim for a foreign symbol. Test/harness scope only.
pub fn register(symbol: &str, f: impl Fn(&[i128], &mut Mem) -> i128 + Send + Sync + 'static) {
    registry().lock().unwrap().insert(symbol.to_string(), Box::new(f));
}

/// Remove a shim, restoring the `no_foreign_runtime` behavior for that symbol.
pub fn unregister(symbol: &str) {
    registry().lock().unwrap().remove(symbol);
}

/// Is a shim registered for this symbol?
pub fn is_registered(symbol: &str) -> bool {
    registry().lock().unwrap().contains_key(symbol)
}

/// Dispatch a foreign call. `Some(result)` if a shim is registered for `symbol`;
/// `None` signals the caller to raise the `no_foreign_runtime` fault. The engine's
/// flat memory is passed through so an I/O shim can read/write buffer bytes.
pub fn dispatch(symbol: &str, args: &[i128], mem: &mut Mem) -> Option<i128> {
    let reg = registry().lock().unwrap();
    reg.get(symbol).map(|f| f(args, mem))
}
