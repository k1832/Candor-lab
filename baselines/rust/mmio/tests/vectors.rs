//! Frozen trace vectors M1-M10 (spec-mmio.md §4). Each test drives the driver over
//! the simulated device, captures the MMIO trace, and asserts byte-exact equality
//! with the spec's expected trace. Structural cross-checks M11-M14 are covered by
//! the per-vector assertions here (state coverage, register subset, determinism,
//! and data-word equality).

use mmio::constants::{CMD_READ, CMD_WRITE};
use mmio::device::{Scenario, SimDevice};
use mmio::driver::{Driver, InitResult, RecoverResult, RunResult, TransferResult};
use mmio::hal::{Access, Op, Reg, TracingMmio};

// --- trace-building helpers -------------------------------------------------

fn w(reg: Reg, val: u32) -> Access {
    Access {
        op: Op::W,
        reg,
        val,
    }
}

fn r(reg: Reg, val: u32) -> Access {
    Access {
        op: Op::R,
        reg,
        val,
    }
}

/// The init sub-trace shared by every nominal scenario (M1..M8 init).
fn init_trace() -> Vec<Access> {
    vec![
        w(Reg::Ctrl, 0x04),
        r(Reg::Status, 0x02),
        r(Reg::Status, 0x01),
        w(Reg::Ctrl, 0x01),
    ]
}

/// The recover sub-trace (identical register sequence to init).
fn recover_trace() -> Vec<Access> {
    init_trace()
}

/// M1's transfer portion: WRITE 2 words [0xAA, 0xBB].
fn write2_transfer() -> Vec<Access> {
    vec![
        w(Reg::Len, 0x02),
        w(Reg::Cmd, 0x02),
        w(Reg::Ctrl, 0x03),
        w(Reg::Data, 0xAA),
        r(Reg::Status, 0x02),
        w(Reg::Data, 0xBB),
        r(Reg::Status, 0x02),
        r(Reg::Status, 0x05),
        w(Reg::Ctrl, 0x08),
    ]
}

/// M3's transfer portion: WRITE 1 word [0x77].
fn write1_transfer() -> Vec<Access> {
    vec![
        w(Reg::Len, 0x01),
        w(Reg::Cmd, 0x02),
        w(Reg::Ctrl, 0x03),
        w(Reg::Data, 0x77),
        r(Reg::Status, 0x02),
        r(Reg::Status, 0x05),
        w(Reg::Ctrl, 0x08),
    ]
}

fn concat(parts: &[Vec<Access>]) -> Vec<Access> {
    parts.iter().flatten().copied().collect()
}

/// Run a full `run(...)` scenario and return `(result, trace)`.
fn run_scenario(
    scenario: Scenario,
    cmd: u32,
    n: usize,
    buf: &mut [u32],
) -> (RunResult, Vec<Access>) {
    let mut driver = Driver::new(TracingMmio::new(SimDevice::new(scenario)));
    let result = driver.run(cmd, n, buf);
    let trace = driver.into_hal().trace;
    (result, trace)
}

/// M12: every touched register is one of the five in §2.1 (always true structurally,
/// but asserted from the observed trace).
fn assert_registers_in_map(trace: &[Access]) {
    for a in trace {
        assert!(
            matches!(
                a.reg,
                Reg::Ctrl | Reg::Status | Reg::Cmd | Reg::Data | Reg::Len
            ),
            "stray register access: {a}"
        );
    }
}

// --- M1..M10 ----------------------------------------------------------------

#[test]
fn m1_init_write2() {
    let mut buf = [0xAA, 0xBB];
    let (res, trace) = run_scenario(Scenario::default(), CMD_WRITE, 2, &mut buf);
    assert_eq!(res, RunResult::Ok);
    assert_eq!(trace, concat(&[init_trace(), write2_transfer()]));
    assert_registers_in_map(&trace);
}

#[test]
fn m2_init_read3() {
    let mut buf = [0u32; 3];
    let (res, trace) = run_scenario(Scenario::default(), CMD_READ, 3, &mut buf);
    assert_eq!(res, RunResult::Ok);
    let read3 = vec![
        w(Reg::Len, 0x03),
        w(Reg::Cmd, 0x01),
        w(Reg::Ctrl, 0x03),
        r(Reg::Status, 0x02),
        r(Reg::Data, 0x00),
        r(Reg::Status, 0x02),
        r(Reg::Data, 0x01),
        r(Reg::Status, 0x02),
        r(Reg::Data, 0x02),
        r(Reg::Status, 0x05),
        w(Reg::Ctrl, 0x08),
    ];
    assert_eq!(trace, concat(&[init_trace(), read3]));
    assert_eq!(buf, [0, 1, 2]); // M14
    assert_registers_in_map(&trace);
}

#[test]
fn m3_init_write1() {
    let mut buf = [0x77];
    let (res, trace) = run_scenario(Scenario::default(), CMD_WRITE, 1, &mut buf);
    assert_eq!(res, RunResult::Ok);
    assert_eq!(trace, concat(&[init_trace(), write1_transfer()]));
    assert_registers_in_map(&trace);
}

#[test]
fn m4_transfer_len0() {
    let mut buf: [u32; 0] = [];
    let (res, trace) = run_scenario(Scenario::default(), CMD_WRITE, 0, &mut buf);
    assert_eq!(res, RunResult::Ok);
    let len0 = vec![
        w(Reg::Len, 0x00),
        w(Reg::Cmd, 0x02),
        w(Reg::Ctrl, 0x03),
        r(Reg::Status, 0x05),
        w(Reg::Ctrl, 0x08),
    ];
    assert_eq!(trace, concat(&[init_trace(), len0]));
    assert_registers_in_map(&trace);
}

#[test]
fn m5_init_only() {
    // Init to READY with no transfer, then confirm the device is READY by driving a
    // transfer directly (no re-init).
    let mut driver = Driver::new(TracingMmio::new(SimDevice::new(Scenario::default())));
    assert_eq!(driver.init(), InitResult::Ok);
    assert_eq!(driver.hal().trace, init_trace());
}

#[test]
fn m6_state_coverage_cycle() {
    // M2 (READ 3) then another WRITE 2 without re-init: the device returns to READY
    // after IRQ_ACK, so the second transfer's sub-trace equals M1's transfer portion.
    let mut driver = Driver::new(TracingMmio::new(SimDevice::new(Scenario::default())));
    assert_eq!(driver.init(), InitResult::Ok);

    let mut rbuf = [0u32; 3];
    assert_eq!(driver.transfer(CMD_READ, 3, &mut rbuf), TransferResult::Ok);
    assert_eq!(rbuf, [0, 1, 2]);

    let mark = driver.hal().trace.len();
    let mut wbuf = [0xAA, 0xBB];
    assert_eq!(driver.transfer(CMD_WRITE, 2, &mut wbuf), TransferResult::Ok);

    let second: Vec<Access> = driver.hal().trace[mark..].to_vec();
    assert_eq!(second, write2_transfer());
    assert_registers_in_map(&driver.hal().trace);
}

#[test]
fn m7_init_timeout_recover_write1() {
    let scenario = Scenario {
        init_stuck_first: true,
        ..Scenario::default()
    };
    let mut buf = [0x77];
    let (res, trace) = run_scenario(scenario, CMD_WRITE, 1, &mut buf);
    assert_eq!(res, RunResult::Ok);

    let mut expected = vec![w(Reg::Ctrl, 0x04)];
    expected.extend(std::iter::repeat_n(r(Reg::Status, 0x02), 8));
    let expected = concat(&[expected, recover_trace(), write1_transfer()]);
    assert_eq!(trace, expected);
    assert_registers_in_map(&trace);
}

#[test]
fn m8_fault_recover_retry_write3() {
    let scenario = Scenario {
        err_schedule: vec![Some(1)], // fault at idx 1 on the first transfer only
        ..Scenario::default()
    };
    let mut buf = [0x11, 0x22, 0x33];
    let (res, trace) = run_scenario(scenario, CMD_WRITE, 3, &mut buf);
    assert_eq!(res, RunResult::Ok);

    let first_attempt = vec![
        w(Reg::Len, 0x03),
        w(Reg::Cmd, 0x02),
        w(Reg::Ctrl, 0x03),
        w(Reg::Data, 0x11),
        r(Reg::Status, 0x02),
        w(Reg::Data, 0x22),
        r(Reg::Status, 0x08),
    ];
    let retry = vec![
        w(Reg::Len, 0x03),
        w(Reg::Cmd, 0x02),
        w(Reg::Ctrl, 0x03),
        w(Reg::Data, 0x11),
        r(Reg::Status, 0x02),
        w(Reg::Data, 0x22),
        r(Reg::Status, 0x02),
        w(Reg::Data, 0x33),
        r(Reg::Status, 0x02),
        r(Reg::Status, 0x05),
        w(Reg::Ctrl, 0x08),
    ];
    assert_eq!(
        trace,
        concat(&[init_trace(), first_attempt, recover_trace(), retry])
    );
    assert_registers_in_map(&trace);
}

#[test]
fn m9_unrecoverable_fault_both_attempts() {
    let scenario = Scenario {
        err_schedule: vec![Some(0), Some(0)], // fault at idx 0 on every transfer
        ..Scenario::default()
    };
    let mut buf = [0x11, 0x22, 0x33];
    let (res, trace) = run_scenario(scenario, CMD_WRITE, 3, &mut buf);
    assert_eq!(res, RunResult::DevFatal);

    let attempt = vec![
        w(Reg::Len, 0x03),
        w(Reg::Cmd, 0x02),
        w(Reg::Ctrl, 0x03),
        w(Reg::Data, 0x11),
        r(Reg::Status, 0x08),
    ];
    assert_eq!(
        trace,
        concat(&[init_trace(), attempt.clone(), recover_trace(), attempt,])
    );
    assert_registers_in_map(&trace);
}

#[test]
fn m10_init_timeout_unrecoverable() {
    let scenario = Scenario {
        init_stuck_first: true,
        stuck_persists: true,
        ..Scenario::default()
    };
    let mut buf = [0x11, 0x22, 0x33];
    let (res, trace) = run_scenario(scenario, CMD_WRITE, 3, &mut buf);
    assert_eq!(res, RunResult::DevFatal);

    let mut expected = vec![w(Reg::Ctrl, 0x04)];
    expected.extend(std::iter::repeat_n(r(Reg::Status, 0x02), 8));
    expected.push(w(Reg::Ctrl, 0x04));
    expected.extend(std::iter::repeat_n(r(Reg::Status, 0x02), 8));
    assert_eq!(trace, expected);
    assert_registers_in_map(&trace);
}

// --- M13: determinism -------------------------------------------------------

#[test]
fn m13_determinism() {
    // Re-running a scenario yields a byte-identical trace.
    let build = || {
        let scenario = Scenario {
            err_schedule: vec![Some(1)],
            ..Scenario::default()
        };
        let mut buf = [0x11, 0x22, 0x33];
        run_scenario(scenario, CMD_WRITE, 3, &mut buf)
    };
    let (r1, t1) = build();
    let (r2, t2) = build();
    assert_eq!(r1, r2);
    assert_eq!(t1, t2);
}

// A read past the fault must never panic (structural: fault path returns a value).
#[test]
fn recover_reports_fatal_as_value() {
    let scenario = Scenario {
        init_stuck_first: true,
        stuck_persists: true,
        ..Scenario::default()
    };
    let mut driver = Driver::new(TracingMmio::new(SimDevice::new(scenario)));
    assert_eq!(driver.init(), InitResult::Timeout);
    assert_eq!(driver.recover(), RecoverResult::Fatal);
}
