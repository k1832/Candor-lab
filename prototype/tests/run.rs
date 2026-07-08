//! Stage 4 runtime tests: faults, regimes, conv, bounds, contracts, drop order,
//! partial moves, Box lifecycle, raw-pointer pools, container_of, slices.

use candor_proto::interp::{Fault, FaultKind, Run};
use candor_proto::{run_source, RunResult};

fn run(src: &str) -> Run {
    match run_source(src) {
        RunResult::Ok(r) => r,
        RunResult::Fault(f) => panic!("unexpected fault: {}", f.to_json()),
        RunResult::CheckErrors(d) => panic!(
            "unexpected check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

fn fault(src: &str) -> Fault {
    match run_source(src) {
        RunResult::Fault(f) => f,
        RunResult::Ok(r) => panic!("expected fault, got ret {}", r.ret),
        RunResult::CheckErrors(d) => panic!(
            "expected fault, got check errors: {:?}",
            d.iter().map(|x| &x.code).collect::<Vec<_>>()
        ),
        RunResult::ParseError(d) => panic!("parse error: {}", d.to_json()),
    }
}

// --------------------------------------------------------------------------
// Arithmetic faults and regimes
// --------------------------------------------------------------------------

#[test]
fn add_overflow_faults() {
    let f = fault("fn main() -> i64 { let a: i32 = 2147483647i32; let b: i32 = a + 1i32; return 0; }");
    assert_eq!(f.kind, FaultKind::Overflow);
}

#[test]
fn div_by_zero_faults() {
    let f = fault("fn main() -> i64 { let z: i64 = 0; let q: i64 = 10 / z; return q; }");
    assert_eq!(f.kind, FaultKind::DivByZero);
}

#[test]
fn negate_min_faults() {
    let f = fault("fn main() -> i64 { let m: i32 = 0i32 - 2147483648i32; return 0; }");
    // -2147483648 as i32 literal: build via wrapping? use MIN arithmetic
    assert_eq!(f.kind, FaultKind::Overflow);
}

#[test]
fn wrapping_block_wraps() {
    let r = run("fn main() -> i64 { let a: i32 = 2147483647i32; wrapping { let b: i32 = a + 1i32; trace(conv i64 (b)); } return 0; }");
    assert_eq!(r.trace, vec![-2147483648]);
}

#[test]
fn saturating_block_saturates() {
    let r = run("fn main() -> i64 { let a: i32 = 2147483647i32; saturating { let b: i32 = a + 100i32; trace(conv i64 (b)); } return 0; }");
    assert_eq!(r.trace, vec![2147483647]);
}

// A function CALLED from inside a wrapping block runs under its OWN (checked)
// regime and still faults on its own overflow (textual-only scope, §8.1).
#[test]
fn wrapping_is_textual_not_dynamic() {
    let src = "
        fn bump(x: i32) -> i32 { return x + 1i32; }
        fn main() -> i64 {
            let a: i32 = 2147483647i32;
            wrapping { let b: i32 = bump(a); trace(conv i64 (b)); }
            return 0;
        }";
    let f = fault(src);
    assert_eq!(f.kind, FaultKind::Overflow);
}

// --------------------------------------------------------------------------
// conv
// --------------------------------------------------------------------------

#[test]
fn conv_narrowing_faults_on_loss() {
    let f = fault("fn main() -> i64 { let a: i32 = 300i32; let b: u8 = conv u8 (a); return 0; }");
    assert_eq!(f.kind, FaultKind::ConvLoss);
}

#[test]
fn conv_wrapping_truncates() {
    let r = run("fn main() -> i64 { let a: i32 = 300i32; wrapping { let b: u8 = conv u8 (a); trace(conv i64 (b)); } return 0; }");
    assert_eq!(r.trace, vec![44]); // 300 & 0xFF
}

#[test]
fn conv_saturating_clamps() {
    let r = run("fn main() -> i64 { let a: i32 = 300i32; saturating { let b: u8 = conv u8 (a); trace(conv i64 (b)); } return 0; }");
    assert_eq!(r.trace, vec![255]);
}

#[test]
fn conv_widening_preserves() {
    let r = run("fn main() -> i64 { let a: u8 = 200u8; let b: i64 = conv i64 (a); return b; }");
    assert_eq!(r.ret, 200);
}

// --------------------------------------------------------------------------
// bounds, assert, contracts, panic
// --------------------------------------------------------------------------

#[test]
fn array_index_out_of_bounds_faults() {
    let f = fault("fn main() -> i64 { let a: [3]i64 = [10, 20, 30]; let i: usize = 5; return a[i]; }");
    assert_eq!(f.kind, FaultKind::Bounds);
}

#[test]
fn assert_fault() {
    let f = fault("fn main() -> i64 { let x: i64 = 1; assert(x == 2); return 0; }");
    assert_eq!(f.kind, FaultKind::Assert);
}

#[test]
fn requires_fault() {
    let src = "fn need(x: i64) requires(x > 0) -> i64 { return x; } fn main() -> i64 { return need(0 - 1); }";
    let f = fault(src);
    assert_eq!(f.kind, FaultKind::Requires);
}

#[test]
fn ensures_fault_uses_result() {
    let src = "fn bad() ensures(result > 0) -> i64 { return 0 - 5; } fn main() -> i64 { return bad(); }";
    let f = fault(src);
    assert_eq!(f.kind, FaultKind::Ensures);
}

#[test]
fn ensures_pass() {
    let src = "fn good(x: i64) ensures(result > x) -> i64 { return x + 1; } fn main() -> i64 { return good(41); }";
    assert_eq!(run(src).ret, 42);
}

#[test]
fn panic_fault() {
    let f = fault("fn main() -> i64 { panic(\"boom\"); return 0; }");
    assert_eq!(f.kind, FaultKind::Panic);
    assert_eq!(f.message, "boom");
}

// --------------------------------------------------------------------------
// Drop order (§1.5)
// --------------------------------------------------------------------------

const RES: &str = "struct Res { id: i64 } drop(write self) { trace((deref self).id); }\n";

#[test]
fn locals_drop_lifo() {
    let src = format!("{RES}fn main() -> i64 {{ let a: Res = Res {{ id: 1 }}; let b: Res = Res {{ id: 2 }}; let c: Res = Res {{ id: 3 }}; return 0; }}");
    assert_eq!(run(&src).trace, vec![3, 2, 1]);
}

#[test]
fn fields_drop_reverse_declaration_order() {
    let src = format!("{RES}struct Pair {{ x: Res, y: Res }}\nfn main() -> i64 {{ let p: Pair = Pair {{ x: Res {{ id: 1 }}, y: Res {{ id: 2 }} }}; return 0; }}");
    // reverse field order: y (2) then x (1)
    assert_eq!(run(&src).trace, vec![2, 1]);
}

#[test]
fn array_elements_drop_high_to_low() {
    let src = format!("{RES}fn mk(n: i64) -> Res {{ return Res {{ id: n }}; }}\nfn main() -> i64 {{ let a: [3]Res = [mk(1), mk(2), mk(3)]; return 0; }}");
    assert_eq!(run(&src).trace, vec![3, 2, 1]);
}

#[test]
fn moved_out_local_not_dropped() {
    // b is moved into the return; only a (id 1) drops here, b (id 2) is returned
    let src = format!("{RES}fn sink(r: Res) -> i64 {{ return r.id; }}\nfn main() -> i64 {{ let a: Res = Res {{ id: 1 }}; let b: Res = Res {{ id: 2 }}; let n: i64 = sink(b); return n; }}");
    let r = run(&src);
    // b moved into sink -> dropped inside sink (id 2); a dropped at main end (id 1)
    assert_eq!(r.trace, vec![2, 1]);
    assert_eq!(r.ret, 2);
}

#[test]
fn reassignment_drops_old() {
    let src = format!("{RES}fn main() -> i64 {{ let mut a: Res = Res {{ id: 1 }}; a = Res {{ id: 2 }}; return 0; }}");
    // old value (1) dropped at reassignment, new (2) at scope end
    assert_eq!(run(&src).trace, vec![1, 2]);
}

#[test]
fn partial_move_remaining_dropped_moved_not() {
    // Move field y out (into z which is returned/dropped inside helper); x still dropped.
    let src = format!("{RES}struct Pair {{ x: Res, y: Res }}\nfn consume(r: Res) -> unit {{ trace(100 + r.id); }}\nfn main() -> i64 {{ let p: Pair = Pair {{ x: Res {{ id: 1 }}, y: Res {{ id: 2 }} }}; consume(p.y); return 0; }}");
    // take(p.y): y moved -> dropped inside take: trace(102) then drop y (2). Then main end: x (1) dropped. y not dropped again.
    let r = run(&src);
    assert_eq!(r.trace, vec![102, 2, 1]);
}

#[test]
fn out_arg_drops_old_at_call_site() {
    let src = format!("{RES}fn fill(o: out Res) -> unit {{ o = Res {{ id: 9 }}; }}\nfn main() -> i64 {{ let mut a: Res = Res {{ id: 1 }}; fill(out a); return 0; }}");
    // old a (1) dropped at call site before call; new (9) dropped at scope end
    assert_eq!(run(&src).trace, vec![1, 9]);
}

// --------------------------------------------------------------------------
// Box lifecycle through an in-language bump allocator (§6.1/§6.2)
// --------------------------------------------------------------------------

// A bump allocator over a fixed high heap region (addresses backed lazily by the
// interpreter's flat memory). `freed` counts free() calls; alloc traces nothing.
const ALLOC: &str = "
struct AllocVtable {
    alloc: fn(ctx: rawptr u8, size: usize, align: usize) alloc -> rawptr u8,
    free:  fn(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) alloc -> unit,
}
copy struct Alloc { ctx: rawptr u8, vt: rawptr AllocVtable }
struct Bump { next: usize, end: usize }
fn bump_alloc(ctx: rawptr u8, size: usize, align: usize) -> rawptr u8 {
    unsafe \"pool owns [ctx..)\" {
        let b: Bump = ptr_read(cast_ptr[Bump](ctx));
        let mask: usize = align - 1;
        let aligned: usize = (b.next + mask) / align * align;
        if aligned + size > b.end { return ptr_null[u8](); }
        ptr_write(cast_ptr[Bump](ctx), Bump { next: aligned + size, end: b.end });
        return addr_to_ptr[u8](aligned);
    }
}
fn bump_free(ctx: rawptr u8, ptr: rawptr u8, size: usize, align: usize) -> unit {
    unsafe \"note the free\" { trace(0 - 999); }
}
static BUMP_VT: AllocVtable = AllocVtable { alloc: bump_alloc, free: bump_free };
fn mk_alloc(state: write Bump) -> Alloc {
    unsafe \"handle + backing outlive every Box\" {
        return Alloc { ctx: cast_ptr[u8](addr_of_mut(deref state)), vt: addr_of(BUMP_VT) };
    }
}
";

fn with_alloc(body: &str) -> String {
    format!("{ALLOC}\n{body}")
}

#[test]
fn box_alloc_deref_and_drop_frees() {
    let src = with_alloc(
        "fn main() alloc -> i64 {
            let mut st: Bump = Bump { next: 16777216, end: 25165824 };
            let a: Alloc = mk_alloc(write st);
            match box(a, 77) {
                case BoxResult::oom => return 0 - 1,
                case BoxResult::boxed(b) => {
                    let v: i64 = deref b;
                    trace(v);
                    return v;
                }
            }
        }",
    );
    let r = run(&src);
    assert_eq!(r.ret, 77);
    // trace: value 77 read, then free (-999) when the Box drops at scope end
    assert_eq!(r.trace, vec![77, -999]);
}

#[test]
fn box_unbox_moves_out_and_frees() {
    let src = with_alloc(
        "fn main() alloc -> i64 {
            let mut st: Bump = Bump { next: 16777216, end: 25165824 };
            let a: Alloc = mk_alloc(write st);
            match box(a, 55) {
                case BoxResult::oom => return 0 - 1,
                case BoxResult::boxed(b) => {
                    let v: i64 = unbox(b);
                    return v;
                }
            }
        }",
    );
    let r = run(&src);
    assert_eq!(r.ret, 55);
    assert_eq!(r.trace, vec![-999]); // unbox frees exactly once; no double free
}

#[test]
fn box_oom_path() {
    // end == next: zero capacity, so alloc returns null -> BoxResult::oom.
    let src = with_alloc(
        "fn main() alloc -> i64 {
            let mut st: Bump = Bump { next: 16777216, end: 16777216 };
            let a: Alloc = mk_alloc(write st);
            match box(a, 5) {
                case BoxResult::oom => return 42,
                case BoxResult::boxed(b) => { let v: i64 = unbox(b); return v; }
            }
        }",
    );
    assert_eq!(run(&src).ret, 42);
}

#[test]
fn box_clone_through_stored_handle() {
    let src = with_alloc(
        "fn main() alloc -> i64 {
            let mut st: Bump = Bump { next: 16777216, end: 25165824 };
            let a: Alloc = mk_alloc(write st);
            match box(a, 30) {
                case BoxResult::oom => return 0 - 1,
                case BoxResult::boxed(b) => {
                    let c: Box i64 = clone b;
                    let x: i64 = deref b;
                    let y: i64 = deref c;
                    return x + y;
                }
            }
        }",
    );
    let r = run(&src);
    assert_eq!(r.ret, 60);
    // both boxes drop at scope end -> two frees
    assert_eq!(r.trace, vec![-999, -999]);
}

// --------------------------------------------------------------------------
// Free-list pool allocator threaded through raw pointers (§11.1 pattern)
// --------------------------------------------------------------------------

// A mini pool: `head` chains free blocks; the `next` pointer of each free block
// lives *inside the block itself* (raw-pointer valve). alloc pops head; free
// pushes onto head. We build a 2-block free list in a scratch region, then
// alloc/free/re-alloc and assert LIFO reuse of the same addresses.
#[test]
fn free_list_pool_reuses_blocks_lifo() {
    let src = "
        struct Pool { head: rawptr u8 }
        fn pool_alloc(p: write Pool) -> rawptr u8 {
            unsafe \"pop head; *block is next free\" {
                let h: rawptr u8 = (deref p).head;
                if is_null(h) { return ptr_null[u8](); }
                let next: rawptr u8 = ptr_read(cast_ptr[rawptr u8](h));
                (deref p).head = next;
                return h;
            }
        }
        fn pool_free(p: write Pool, block: rawptr u8) -> unit {
            unsafe \"push block onto head; store old head into *block\" {
                let old: rawptr u8 = (deref p).head;
                ptr_write(cast_ptr[rawptr u8](block), old);
                (deref p).head = block;
            }
        }
        fn main() -> i64 {
            unsafe \"build a 2-block free list at fixed addresses\" {
                let b0: rawptr u8 = addr_to_ptr[u8](2097152);
                let b1: rawptr u8 = addr_to_ptr[u8](2097216);
                // free list: head -> b0 -> b1 -> null
                ptr_write(cast_ptr[rawptr u8](b1), ptr_null[u8]());
                ptr_write(cast_ptr[rawptr u8](b0), b1);
                let mut pool: Pool = Pool { head: b0 };
                let a0: rawptr u8 = pool_alloc(write pool);   // pops b0
                let a1: rawptr u8 = pool_alloc(write pool);   // pops b1
                // use the blocks: store a u32 in each
                ptr_write(cast_ptr[u32](a0), 111u32);
                ptr_write(cast_ptr[u32](a1), 222u32);
                let v0: u32 = ptr_read(cast_ptr[u32](a0));
                let v1: u32 = ptr_read(cast_ptr[u32](a1));
                // free a0 then re-alloc: must hand back a0 (LIFO)
                pool_free(write pool, a0);
                let a2: rawptr u8 = pool_alloc(write pool);
                let mut same: i64 = 0;
                if ptr_to_addr(a2) == ptr_to_addr(a0) { same = 1; }
                return conv i64 (v0) + conv i64 (v1) + same;   // 111 + 222 + 1 = 334
            }
        }";
    assert_eq!(run(src).ret, 334);
}

// --------------------------------------------------------------------------
// container_of: recover an outer struct from an interior field pointer (§11.2)
// --------------------------------------------------------------------------

#[test]
fn container_of_recovers_outer_struct() {
    let src = "
        struct Link { next: rawptr Link, prev: rawptr Link }
        struct Task { link: Link, prio: u8, id: i64 }
        fn task_of(link: rawptr Link) -> rawptr Task {
            unsafe \"link is the `link` field of a live Task\" {
                return cast_ptr[Task](ptr_offset(cast_ptr[u8](link),
                                                 0 - conv isize (offsetof(Task, link))));
            }
        }
        fn main() -> i64 {
            unsafe \"take interior pointer then recover the Task\" {
                let mut t: Task = Task { link: Link { next: ptr_null[Link](), prev: ptr_null[Link]() }, prio: 3u8, id: 4242 };
                let lp: rawptr Link = addr_of_mut(t.link);
                let tp: rawptr Task = task_of(lp);
                let back: Task = ptr_read(tp);
                return back.id;
            }
        }";
    assert_eq!(run(src).ret, 4242);
}

// --------------------------------------------------------------------------
// Slices: index, len, subslice, mutation through slice_mut (§5)
// --------------------------------------------------------------------------

#[test]
fn slice_index_len_and_subslice() {
    let src = "
        fn main() -> i64 {
            let a: [5]i64 = [10, 20, 30, 40, 50];
            let s: slice i64 = slice_of(a);
            let n: usize = len(s);
            let sub: slice i64 = subslice(s, 1, 4);   // [20,30,40]
            let m: usize = len(sub);
            let x: i64 = sub[2];                       // 40
            return conv i64 (n) + conv i64 (m) + x;   // 5 + 3 + 40 = 48
        }";
    assert_eq!(run(src).ret, 48);
}

#[test]
fn slice_mut_mutation_writes_through() {
    let src = "
        fn bump3(s: slice_mut i64) -> unit {
            s[0] = s[0] + 1;
            s[1] = s[1] + 1;
            s[2] = s[2] + 1;
        }
        fn main() -> i64 {
            let mut a: [3]i64 = [5, 6, 7];
            bump3(slice_of_mut(a));
            return a[0] + a[1] + a[2];   // 6+7+8 = 21
        }";
    assert_eq!(run(src).ret, 21);
}

#[test]
fn subslice_out_of_bounds_faults() {
    let f = fault("fn main() -> i64 { let a: [3]i64 = [1,2,3]; let s: slice i64 = slice_of(a); let sub: slice i64 = subslice(s, 1, 9); return 0; }");
    assert_eq!(f.kind, FaultKind::Bounds);
}

// --------------------------------------------------------------------------
// E0309 accepted-program runtime behavior is unchanged (finding 2026-07-07):
// a needs-drop local whose init is path-independent still drops correctly, and
// the scope-exit debug assertion (active in these debug test builds) holds.
// --------------------------------------------------------------------------

#[test]
fn drop_hooked_conditional_init_both_branches_runs() {
    // Accepted by E0309 (initialized on every path); dropped once at scope end.
    let src = format!(
        "{RES}fn mk(n: i64) -> Res {{ return Res {{ id: n }}; }} \
         fn main() -> i64 {{ let c: bool = true; let x: Res; \
         if c {{ x = mk(7); }} else {{ x = mk(8); }} return 0; }}"
    );
    assert_eq!(run(&src).trace, vec![7]);
}

#[test]
fn drop_hooked_uninit_on_all_taken_paths_no_drop() {
    // Accepted by E0309 (Uninit on the taken path is path-independent here);
    // the scope-exit mask read correctly skips the drop — no trace.
    let src = format!(
        "{RES}fn mk(n: i64) -> Res {{ return Res {{ id: n }}; }} \
         fn sink(r: Res) -> unit {{ trace(100 + r.id); }} \
         fn main() -> i64 {{ let c: bool = false; let x: Res; \
         if c {{ x = mk(7); sink(x); }} return 0; }}"
    );
    let r = run(&src);
    assert!(r.trace.is_empty(), "expected no drop, got {:?}", r.trace);
}

#[test]
fn ensures_reading_live_param_runs() {
    // Review #3 control: a clause reading a still-live param checks clean and is
    // evaluated at the normal return without touching moved/freed state.
    let src = "struct R { v: i64 } \
               fn f(x: R) ensures(x.v == 7) -> i64 { return x.v; } \
               fn main() -> i64 { return f(R { v: 7 }); }";
    assert_eq!(run(src).ret, 7);
}

// --------------------------------------------------------------------------
// design 0004: `field_ptr` runs (safe field-address projection)
// --------------------------------------------------------------------------

#[test]
fn field_ptr_projects_field_address_and_reads_it() {
    // Compute the address of field `b` in SAFE code via `field_ptr`, then read
    // through it inside `unsafe` (deref stays gated). Expect 32. Design 0004.
    let src = "struct T { a: i64, b: i64 } \
      fn base_ptr(t: write T) -> rawptr T { unsafe \"addr of a live T\" { return addr_of_mut(deref t); } } \
      fn read_i64(p: rawptr i64) -> i64 { unsafe \"p is a valid i64 address\" { return ptr_read(p); } } \
      fn main() -> i64 { let mut t: T = T { a: 10, b: 32 }; \
        let fp: rawptr i64 = field_ptr(base_ptr(write t), b); \
        return read_i64(fp); }";
    assert_eq!(run(src).ret, 32);
}
