# Candor port: driver-like state machine over MMIO

Port of `docs/basket/spec-mmio.md` (frozen) to Candor, per
`BET5_CRITERION.md` §2.4(c). One file, `mmio.cn`, containing the driver and
the full frozen scenario suite (M1–M10 with byte-exact trace comparison,
plus the M11–M14 cross-checks). `main` returns the sentinel **777** on full
success; any scenario failure faults (exit 2).

Toolchain: `prototype/target/release/candor-proto check|run ports/candor/mmio/mmio.cn`.
The implementation/harness split per ruling R14 (`// Test harness` marker) is
verified: the implementation section (222 lines) parses and checks standalone.

## Mechanism (spec §3, for the adjudicator)

The driver is exactly the §3 algorithm: `poll_ready` (MAX_POLLS-capped
STATUS polling), `dev_init` (RESET, poll, ENABLE; TIMEOUT as a value),
`dev_recover` (same accesses, FATAL as a value), `dev_transfer` (LEN/CMD/
ENABLE|START programming, the write-side DATA pump with the per-word stall
reset, the read-side DATA drain, ERROR/TIMEOUT/DONE dispatch on every STATUS
poll, IRQ_ACK on completion), and `dev_run` (the composed path: one init
recovery, one transfer recovery + retry, `DEV_FATAL` as a returned value).
`dev_init` and `dev_transfer` are public entry points per ruling **R9**; M6
drives `dev_transfer` directly after a completed `dev_run`, with no re-init.

All FSM logic, status-bit dispatch, and results are safe value gear: `copy`
enums (`PollRes`/`InitRes`/`RecovRes`/`XferRes`/`RunRes`) and `match`, per
design 0001 §11.3. The driver contains **no loops that are not bounded** by
`MAX_POLLS` or by `n` (spec 3.6) and cannot fault on any in-scope device
behavior.

## Architecture: where the simulated device lives

The spec's artifact under test is the MMIO **trace**, and design 0001 §11.3
fixes the shape: safe FSM logic, valve I/O through `reg_read`/`reg_write`.
The port keeps that seam real:

- **The register window is real memory.** Each scenario owns a 5-word
  `[5]u32` local; its address is `base`. `reg_write`/`reg_read` are the
  §11.3 valves verbatim — `addr_to_ptr[u32](base + off)` +
  `ptr_write`/`ptr_read`, one pointer op each. The interpreter's flat memory
  makes the fixed addresses genuine: the words the driver stores and loads
  are the words the device model reads and drives.
- **The device model is the harness's test double**, below the R14 marker.
  What real hardware does on each bus cycle — advance the 7-state FSM
  (spec 2.4), drive the next STATUS/DATA value — is simulated by two
  hook functions carried in the `Dev` handle as fn-pointer fields
  (design 0001 §6.1 machinery): `on_write` fires after the valve's store;
  `on_read` fires before its load and leaves the value the device presents
  in the register word, which the valve's `ptr_read` then returns. The
  driver passes an opaque `ctx` word through to the hooks (the standard
  callback-context idiom) and never interprets it.

This was chosen over the alternative sketched in the task (harness advances
the device only *between* driver steps) because that alternative cannot be
faithful: the device of spec 2.4 advances **on each STATUS read**
(`reset_ctr`, `rem`) — inside `poll_ready`'s and `dev_transfer`'s loops —
so the reaction must be attached to the access itself. The hook seam does
that while keeping the measured section free of any simulation code: the
implementation names no harness symbol and checks standalone (R14), and in
a real deployment the hooks are the no-ops of the hardware itself.

The device FSM (`DevState`, 7 states) is itself safe gear — `copy enum` +
`match` in `dev_status`/`dev_ctrl` — reacting per spec 2.4 with rulings
**R7** (err_at as an ordered per-transfer-attempt schedule, one entry
consumed per START) and **R8** (CTRL bit-rules evaluated independently in
listed order). Trace recording (`trace_rec`) asserts M12 (only the five
§2.1 registers) on every access.

## Valve shape (Bet 5 data)

The measured section's valve is **two functions, one pointer op each**:

- `reg_write` — one `ptr_write` at `base + off`.
- `reg_read` — one `ptr_read` at `base + off`.

Nothing else in the measured driver is unsafe. This is the thinnest valve of
the three ports so far and exactly §11.3's prediction: pointer-danger
localized to I/O, the logic value-first. Harness-side valves: `sim_load`/
`sim_store` (whole-struct device load/store at its context address, the
§11.1 Pool idiom), the one store in `dev_on_read` that presents the driven
value on the bus, and the per-vector address anchors (`regs`/`sim` locals).

## Ambiguities flagged for adjudication

1. **M11's "every scenario" read literally contradicts M5 and M10.** M11
   says every scenario visits `UNINIT→…→ACTIVE→{COMPLETE|FAULT}`, but M5
   (init only) stops at READY and M10 never leaves RESETTING — by those
   scenarios' own frozen traces. Adopted reading: M11 is a coverage check
   applied per scenario up to the states its own trace can reach (harness:
   `check_chain` + tailored `vis` asserts; M5 asserts READY, M10 asserts
   INITED was *never* reached). Suite-wide, the full chain is covered.
2. **M14 for M8.** "The DATA words written equal `buf`, in order" — with
   recovery, M8's trace has five DATA writes: `[0x11,0x22]` (attempt 1,
   aborted at the fault) then `[0x11,0x22,0x33]` (retry). Adopted reading:
   each attempt's writes are a prefix of `buf` in order; the harness checks
   the exact five-word sequence, which the byte-exact trace comparison pins
   anyway.
3. **Observation (not exercised by the frozen vectors):** the per-word
   stall reset in the write loop (spec 3.4) is trace-unobservable under
   M1–M10 — the simulated device never withholds progress from a write
   transfer for `MAX_POLLS` polls, so a driver lacking the reset produces
   identical traces (verified by mutation). Implemented as specified;
   recorded so the suite's discrimination limits are on the record.

## Friction notes (Bet 5 data)

1. **Fn-pointer fields carried their weight.** The simulation seam reuses
   design 0001 §6.1's vtable machinery (fn-pointer struct fields, top-level
   functions as values, calls through fields) outside its allocator home
   with no checker or interpreter trouble — including hooks that
   `ptr_read`/`ptr_write` a struct containing a `copy enum` field. First
   `check` of the finished file passed with zero diagnostics.
2. **Inherited, not new:** a borrow-mode parameter of fixed-array type still
   does not parse (the `Buf` wrapper struct, allocator-port friction 1); no
   bitwise operators, so status-bit tests are `(v / bit) % 2 == 1`
   (`bit_set`) and the spec's composite `ENABLE|START` is the literal
   `0x03` the spec itself fixes; the defensive `;` after an `unsafe` block
   before a `(`-starting statement (parser quirk) appears in `reg_write`
   and `dev_on_read`.
3. **A pre-read hook is the only faithful spelling of "reading advances the
   device".** `ptr_read` cannot compute, so the device must deposit the
   value it drives *before* the valve's load. This fell out naturally but
   is worth recording: any future volatile-load modeling in Candor needs a
   seam on the *access*, not on the value.

No new checker false positives were hit (the known E0304
match-inside-while pattern was avoided by construction).

## Verification

- `check` + `run` → sentinel 777 (all of M1–M10, M13 determinism re-runs,
  M11/M12/M14 cross-checks inline).
- R14 split: implementation section checks standalone.
- Mutation checks (scratch copies, not committed): elided IRQ_ACK, skipped
  recovery retry, poll cap 7, and a device ignoring `stuck_persists` are
  each caught by the trace comparison; the stall-reset mutation is not
  (ambiguity 3 above).
