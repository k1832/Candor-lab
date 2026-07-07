//! Intrusive-list strict-priority scheduler.
//!
//! Implements `docs/basket/spec-scheduler.md`: `NPRIO = 4` strict-priority
//! levels (0 highest .. 3 lowest), FIFO / round-robin within a level.
//!
//! The run-queues are **intrusive doubly-linked lists**: the link field lives
//! inside each [`Task`], and the scheduler never owns a task. Tasks live in
//! caller-owned storage (e.g. `Box<Task>`); the scheduler only threads them onto
//! its queues through their embedded [`LinkedListLink`]. Insertion at a tail and
//! removal of an element from the middle of a queue (given a handle to it) are
//! both O(1) via the embedded links — no scan.
//!
//! The intrusive-list machinery is provided by the `intrusive-collections`
//! crate (see `README.md` for provenance); the scheduling policy on top is
//! written against the spec.

use std::cell::Cell;

use intrusive_collections::{LinkedList, LinkedListLink, UnsafeRef, intrusive_adapter};

/// Number of strict-priority levels: 0 (highest) .. 3 (lowest).
pub const NPRIO: usize = 4;

/// Lifecycle state of a task (spec §3.3: a task is in exactly one location).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Created but never admitted (unknown to the scheduler).
    New,
    /// In a run-queue, waiting to be picked.
    Ready,
    /// Currently the single RUNNING task.
    Running,
    /// In the blocked set (not in any run-queue).
    Blocked,
    /// Exited; no longer schedulable until re-admitted.
    Exited,
}

/// Errors are returned as values; the scheduler never panics on an invalid
/// request (spec §1.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedError {
    /// Priority outside `0..NPRIO`.
    BadPrio,
    /// Task is already in a queue, RUNNING, or otherwise present.
    AlreadyQueued,
    /// Operation requires a RUNNING task and there is none.
    NoRunning,
    /// Task is BLOCKED, EXITED, or unknown when a schedulable task was required.
    NotSchedulable,
    /// `wake` called on a task that is not BLOCKED.
    NotBlocked,
}

/// A schedulable entity. The [`LinkedListLink`] is embedded here (intrusive);
/// the scheduler threads the task onto a run-queue through this field without
/// owning it. `prio` and `state` use [`Cell`] because the scheduler mutates them
/// through the shared reference it is handed as a handle.
pub struct Task {
    link: LinkedListLink,
    id: u32,
    prio: Cell<u8>,
    state: Cell<State>,
}

impl Task {
    /// Create a task in caller-owned storage. Boxed so its address is stable
    /// while it is threaded onto a queue.
    pub fn new(id: u32) -> Box<Task> {
        Box::new(Task {
            link: LinkedListLink::new(),
            id,
            prio: Cell::new(0),
            state: Cell::new(State::New),
        })
    }

    /// This task's identifier.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// This task's current priority field.
    pub fn prio(&self) -> u8 {
        self.prio.get()
    }

    /// This task's current lifecycle state.
    pub fn state(&self) -> State {
        self.state.get()
    }
}

intrusive_adapter!(TaskAdapter = UnsafeRef<Task>: Task { link => LinkedListLink });

/// The scheduler: four intrusive run-queues and a single RUNNING slot. It owns
/// none of the tasks — every queue holds non-owning [`UnsafeRef`] handles into
/// caller storage.
pub struct Scheduler {
    queues: [LinkedList<TaskAdapter>; NPRIO],
    running: Option<UnsafeRef<Task>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// An empty scheduler: all queues empty, no running task (spec §2.1).
    pub fn new() -> Self {
        Scheduler {
            queues: std::array::from_fn(|_| LinkedList::new(TaskAdapter::NEW)),
            running: None,
        }
    }

    // A task that is READY lives in `queues[task.prio]`. Remove it from there in
    // O(1) using the handle to its embedded links (spec §1.3, §3.5).
    fn unlink_ready(&mut self, task: &Task) {
        let prio = task.prio.get() as usize;
        // SAFETY: `task` is READY, so it is currently linked into this exact
        // queue; the cursor is positioned at its embedded link node.
        let mut cursor = unsafe { self.queues[prio].cursor_mut_from_ptr(task) };
        cursor.remove();
    }

    // Thread a task onto the tail of `queues[prio]`, marking it READY.
    fn enqueue_tail(&mut self, task: &Task, prio: u8) {
        task.prio.set(prio);
        task.state.set(State::Ready);
        // SAFETY: `task` is caller-owned and outlives its queue membership; it
        // is not currently linked into any queue (enforced by the state checks
        // in the callers).
        let handle = unsafe { UnsafeRef::from_raw(task as *const Task) };
        self.queues[prio as usize].push_back(handle);
    }

    /// Insert `task` at the tail of run-queue `prio`; state becomes READY
    /// (spec §2.2).
    ///
    /// `E_BAD_PRIO` if `prio` is out of range. `E_ALREADY_QUEUED` if `task` is
    /// already present (in a queue, RUNNING, or — see README, a deliberate
    /// extension of §2.2 — BLOCKED). Admit is valid only from `New`/`Exited`.
    pub fn admit(&mut self, task: &Task, prio: u8) -> Result<(), SchedError> {
        if prio as usize >= NPRIO {
            return Err(SchedError::BadPrio);
        }
        match task.state.get() {
            State::New | State::Exited => {}
            _ => return Err(SchedError::AlreadyQueued),
        }
        self.enqueue_tail(task, prio);
        Ok(())
    }

    /// Select the head of the lowest-numbered non-empty level, remove it from
    /// its run-queue, mark it RUNNING, and return its handle. `None` (a value)
    /// if all run-queues are empty (spec §2.3).
    ///
    /// If a task is already RUNNING this returns `None` and makes no change:
    /// there is no preemption (spec §5.1), so the caller must yield/block/exit
    /// the current task before picking another (see README).
    pub fn pick_next(&mut self) -> Option<UnsafeRef<Task>> {
        if self.running.is_some() {
            return None;
        }
        for q in self.queues.iter_mut() {
            if let Some(handle) = q.pop_front() {
                handle.state.set(State::Running);
                self.running = Some(handle.clone());
                return Some(handle);
            }
        }
        None
    }

    /// The RUNNING task returns to READY at the tail of its own level; no task
    /// is RUNNING afterward (spec §2.4). `E_NO_RUNNING` if none is running.
    pub fn yield_current(&mut self) -> Result<(), SchedError> {
        let handle = self.running.take().ok_or(SchedError::NoRunning)?;
        let prio = handle.prio.get();
        handle.state.set(State::Ready);
        self.queues[prio as usize].push_back(handle);
        Ok(())
    }

    /// Remove `task` (READY in a queue, or RUNNING) from wherever it is and mark
    /// it BLOCKED (spec §2.5). `E_NOT_SCHEDULABLE` if it is BLOCKED/EXITED/unknown.
    pub fn block(&mut self, task: &Task) -> Result<(), SchedError> {
        match task.state.get() {
            State::Ready => self.unlink_ready(task),
            State::Running => self.running = None,
            _ => return Err(SchedError::NotSchedulable),
        }
        task.state.set(State::Blocked);
        Ok(())
    }

    /// Move a BLOCKED `task` to the tail of its priority level, state READY
    /// (spec §2.6). `E_NOT_BLOCKED` if it is not currently BLOCKED. The level
    /// used is the task's current `prio` field, which a prior `set_priority`
    /// may have updated (spec §2.7 BLOCKED branch).
    pub fn wake(&mut self, task: &Task) -> Result<(), SchedError> {
        if task.state.get() != State::Blocked {
            return Err(SchedError::NotBlocked);
        }
        let prio = task.prio.get();
        self.enqueue_tail(task, prio);
        Ok(())
    }

    /// Reschedule `task` to level `prio` (spec §2.7). If READY, it is moved to
    /// the tail of `prio` immediately. If BLOCKED or RUNNING, only the priority
    /// field is updated and takes effect at the next enqueue (wake/yield).
    /// `E_BAD_PRIO`; `E_NOT_SCHEDULABLE` if EXITED/unknown.
    pub fn set_priority(&mut self, task: &Task, prio: u8) -> Result<(), SchedError> {
        if prio as usize >= NPRIO {
            return Err(SchedError::BadPrio);
        }
        match task.state.get() {
            State::Ready => {
                self.unlink_ready(task);
                self.enqueue_tail(task, prio);
            }
            State::Running | State::Blocked => task.prio.set(prio),
            _ => return Err(SchedError::NotSchedulable),
        }
        Ok(())
    }

    /// Remove `task` from all structures; state EXITED (spec §2.8).
    /// `E_NOT_SCHEDULABLE` if already EXITED/unknown.
    pub fn exit(&mut self, task: &Task) -> Result<(), SchedError> {
        match task.state.get() {
            State::Ready => self.unlink_ready(task),
            State::Running => self.running = None,
            State::Blocked => {}
            _ => return Err(SchedError::NotSchedulable),
        }
        task.state.set(State::Exited);
        Ok(())
    }

    /// The ordered ids in run-queue `prio`, head first (spec §2.9).
    pub fn queue_dump(&self, prio: usize) -> Vec<u32> {
        self.queues[prio].iter().map(|t| t.id).collect()
    }

    /// The ids in run-queue `prio` walked from the **tail backward** via the
    /// `prev` links. A well-formed doubly-linked list (spec §3.4) satisfies
    /// `queue_dump_rev(p).reversed() == queue_dump(p)`; the test suite asserts
    /// this every step to check list integrity.
    pub fn queue_dump_rev(&self, prio: usize) -> Vec<u32> {
        let mut out = Vec::new();
        let mut cursor = self.queues[prio].cursor();
        cursor.move_prev(); // from the null sentinel, move_prev lands on the tail
        while let Some(t) = cursor.get() {
            out.push(t.id);
            cursor.move_prev();
        }
        out
    }

    /// The id of the currently RUNNING task, if any.
    pub fn running_id(&self) -> Option<u32> {
        self.running.as_ref().map(|r| r.id)
    }
}
