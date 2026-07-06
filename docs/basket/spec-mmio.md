# Spec: Driver-Like State Machine over MMIO (`spec-mmio.md`)

**Status:** FROZEN on hash. Authored blind to Candor design docs (see README).
**Source obligation:** `BET5_CRITERION.md` §2.4(c). Restates and sharpens; never
weakens.

---

## 1. Purpose & required features

1.1 The program is a **device driver** implemented as a state machine that
communicates with a **simulated memory-mapped device** exclusively through
**volatile-equivalent register accesses**: reads and writes that the interpreter
observes as **ordered, non-elidable external effects**. No access may be
reordered, duplicated, coalesced, or elided relative to program order.

1.2 The driver MUST implement a device protocol with **at least five states**.
This spec fixes a 7-state device FSM (§2.3) and the exact driver algorithm
(§3) so that two conforming implementations produce **identical MMIO traces**.

1.3 The driver MUST implement **at least two fault-recovery paths**: (a) an
**init timeout** path (device fails to become ready), and (b) a **transfer error**
path (device faults mid-transfer). Both recover by resetting and retrying once,
and both report unrecoverable failure **as a value** (§3.6), never a crash.

1.4 The driver MUST NOT crash/panic/abort on any device behavior in scope.

**Trace = the artifact under test (criterion §2.4c).** "The program" is the driver
whose emitted MMIO trace equals the expected trace of §4 for every scenario. An
easier driver (fewer states, no recovery, elided polling) produces a different
trace and detectably fails.

---

## 2. The simulated device model (binding)

2.1 **Register map.** Word-addressed (32-bit) registers at fixed byte offsets:

| Name   | Offset | Access | Meaning |
|--------|--------|--------|---------|
| CTRL   | 0x00   | write  | control bits |
| STATUS | 0x04   | read   | status bits (reading advances the device, §2.4) |
| CMD    | 0x08   | write  | command code |
| DATA   | 0x0C   | r/w    | data FIFO port |
| LEN    | 0x10   | write  | transfer length in words |

2.2 **Bit / code constants.**
- STATUS: `READY=0x01`, `BUSY=0x02`, `DONE=0x04`, `ERROR=0x08`, `TIMEOUT=0x10`.
- CTRL: `ENABLE=0x01`, `START=0x02`, `RESET=0x04`, `IRQ_ACK=0x08`.
- CMD: `CMD_READ=0x01`, `CMD_WRITE=0x02`.
- `RESET_DELAY = 1`, `MAX_POLLS = 8`.

2.3 **Device states:** `UNINIT`, `RESETTING`, `INITED`, `READY`, `ACTIVE`,
`COMPLETE`, `FAULT` (7 ≥ 5). Internal fields: `rem` (words remaining),
`data_out` (read counter), `err_at` (fault index or none), `stuck` (init-timeout
flag), latched status bits.

2.4 **Device transition rules (deterministic).** Initial: `UNINIT`, all regs 0,
no latched bits.

**On write:**
- CTRL with `RESET` set: if the scenario's `init_stuck_first` flag is armed and
  this is the first RESET, set `stuck := true`, `DS := RESETTING`, consume the
  flag; else `DS := RESETTING`, `reset_ctr := RESET_DELAY`, clear `stuck`, clear
  all latched bits and `err_at`.
- CTRL with `ENABLE` set while `DS == INITED`: `DS := READY`.
- CTRL with `START` set while `DS == READY`: `DS := ACTIVE`, `rem := LEN`,
  `data_out := 0`, `err_at :=` the scenario's configured fault index for the
  *current* transfer (or none).
- CTRL with `IRQ_ACK` set: clear latched `DONE`/`ERROR`.
- LEN: store into `LEN`. CMD: store into `CMD`.
- DATA while `DS == ACTIVE` and `CMD == CMD_WRITE`: accept one word (no state
  advance; advancing happens on STATUS reads).

**On read:**
- STATUS: return a status word AND advance per current `DS`:
  - `UNINIT` -> `0x00`.
  - `RESETTING`: if `stuck` -> `BUSY` (never leaves). Else if `reset_ctr > 0`:
    `reset_ctr -= 1`, return `BUSY`. Else `DS := INITED`, return `READY`.
  - `INITED` / `READY` -> `READY`.
  - `ACTIVE`: let `idx := LEN - rem`. If `err_at` is set and `idx == err_at`:
    `DS := FAULT`, latch `ERROR`, return `ERROR`. Else if `rem > 0`: `rem -= 1`,
    return `BUSY`. Else `DS := COMPLETE`, latch `DONE`, return `READY|DONE`
    (`0x05`).
  - `COMPLETE` -> `READY|DONE` until `IRQ_ACK`, then `READY`.
  - `FAULT` -> `ERROR` until a RESET.
- DATA while `DS == ACTIVE` and `CMD == CMD_READ`: return `data_out`, then
  `data_out += 1`.

2.5 **Scenario configuration** (set by the harness, not via MMIO): `init_stuck_first`
(bool), and a per-transfer `err_at` (word index at which a transfer faults, or
none). All device behavior is a deterministic function of these plus the driver's
access sequence.

---

## 3. The driver algorithm (binding at the register-access level)

The driver MUST issue exactly the accesses below, in order. CTRL write values are
fixed: `RESET=0x04`, `ENABLE=0x01`, `ENABLE|START=0x03`, `IRQ_ACK=0x08`.

3.1 `poll_ready()`: repeat up to `MAX_POLLS` times — read STATUS; if `READY` set
return OK; if `TIMEOUT` set return TIMEOUT. After `MAX_POLLS` reads without READY,
return TIMEOUT.

3.2 `init()`: write CTRL=RESET; `r := poll_ready()`; if `r != OK` return TIMEOUT;
write CTRL=ENABLE; return OK.

3.3 `recover()`: write CTRL=RESET; `r := poll_ready()`; if `r != OK` return
FATAL; write CTRL=ENABLE; return OK.

3.4 `transfer(cmd, n, buf)` (precondition: device READY):
1. write LEN=n; write CMD=cmd; write CTRL=ENABLE|START.
2. `transferred := 0`, `stall := 0`.
3. loop:
   - if `cmd == CMD_WRITE` and `transferred < n`: write DATA=buf[transferred].
   - read STATUS -> `st`.
   - if `st & ERROR`: return XFER_ERROR.
   - if `st & TIMEOUT`: return XFER_TIMEOUT.
   - if `st & DONE`: break.
   - if `st & BUSY`: if `cmd == CMD_READ`: read DATA -> buf[transferred];
     `transferred += 1`; `stall := 0`.
   - else: `stall += 1`; if `stall >= MAX_POLLS` return XFER_TIMEOUT.
4. write CTRL=IRQ_ACK; return OK.

3.5 `run(cmd, n, buf)` (top-level, per scenario):
1. `r := init()`; if `r == TIMEOUT` then `r := recover()`; if `r == FATAL`
   return DEV_FATAL.
2. `t := transfer(cmd, n, buf)`.
3. if `t == XFER_ERROR` or `t == XFER_TIMEOUT`: `r := recover()`; if `r == FATAL`
   return DEV_FATAL; `t := transfer(cmd, n, buf)` (one retry).
4. if `t == OK` return OK; else return DEV_FATAL.

3.6 **Error reporting.** `DEV_FATAL` is a returned value. The driver MUST NOT
crash, spin forever, or leave the device wedged; all loops are bounded by
`MAX_POLLS` (3.1) or by `n` (3.4).

---

## 4. Observable-behavior requirements & frozen trace vectors

4.1 **Trace discipline.** A **trace** is the ordered list of MMIO accesses the
driver issues, each `(op, register, value)` where `op ∈ {W,R}`, and for reads
`value` is the device-returned word. For each scenario the driver's trace MUST
**exactly equal** the expected trace below (same length, order, and values). Extra,
missing, reordered, or elided accesses fail the suite (this enforces
volatile-equivalent, non-elidable, in-order access, §1.1).

4.2 **No stray access.** The driver MUST access only the five registers of §2.1.

Notation: `W REG=hh` / `R REG=hh` (hex). Register names abbreviate offsets.

### Nominal
- **M1 (init + WRITE 2 words [0xAA,0xBB]):**
  `W CTRL=0x04`, `R STATUS=0x02`, `R STATUS=0x01`, `W CTRL=0x01`,
  `W LEN=0x02`, `W CMD=0x02`, `W CTRL=0x03`,
  `W DATA=0xAA`, `R STATUS=0x02`, `W DATA=0xBB`, `R STATUS=0x02`,
  `R STATUS=0x05`, `W CTRL=0x08`. Result OK.
- **M2 (init + READ 3 words):** init as M1
  (`W CTRL=0x04,R STATUS=0x02,R STATUS=0x01,W CTRL=0x01`), then
  `W LEN=0x03`, `W CMD=0x01`, `W CTRL=0x03`,
  `R STATUS=0x02`, `R DATA=0x00`, `R STATUS=0x02`, `R DATA=0x01`,
  `R STATUS=0x02`, `R DATA=0x02`, `R STATUS=0x05`, `W CTRL=0x08`.
  Result OK; `buf == [0,1,2]`.
- **M3 (init + WRITE 1 word [0x77]):** init, then `W LEN=0x01,W CMD=0x02,
  W CTRL=0x03, W DATA=0x77, R STATUS=0x02, R STATUS=0x05, W CTRL=0x08`. OK.

### Boundary
- **M4 (init + transfer LEN 0):** init, then `W LEN=0x00, W CMD=0x02,
  W CTRL=0x03, R STATUS=0x05, W CTRL=0x08`. OK (immediate DONE).
- **M5 (init only, no transfer):** `W CTRL=0x04, R STATUS=0x02, R STATUS=0x01,
  W CTRL=0x01`. Device reaches READY.
- **M6 (state coverage):** M2 followed by another WRITE 2 without re-init (device
  returns to READY after IRQ_ACK): the second transfer's sub-trace equals M1's
  transfer portion. Confirms READY->ACTIVE->COMPLETE->READY cycling.

### Error / recovery
- **M7 (init timeout -> recover -> WRITE 1 [0x77]):** with `init_stuck_first`:
  `W CTRL=0x04`, then `R STATUS=0x02` ×8 (poll exhausted -> TIMEOUT), then recover
  `W CTRL=0x04, R STATUS=0x02, R STATUS=0x01, W CTRL=0x01`, then transfer as M3
  (`W LEN=0x01,W CMD=0x02,W CTRL=0x03,W DATA=0x77,R STATUS=0x02,R STATUS=0x05,
  W CTRL=0x08`). Result OK.
- **M8 (transfer fault at idx 1 -> recover -> retry WRITE 3 [0x11,0x22,0x33]):**
  init (M1 init), first attempt `W LEN=0x03,W CMD=0x02,W CTRL=0x03,
  W DATA=0x11,R STATUS=0x02,W DATA=0x22,R STATUS=0x08` (ERROR -> abort), recover
  `W CTRL=0x04,R STATUS=0x02,R STATUS=0x01,W CTRL=0x01`, retry
  `W LEN=0x03,W CMD=0x02,W CTRL=0x03,W DATA=0x11,R STATUS=0x02,
  W DATA=0x22,R STATUS=0x02,W DATA=0x33,R STATUS=0x02,R STATUS=0x05,W CTRL=0x08`.
  Result OK.
- **M9 (unrecoverable: fault on both attempts):** `err_at=0` armed for every
  transfer; init, attempt faults on first STATUS (`... R STATUS=0x08`), recover,
  retry faults again -> driver returns `DEV_FATAL` (a value). Trace ends after the
  second fault + no further register access. No crash.
- **M10 (init timeout unrecoverable):** `init_stuck_first` armed AND recover's
  RESET also stuck (scenario keeps `stuck`): init poll ×8 TIMEOUT, recover poll
  ×8 TIMEOUT -> `recover()` returns FATAL -> `run` returns `DEV_FATAL`. No crash.

### Cross-checks (structural, all scenarios)
- **M11.** Every scenario visits at least the states
  `UNINIT->RESETTING->INITED->READY->ACTIVE->{COMPLETE|FAULT}`; M7/M10 additionally
  exercise the timeout path; M8/M9 the FAULT path.
- **M12.** For every scenario, the multiset of registers touched ⊆ §2.1 (4.2).
- **M13.** Re-running any scenario yields a byte-identical trace (determinism).
- **M14.** In M1/M3/M8, the DATA words written equal `buf`, in order; in M2 the
  DATA words read populate `buf` as `[0,1,2]` — verified against the trace.

---

## 5. Non-goals

5.1 **Real hardware, real interrupts, DMA, or real timing** are NOT modeled; the
device is the deterministic simulator of §2 and "completion" is a STATUS poll.
5.2 **Concurrency / interrupt reentrancy** is NOT required.
5.3 **Performance** (poll efficiency, batching) is NOT graded; only exact trace
equality and returned results (criterion §8.2).
5.4 **Additional device features** beyond §2's register map MUST NOT be added —
they would perturb the trace and fail §4.
5.5 **Retry counts beyond one** are out of scope; the spec fixes exactly one
retry per recovery path.
