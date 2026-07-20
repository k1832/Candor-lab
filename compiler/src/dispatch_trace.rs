//! Test-only instrumentation for design-0018 gate (d) — the dispatch-consistency
//! check (§5.2). Records the `(target, interface, method)` dispatch key each
//! engine's dispatch path resolves to, so a test can assert the impl that ran
//! equals the impl the checker resolved (`dispatch = resolve`, §4.1) — the check
//! the five-engine differential gate is structurally blind to (§2.2, §6).
//!
//! Off by default: [`record`] is a no-op until [`start`] installs a sink on the
//! current thread. Only the tree-walker (`interp::eval`) and the MIR lowering
//! (`mir::build`) dispatch paths call [`record`]; the native backends inherit
//! MIR's resolution unchanged, so MIR coverage plus the existing byte-exact
//! five-engine corpus gates carry them (no native introspection is claimed).

use std::cell::RefCell;

/// A resolved dispatch key: `(target nominal, interface, method)`.
pub type DispatchKey = (String, String, String);

thread_local! {
    static SINK: RefCell<Option<Vec<DispatchKey>>> = const { RefCell::new(None) };
}

/// Install a fresh recording sink on the current thread. Until this is called
/// (or after [`take`]), [`record`] does nothing.
pub fn start() {
    SINK.with(|s| *s.borrow_mut() = Some(Vec::new()));
}

/// Remove and return the recorded keys, disabling recording again.
pub fn take() -> Vec<DispatchKey> {
    SINK.with(|s| s.borrow_mut().take().unwrap_or_default())
}

/// Record one executed dispatch key, if a sink is installed.
pub fn record(target: &str, iface: &str, method: &str) {
    SINK.with(|s| {
        if let Some(v) = s.borrow_mut().as_mut() {
            v.push((target.to_string(), iface.to_string(), method.to_string()));
        }
    });
}
