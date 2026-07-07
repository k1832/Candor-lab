//! Frozen test suite from `docs/basket/spec-scheduler.md` §4. Every numbered
//! vector T1..T20 is encoded, named by its vector ID, including the T19 shadow
//! model stress with the spec's exact PRNG constants and per-step queue_dump
//! comparison, and the T20 set_priority-on-BLOCKED vector.

use scheduler::{NPRIO, SchedError, Scheduler, State, Task};

// Convenience: build the 64 caller-owned tasks (ids 0..63) up front. Declared so
// that in each test the `Scheduler` (which holds non-owning handles) is dropped
// before this storage.
fn make_tasks() -> [Box<Task>; 64] {
    std::array::from_fn(|i| Task::new(i as u32))
}

// Assert list integrity (spec §3.4) for a queue: the forward walk equals the
// reverse-of-backward walk, i.e. prev/next links are mutually consistent.
fn assert_integrity(s: &Scheduler, prio: usize) -> Vec<u32> {
    let fwd = s.queue_dump(prio);
    let mut rev = s.queue_dump_rev(prio);
    rev.reverse();
    assert_eq!(fwd, rev, "queue {prio} prev/next links inconsistent");
    fwd
}

// --- Nominal ---------------------------------------------------------------

#[test]
fn t1_admit_fifo_dump() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[0], 1).unwrap();
    s.admit(&t[1], 1).unwrap();
    s.admit(&t[2], 1).unwrap();
    assert_eq!(s.queue_dump(1), vec![0, 1, 2]);
}

#[test]
fn t2_fifo_within_level() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[0], 1).unwrap();
    s.admit(&t[1], 1).unwrap();
    s.admit(&t[2], 1).unwrap();
    assert_eq!(s.pick_next().unwrap().id(), 0);
    s.yield_current().unwrap();
    assert_eq!(s.pick_next().unwrap().id(), 1);
    s.yield_current().unwrap();
    assert_eq!(s.pick_next().unwrap().id(), 2);
}

#[test]
fn t3_strict_priority() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[0], 1).unwrap();
    s.admit(&t[5], 0).unwrap();
    assert_eq!(s.pick_next().unwrap().id(), 5);
}

#[test]
fn t4_yield_to_tail() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[10], 2).unwrap();
    s.admit(&t[11], 2).unwrap();
    assert_eq!(s.pick_next().unwrap().id(), 10);
    s.yield_current().unwrap();
    assert_eq!(s.queue_dump(2), vec![11, 10]);
    assert_eq!(s.pick_next().unwrap().id(), 11);
}

// --- Priority / round-robin ------------------------------------------------

#[test]
fn t5_priority_then_fifo_order() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[20], 0).unwrap();
    s.admit(&t[21], 2).unwrap();
    s.admit(&t[22], 1).unwrap();
    s.admit(&t[23], 0).unwrap();
    // pick+exit to deschedule between picks.
    for expect in [20u32, 23, 22, 21] {
        let id = s.pick_next().unwrap().id();
        assert_eq!(id, expect);
        s.exit(&t[id as usize]).unwrap();
    }
}

#[test]
fn t6_set_priority_moves_to_tail() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[20], 0).unwrap();
    s.admit(&t[23], 0).unwrap();
    s.admit(&t[21], 2).unwrap();
    s.set_priority(&t[21], 0).unwrap();
    assert_eq!(s.queue_dump(0), vec![20, 23, 21]);
    assert_eq!(s.queue_dump(2), Vec::<u32>::new());
}

// --- Mid-list removal / re-insertion ---------------------------------------

#[test]
fn t7_block_middle_then_wake() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    for id in [30, 31, 32, 33] {
        s.admit(&t[id], 1).unwrap();
    }
    s.block(&t[32]).unwrap();
    assert_eq!(assert_integrity(&s, 1), vec![30, 31, 33]);
    s.wake(&t[32]).unwrap();
    assert_eq!(assert_integrity(&s, 1), vec![30, 31, 33, 32]);
}

#[test]
fn t8_exit_middle_then_reexit_errors() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    for id in [40, 41, 42] {
        s.admit(&t[id], 1).unwrap();
    }
    s.exit(&t[41]).unwrap();
    assert_eq!(assert_integrity(&s, 1), vec![40, 42]);
    assert_eq!(s.exit(&t[41]), Err(SchedError::NotSchedulable));
}

#[test]
fn t9_spawn_block_wake_yield_exit_mix() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[50], 1).unwrap();
    assert_eq!(s.pick_next().unwrap().id(), 50);
    s.block(&t[50]).unwrap();
    s.admit(&t[51], 1).unwrap();
    assert_eq!(s.pick_next().unwrap().id(), 51);
    s.yield_current().unwrap();
    s.wake(&t[50]).unwrap();
    assert_eq!(s.queue_dump(1), vec![51, 50]);
}

// --- Boundary --------------------------------------------------------------

#[test]
fn t10_pick_next_empty_is_none() {
    let mut s = Scheduler::new();
    assert!(s.pick_next().is_none());
}

#[test]
fn t11_drain_strict_across_levels() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[10], 0).unwrap();
    s.admit(&t[11], 0).unwrap();
    s.admit(&t[20], 1).unwrap();
    s.admit(&t[21], 1).unwrap();
    s.admit(&t[30], 2).unwrap();
    s.admit(&t[31], 2).unwrap();
    s.admit(&t[40], 3).unwrap();
    s.admit(&t[41], 3).unwrap();
    let mut drained = Vec::new();
    while let Some(h) = s.pick_next() {
        let id = h.id();
        drained.push(id);
        s.exit(&t[id as usize]).unwrap();
    }
    assert_eq!(drained, vec![10, 11, 20, 21, 30, 31, 40, 41]);
    assert!(s.pick_next().is_none());
}

#[test]
fn t12_admit_twice_errors() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[7], 1).unwrap();
    assert_eq!(s.admit(&t[7], 1), Err(SchedError::AlreadyQueued));
}

// --- Error -----------------------------------------------------------------

#[test]
fn t13_admit_bad_prio() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    assert_eq!(s.admit(&t[0], 4), Err(SchedError::BadPrio));
}

#[test]
fn t14_set_priority_bad_prio() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[0], 1).unwrap();
    assert_eq!(s.set_priority(&t[0], 9), Err(SchedError::BadPrio));
}

#[test]
fn t15_wake_ready_errors() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[0], 1).unwrap();
    assert_eq!(s.wake(&t[0]), Err(SchedError::NotBlocked));
}

#[test]
fn t16_block_blocked_errors() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[0], 1).unwrap();
    s.block(&t[0]).unwrap();
    assert_eq!(s.block(&t[0]), Err(SchedError::NotSchedulable));
}

#[test]
fn t17_yield_no_running_errors() {
    let mut s = Scheduler::new();
    assert_eq!(s.yield_current(), Err(SchedError::NoRunning));
}

#[test]
fn t18_exit_unknown_errors() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    assert_eq!(s.exit(&t[0]), Err(SchedError::NotSchedulable));
}

// --- Reschedule on a BLOCKED task ------------------------------------------

#[test]
fn t20_set_priority_on_blocked_takes_effect_at_wake() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    s.admit(&t[61], 0).unwrap();
    s.admit(&t[62], 0).unwrap();
    assert_eq!(s.queue_dump(0), vec![61, 62]);
    s.admit(&t[60], 2).unwrap();
    assert_eq!(s.queue_dump(2), vec![60]);

    s.block(&t[60]).unwrap();
    assert_eq!(s.queue_dump(2), Vec::<u32>::new());

    // BLOCKED: new level recorded, priority field updated, no run-queue change.
    s.set_priority(&t[60], 0).unwrap();
    assert_eq!(s.queue_dump(0), vec![61, 62]);
    assert_eq!(s.queue_dump(2), Vec::<u32>::new());
    assert_eq!(t[60].prio(), 0);
    assert_eq!(t[60].state(), State::Blocked);

    // wake applies the recorded level: tail of level 0.
    s.wake(&t[60]).unwrap();
    assert_eq!(s.queue_dump(0), vec![61, 62, 60]);
    assert_eq!(s.queue_dump(2), Vec::<u32>::new());
}

// --- T19: deterministic pseudo-random stress with shadow model -------------

/// xorshift64 (spec §4.1), seed `0x9E3779B97F4A7C15`.
struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// Language-neutral reference (spec §4.2): four ordered lists, a running slot,
/// and per-task state + recorded priority. Updated by the exact rules of §2 —
/// mirroring the same ambiguity resolutions documented in the library (admit is
/// a no-op when the task is present incl. BLOCKED; pick_next is a no-op while a
/// task is RUNNING).
struct Shadow {
    q: [Vec<u32>; NPRIO],
    running: Option<u32>,
    state: [State; 64],
    prio: [u8; 64],
}

impl Shadow {
    fn new() -> Self {
        Shadow {
            q: Default::default(),
            running: None,
            state: [State::New; 64],
            prio: [0; 64],
        }
    }

    fn present(&self, id: u32) -> bool {
        !matches!(self.state[id as usize], State::New | State::Exited)
    }

    fn remove_from_queue(&mut self, id: u32) {
        let p = self.prio[id as usize] as usize;
        self.q[p].retain(|&x| x != id);
    }

    fn admit(&mut self, id: u32, prio: u8) {
        if self.present(id) {
            return;
        }
        self.state[id as usize] = State::Ready;
        self.prio[id as usize] = prio;
        self.q[prio as usize].push(id);
    }

    fn pick_next(&mut self) {
        if self.running.is_some() {
            return;
        }
        for p in 0..NPRIO {
            if !self.q[p].is_empty() {
                let id = self.q[p].remove(0);
                self.state[id as usize] = State::Running;
                self.running = Some(id);
                return;
            }
        }
    }

    fn yield_current(&mut self) {
        if let Some(id) = self.running.take() {
            self.state[id as usize] = State::Ready;
            self.q[self.prio[id as usize] as usize].push(id);
        }
    }

    fn block(&mut self, id: u32) {
        match self.state[id as usize] {
            State::Ready => self.remove_from_queue(id),
            State::Running => self.running = None,
            _ => return,
        }
        self.state[id as usize] = State::Blocked;
    }

    fn wake(&mut self, id: u32) {
        if self.state[id as usize] != State::Blocked {
            return;
        }
        self.state[id as usize] = State::Ready;
        self.q[self.prio[id as usize] as usize].push(id);
    }

    fn set_priority(&mut self, id: u32, prio: u8) {
        match self.state[id as usize] {
            State::Ready => {
                self.remove_from_queue(id);
                self.prio[id as usize] = prio;
                self.q[prio as usize].push(id);
            }
            State::Running | State::Blocked => self.prio[id as usize] = prio,
            _ => {}
        }
    }

    fn exit(&mut self, id: u32) {
        match self.state[id as usize] {
            State::Ready => self.remove_from_queue(id),
            State::Running => self.running = None,
            State::Blocked => {}
            _ => return,
        }
        self.state[id as usize] = State::Exited;
    }
}

#[test]
fn t19_shadow_model_stress() {
    let t = make_tasks();
    let mut s = Scheduler::new();
    let mut shadow = Shadow::new();
    let mut rng = Xorshift(0x9E37_79B9_7F4A_7C15);

    for step in 0..20_000u32 {
        let w = rng.next();
        let id = ((w >> 3) % 64) as u32;
        let op = w & 0x7;
        let prio = ((w >> 9) % 4) as u8;
        let idx = id as usize;

        match op {
            0 | 1 => {
                let _ = s.admit(&t[idx], prio);
                shadow.admit(id, prio);
            }
            2 => {
                let _ = s.pick_next();
                shadow.pick_next();
            }
            3 => {
                let _ = s.yield_current();
                shadow.yield_current();
            }
            4 => {
                let _ = s.block(&t[idx]);
                shadow.block(id);
            }
            5 => {
                let _ = s.wake(&t[idx]);
                shadow.wake(id);
            }
            6 => {
                let _ = s.set_priority(&t[idx], prio);
                shadow.set_priority(id, prio);
            }
            7 => {
                let _ = s.exit(&t[idx]);
                shadow.exit(id);
            }
            _ => unreachable!(),
        }

        // §4.4 per-step assertions.
        // queue_dump equality + list integrity (3.4) for every level.
        for p in 0..NPRIO {
            let dump = assert_integrity(&s, p);
            assert_eq!(dump, shadow.q[p], "step {step}: queue {p} mismatch");
        }
        // RUNNING classification matches.
        assert_eq!(
            s.running_id(),
            shadow.running,
            "step {step}: running mismatch"
        );
        // Per-task state classification + single-membership (3.3).
        for check_id in 0..64u32 {
            let ci = check_id as usize;
            assert_eq!(
                t[ci].state(),
                shadow.state[ci],
                "step {step}: task {check_id} state mismatch"
            );
            let in_queues = (0..NPRIO)
                .filter(|&p| shadow.q[p].contains(&check_id))
                .count();
            let is_running = shadow.running == Some(check_id);
            let is_blocked = shadow.state[ci] == State::Blocked;
            let locations = in_queues + usize::from(is_running) + usize::from(is_blocked);
            match shadow.state[ci] {
                State::Ready | State::Running | State::Blocked => {
                    assert_eq!(
                        locations, 1,
                        "step {step}: task {check_id} not in exactly one location"
                    )
                }
                State::New | State::Exited => {
                    assert_eq!(
                        locations, 0,
                        "step {step}: task {check_id} unexpectedly present"
                    )
                }
            }
        }
    }
}
