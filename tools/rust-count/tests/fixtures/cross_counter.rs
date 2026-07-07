// Shared cross-counter fixture (Rust side).
// Paired with prototype/tests/fixtures/count/cross_counter.cn; both counters
// must report the SAME valve_statements. Hand-computed total = 6.
//
// fn worker: a statement before, an unsafe block holding two statements, a
//   statement after. Valve statements = the unsafe-block statement itself
//   (its span is coincident with the valve region: it intersects without
//   strictly enclosing it, the "partly inside" / partial-overlap case) plus
//   the two inner lets = 3. The enclosing fn and the before/after lets sit
//   outside the valve and are excluded.
// unsafe fn wrapped: an `unsafe fn` makes its whole body a valve region (the
//   Rust correspondent of Candor's body-spanning unsafe block). Valve
//   statements = the fn declaration statement (its span is coincident with the
//   whole-body valve region) + the two body lets = 3.
// Total valve_statements = 3 + 3 = 6.
fn worker() {
    let before = 0;
    unsafe {
        let a = 1;
        let b = 2;
    }
    let after = 3;
}
unsafe fn wrapped() {
    let p = 1;
    let q = 2;
}
