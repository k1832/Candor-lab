//! Stage B — AOT object emission (design 0010 §1/§5, "cranelift-object shares the
//! IR-building code with cranelift-jit; the delta is module plumbing + a real
//! runtime"). The same MIR->Cranelift-IR lowering (`super::lower`) targets an
//! `ObjectModule` instead of the JIT, emitting a relocatable ELF `.o` for x86-64
//! Linux; the system linker (`cc`) links it with the static C runtime
//! (`aot_runtime.c`) into a standalone native executable that needs neither the
//! JIT nor `candor-proto` at runtime.
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
use crate::mir::MirProgram;
use crate::resolve::Items;

use super::lower::{
    collect_glue_types, declare_functions, define_entry, define_functions, FnTable, Shims,
};

/// The fixed virtual address the runtime maps the flat buffer at (`MAP_FIXED`).
/// A high, normally-unused region on x86-64 Linux; must match `aot_runtime.c`.
pub const MEM_BASE: i64 = 0x0000_2000_0000_0000;

/// The static C runtime source, compiled by `cc` at link time.
const RUNTIME_C: &str = include_str!("aot_runtime.c");

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
        .status()
        .map_err(|e| format!("could not invoke the system linker (cc): {e}"))?;

    let _ = std::fs::remove_dir_all(&tmp);
    if !status.success() {
        return Err(format!("linker (cc) failed with status {status}"));
    }
    Ok(())
}
