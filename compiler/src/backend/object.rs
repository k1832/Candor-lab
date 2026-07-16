//! Stage B — AOT object emission (design 0010 §1/§5, "cranelift-object shares the
//! IR-building code with cranelift-jit; the delta is module plumbing + a real
//! runtime"). The same MIR->Cranelift-IR lowering (`super::lower`) targets an
//! `ObjectModule` instead of the JIT, emitting a relocatable ELF `.o` for x86-64
//! Linux; the system linker (`cc`) links it with the static C runtime
//! (`aot_runtime.c`) into a standalone native executable that needs neither the
//! JIT nor `candor` at runtime.
//!
//! ## The module-plumbing delta (vs the JIT)
//! * **Flat-memory base.** The JIT bakes the host buffer address as a constant.
//!   AOT bakes a **fixed** virtual address (`MEM_BASE`) that the runtime maps with
//!   `MAP_FIXED` at startup, so `host_addr = MEM_BASE + candor_addr` stays a
//!   compile-time constant and the lowering is otherwise identical.
//! * **Fn-pointer table.** The JIT fills a leaked host array post-finalization.
//!   AOT emits a **data object** of function-address relocations the linker
//!   resolves; indirect calls read it via a relocated `symbol_value`.
//! * **Statics/strings.** Their Candor addresses are laid out here with the same
//!   bump arithmetic the JIT driver uses; the emitted `candor_entry` writes the
//!   string bytes and runs the static initializers at process start.
//! * **Entry + runtime.** `candor_entry` (exported) is the emitted startup glue;
//!   the six `rt_*` shims are imports the linker binds to the C runtime.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use cranelift_codegen::ir::{types, AbiParam};
use cranelift_codegen::settings;
use cranelift_module::{default_libcall_names, DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::interp::layout::Layout;
use crate::interp::mem::{round_up, STATIC_BASE};
use crate::mir::{MirProgram, Rvalue, StatementKind};
use crate::resolve::Items;

use super::lower::{
    collect_glue_types, declare_externs, declare_functions, define_entry, define_functions,
    FnTable, Shims,
};

/// The fixed virtual address the runtime maps the flat buffer at (`MAP_FIXED`).
/// A high, normally-unused region on x86-64 Linux; must match `aot_runtime.c`.
pub const MEM_BASE: i64 = 0x0000_2000_0000_0000;

/// The static C runtime source, compiled by `cc` at link time.
const RUNTIME_C: &str = include_str!("aot_runtime.c");

/// The freestanding (no-libc) runtime source; linked instead of `RUNTIME_C`
/// under `--freestanding` (design 0010 §5; P7/P9/NN#6).
const FREESTANDING_RUNTIME_C: &str = include_str!("freestanding_runtime.c");

/// Lay out static + string-literal Candor addresses with the same bump arithmetic
/// the JIT driver (`backend::run`) uses, so the addresses `candor_entry` writes to
/// agree with the `StaticAddr`/`StrAddr` constants the lowering bakes.
fn layout_statics_strings(
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
) -> (HashMap<String, u64>, HashMap<String, u64>) {
    let lay = Layout { items, consts };
    let mut bump = STATIC_BASE;
    let mut statics = HashMap::new();
    for st in &prog.statics {
        let size = lay.size_of(&st.ty).max(1);
        let align = lay.align_of(&st.ty).max(1);
        let a = round_up(bump, align);
        bump = a + size;
        statics.insert(st.name.clone(), a);
    }
    let mut strings = HashMap::new();
    for s in super::collect_strings(prog) {
        let len = (s.len().max(1)) as u64;
        let a = round_up(bump, 1);
        bump = a + len;
        strings.insert(s, a);
    }
    (statics, strings)
}

/// Emit the linked native executable for `prog` at `out`. Compiles the MIR to a
/// relocatable object, then invokes `cc` to link it with the C runtime.
pub fn emit_executable(
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
    out: &Path,
) -> Result<(), String> {
    let object = compile_object(prog, items, consts)?;
    link(&object, out)
}

/// Emit a FREESTANDING (no-libc) native executable for `prog` at `out` — the
/// NN#6 proof artifact. Same emitted object as `emit_executable`; the delta is
/// the runtime (`freestanding_runtime.c`) and the link flags
/// (`-nostdlib -static -no-pie`, the flat region pinned at `MEM_BASE`).
pub fn emit_executable_freestanding(
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
    out: &Path,
) -> Result<(), String> {
    // Freestanding has no libc: a foreign `extern` call has no symbol to bind, so
    // it is a compile error here (freestanding + FFI is a contradiction; 0011 §5).
    if program_uses_externs(prog, items) {
        return Err(
            "freestanding profile has no libc: a foreign `extern` call cannot be              linked (freestanding + FFI is a contradiction; design 0011 §5)"
                .to_string(),
        );
    }
    let object = compile_object(prog, items, consts)?;
    link_freestanding(&object, out)
}

/// Whether any function body makes a foreign `extern` call (a `Call` whose callee
/// resolves to a boundary extern rather than a MIR fn) — the freestanding profile
/// rejects these (design 0011 §5).
fn program_uses_externs(prog: &MirProgram, items: &Items) -> bool {
    let is_extern_call = |rv: &Rvalue| matches!(rv, Rvalue::Call { func, .. } if items.externs.contains_key(func));
    prog.fns.iter().any(|f| {
        f.blocks.iter().any(|b| {
            b.stmts.iter().any(|st| match &st.kind {
                StatementKind::Assign(_, rv) | StatementKind::Store(_, rv) => is_extern_call(rv),
                _ => false,
            })
        })
    })
}

/// Lower the whole program to a relocatable ELF object (`.o` bytes) for the host.
pub fn compile_object(
    prog: &MirProgram,
    items: &Items,
    consts: &HashMap<String, u64>,
) -> Result<Vec<u8>, String> {
    if prog.get("main").is_none() {
        return Err("no `main` function to compile".to_string());
    }

    // Host ISA, non-PIC (linked `-no-pie`) so the absolute fn-pointer-table
    // relocations and the fixed `MEM_BASE` mapping resolve at link time.
    let flag_builder = settings::builder();
    let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|e| e.to_string())?;
    let builder = ObjectBuilder::new(isa, "candor", default_libcall_names()).map_err(|e| e.to_string())?;
    let mut module = ObjectModule::new(builder);

    let shims = Shims::declare(&mut module)?;
    let (statics, strings) = layout_statics_strings(prog, items, consts);
    let glue_types = collect_glue_types(prog, items, consts);

    // Symbol names are prefixed so the Candor `main` never clashes with the C
    // runtime's `main`; the maps stay keyed by MIR name (calls resolve by id).
    let (func_ids, glue_ids) = declare_functions(&mut module, prog, &glue_types, "cnf_")?;
    let extern_ids = declare_externs(&mut module, items)?;

    // The fn-pointer dispatch table (a data object filled with function-address
    // relocations the linker resolves).
    let table_id = module
        .declare_data("cn_fnptr_table", Linkage::Local, false, false)
        .map_err(|e| e.to_string())?;

    define_functions(
        &mut module,
        prog,
        items,
        consts,
        MEM_BASE,
        &statics,
        &strings,
        FnTable::Data(table_id),
        &shims,
        &func_ids,
        &glue_ids,
        &extern_ids,
        &glue_types,
    )?;

    // Populate the fn-pointer table now that every FuncId is declared.
    let n = prog.fn_ptrs.len().max(1);
    // Real (initialized) data, not `define_zeroinit`: a zeroinit object lands in
    // `.bss`, which stores no bytes on disk, so its function-address relocations
    // would be silently dropped (leaving null slots -> a call to 0). A zeroed byte
    // buffer forces `.data`, where the linker applies the relocations.
    let mut desc = DataDescription::new();
    desc.define(vec![0u8; n * 8].into_boxed_slice());
    for (i, name) in prog.fn_ptrs.iter().enumerate() {
        if let Some(id) = func_ids.get(name) {
            let fref = module.declare_func_in_data(*id, &mut desc);
            desc.write_function_addr((i * 8) as u32, fref);
        }
    }
    module.define_data(table_id, &desc).map_err(|e| e.to_string())?;

    // The exported startup glue the runtime calls.
    let mut esig = module.make_signature();
    esig.returns.push(AbiParam::new(types::I64));
    let entry_id = module
        .declare_function("candor_entry", Linkage::Export, &esig)
        .map_err(|e| e.to_string())?;
    define_entry(
        &mut module,
        prog,
        items,
        consts,
        MEM_BASE,
        &statics,
        &strings,
        FnTable::Data(table_id),
        &shims,
        &func_ids,
        &glue_ids,
        &extern_ids,
        entry_id,
    )?;

    let product = module.finish();
    product.emit().map_err(|e| e.to_string())
}

/// Link the emitted object with the C runtime into a runnable executable via the
/// system compiler driver (`cc`). Non-PIE so the absolute relocations resolve.
fn link(object: &[u8], out: &Path) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!(
        "candor-aot-{}-{}",
        std::process::id(),
        // a cheap unique-ish suffix so concurrent links don't collide
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let obj_path = tmp.join("candor.o");
    let rt_path = tmp.join("aot_runtime.c");
    std::fs::write(&obj_path, object).map_err(|e| e.to_string())?;
    std::fs::write(&rt_path, RUNTIME_C).map_err(|e| e.to_string())?;

    let status = Command::new("cc")
        .arg(&obj_path)
        .arg(&rt_path)
        .arg("-o")
        .arg(out)
        .arg("-no-pie")
        .arg("-pthread")
        .args(linker_select_args())
        .status()
        .map_err(|e| format!("could not invoke the system linker (cc): {e}"))?;

    let _ = std::fs::remove_dir_all(&tmp);
    if !status.success() {
        return Err(format!("linker (cc) failed with status {status}"));
    }
    Ok(())
}

/// Link the emitted object into a FREESTANDING executable: no libc, statically
/// linked, no PIE. The flat memory region (`.candor_flat`) is pinned at the fixed
/// VA `MEM_BASE` so the baked `MEM_BASE + candor_addr` constant resolves with no
/// mmap; `-nostdlib` drops the C runtime and crt startup (the freestanding runtime
/// supplies `_start`); `-static` makes it a self-contained image (`ldd`:
/// "not a dynamic executable"). The result depends on nothing but the kernel.
fn link_freestanding(object: &[u8], out: &Path) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!(
        "candor-fs-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let obj_path = tmp.join("candor.o");
    let rt_path = tmp.join("freestanding_runtime.c");
    std::fs::write(&obj_path, object).map_err(|e| e.to_string())?;
    std::fs::write(&rt_path, FREESTANDING_RUNTIME_C).map_err(|e| e.to_string())?;

    let section_arg = format!("-Wl,--section-start=.candor_flat={:#x}", MEM_BASE);
    let status = Command::new("cc")
        .arg(&rt_path)
        .arg(&obj_path)
        .arg("-o")
        .arg(out)
        .arg("-ffreestanding")
        .arg("-nostdlib")
        .arg("-static")
        .arg("-no-pie")
        .arg("-fno-stack-protector")
        .arg("-fno-pic")
        .arg(&section_arg)
        .arg("-Wl,-e,_start")
        .args(linker_select_args())
        .status()
        .map_err(|e| format!("could not invoke the system linker (cc): {e}"))?;

    let _ = std::fs::remove_dir_all(&tmp);
    if !status.success() {
        return Err(format!("freestanding linker (cc) failed with status {status}"));
    }
    Ok(())
}

/// Opt-in linker selection from the `CANDOR_LINKER` env var (set by
/// `candor compile --linker=<name>`), passed to `cc`/`clang` as `-fuse-ld=<name>`
/// (e.g. `mold`, `lld`, `gold`, `bfd`). Empty/unset -> the system default linker.
///
/// This is deliberately explicit and never "use whatever is installed": the linker
/// is a chosen toolchain component, and a build's linker must be deterministic for
/// the NN#16 reproducibility guarantee (a mold-linked binary differs from an
/// ld-linked one). Selecting a linker that is not installed fails the link with the
/// underlying `cc`/`clang` error.
pub(crate) fn linker_select_args() -> Vec<String> {
    match std::env::var("CANDOR_LINKER") {
        Ok(name) if !name.trim().is_empty() => vec![format!("-fuse-ld={}", name.trim())],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod linker_select_tests {
    use super::linker_select_args;

    // Deterministic (no external linker needed). nextest runs each test in its own
    // process, so the CANDOR_LINKER mutation here is isolated.
    #[test]
    fn env_maps_to_fuse_ld_arg() {
        std::env::remove_var("CANDOR_LINKER");
        assert!(linker_select_args().is_empty(), "unset => system default linker");

        std::env::set_var("CANDOR_LINKER", "mold");
        assert_eq!(linker_select_args(), vec!["-fuse-ld=mold".to_string()]);

        std::env::set_var("CANDOR_LINKER", "  lld  ");
        assert_eq!(linker_select_args(), vec!["-fuse-ld=lld".to_string()], "trims");

        std::env::set_var("CANDOR_LINKER", "   ");
        assert!(linker_select_args().is_empty(), "blank => default");

        std::env::remove_var("CANDOR_LINKER");
    }
}
