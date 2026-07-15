//! Candor Bet 5 validation prototype — Stage 1 (lexer, AST, parser).
//!
//! Structured as a library crate so later stages (checker, interpreter) add
//! modules alongside these without reshaping the front end. The `candor`
//! binary is a thin CLI over `parse_source`.

pub mod ast;
pub mod audit;
pub mod build;
pub mod check;
pub mod count;
pub mod diag;
pub mod foreign;
pub mod foreign_io;
pub mod generics;
pub mod interp;
pub mod lexer;
pub mod manifest;
pub mod mir;
pub mod modules;
pub mod parser;
pub mod real;
pub mod pkg_fetch;
pub mod resolve_pkg;
pub mod resolve;
pub mod span;
pub mod token;
pub mod types;

use ast::Program;
use diag::Diag;

/// Lex then parse a whole source string. Returns the AST or the first `Diag`.
pub fn parse_source(src: &str) -> Result<Program, Diag> {
    let tokens = lexer::lex(src)?;
    parser::parse(tokens)
}

/// Parse (and run the checker for its side effect / admissibility note), then
/// count the program against the frozen unit table (`docs/BET5_UNIT_TABLE.md`).
/// Counting is purely syntactic over the AST; a parse error is returned as-is.
pub fn count_source(src: &str) -> Result<count::Counts, Diag> {
    let program = parse_source(src)?;
    let _diags = check::check_program(&program);
    Ok(count::count_program(&program, src))
}

/// Parse then run the Stage 2 static checker. Returns all diagnostics (empty
/// on success). A parse error is returned as a single-element vector.
pub fn check_source(src: &str) -> Result<Vec<Diag>, Diag> {
    let program = parse_source(src)?;
    Ok(check::check_program(&program))
}

/// Parse, fully check, then execute a program's `main`. Returns the run outcome,
/// a fault, or a list of check diagnostics (as the error path).
pub enum RunResult {
    Ok(interp::Run),
    Fault(interp::Fault),
    CheckErrors(Vec<Diag>),
    ParseError(Diag),
}

pub fn run_source(src: &str) -> RunResult {
    let program = match parse_source(src) {
        Ok(p) => p,
        Err(d) => return RunResult::ParseError(d),
    };
    let diags = check::check_program(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    match interp::run_program(&program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}


// ---------------------------------------------------------------------------
// Real (`.cnr`) surface-syntax entry points (design 0006; spec 01/02).
// Same downstream pipeline as the throwaway front-end; only the parser and the
// real-syntax-only surface checks differ.
// ---------------------------------------------------------------------------

/// Parse a real-syntax (`.cnr`) source string into the shared AST.
pub fn parse_source_real(src: &str) -> Result<Program, Diag> {
    real::parse_source(src)
}

/// Parse + count a real-syntax program against the frozen unit table.
pub fn count_source_real(src: &str) -> Result<count::Counts, Diag> {
    let program = real::parse_source(src)?;
    let _diags = check::check_program_real(&program);
    Ok(count::count_program(&program, src))
}

/// Parse + check a real-syntax program (with the real surface rules enabled).
pub fn check_source_real(src: &str) -> Result<Vec<Diag>, Diag> {
    let program = real::parse_source(src)?;
    Ok(check::check_program_real(&program))
}

/// Parse, check, then run a real-syntax program's `main`.
pub fn run_source_real(src: &str) -> RunResult {
    let program = match real::parse_source(src) {
        Ok(p) => p,
        Err(d) => return RunResult::ParseError(d),
    };
    if generics::is_generic_program(&program) {
        return run_generic(&program);
    }
    let diags = check::check_program_real(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    match interp::run_program(&program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}

/// Check a generic program, then monomorphize it and run the concrete result
/// (design 0007 §5.2: the interpreter executes the instantiated AST, trusting the
/// definition-site check — it re-runs no analysis tier).
fn run_generic(program: &ast::Program) -> RunResult {
    let (diags, insts, shapes) = check::check_generic_program(program, true);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    let mono = generics::monomorphize(program, &insts, &shapes);
    if mono.diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(mono.diags);
    }
    match interp::run_program(&mono.program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}

// ---------------------------------------------------------------------------
// Module-tree (`.cnr` directory) entry points (design 0008 stage 1). A directory
// argument is a module tree; it is discovered, `use`-resolved, visibility- and
// cycle-checked, then merged into one program fed to the shared real-syntax
// pipeline. Single files keep their existing behavior.
// ---------------------------------------------------------------------------

/// Check a module tree rooted at `dir`. Returns module-layer diagnostics
/// (imports, visibility, cycles) followed by the shared checker's diagnostics,
/// or a hard I/O/parse error as a single `Diag`.
pub fn check_dir(dir: &std::path::Path) -> Result<Vec<Diag>, Diag> {
    let build = modules::build_tree(dir)?;
    let mut diags = build.diags;
    diags.extend(check::check_program_real(&build.program));
    Ok(diags)
}

/// Build, check, then run a module tree's `main` (in the root `main.cnr`).
pub fn run_dir(dir: &std::path::Path) -> RunResult {
    let build = match modules::build_tree(dir) {
        Ok(b) => b,
        Err(d) => return RunResult::ParseError(d),
    };
    let mut diags = build.diags;
    if generics::is_generic_program(&build.program) {
        if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
            return RunResult::CheckErrors(diags);
        }
        return run_generic(&build.program);
    }
    diags.extend(check::check_program_real(&build.program));
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return RunResult::CheckErrors(diags);
    }
    match interp::run_program(&build.program) {
        Ok(run) => RunResult::Ok(run),
        Err(f) => RunResult::Fault(f),
    }
}

// ---------------------------------------------------------------------------
// P15 migrator (design 0006 §5): parse throwaway (`.cn`) syntax with the
// existing front-end, then re-emit the shared AST in real (`.cnr`) syntax.
// Semantic fidelity is by construction — the emitter only re-spells the AST the
// throwaway parser produced. See `real::emit` for the mechanical/author-assisted
// row handling.
// ---------------------------------------------------------------------------

/// Migrate a throwaway (`.cn`) source string to real (`.cnr`) surface syntax.
/// Returns the emitted program text, or the throwaway front-end's parse `Diag`.
pub fn migrate_source(src: &str) -> Result<String, Diag> {
    let program = parse_source(src)?;
    Ok(real::emit::emit_program(&program))
}

// ---------------------------------------------------------------------------
// Blessed formatter (P16 / NN#11): parse real (`.cnr`) syntax and re-print it in
// the one canonical form, preserving comments (spec 02 §9; design 0006 §4).
// ---------------------------------------------------------------------------

/// Format a real-syntax (`.cnr`) source string into canonical form. Returns the
/// formatted text or the parse `Diag`. Idempotent (`format(format(x)) == format(x)`).
pub fn format_source_real(src: &str) -> Result<String, Diag> {
    real::fmt::format_source(src)
}


// ---------------------------------------------------------------------------
// Stage A — the MIR interpreter as an alternate execution engine (design 0010
// §5). Same parse+check front; a MIR lowering + precise MIR interpreter replaces
// the tree-walker. Out-of-subset constructs return `Unsupported` (the honest
// coverage boundary), never a silent divergence.
// ---------------------------------------------------------------------------

/// Outcome of running a program through the MIR engine.
pub enum MirRunResult {
    Ok(interp::Run),
    Fault(interp::Fault),
    CheckErrors(Vec<Diag>),
    ParseError(Diag),
    /// The program uses a construct outside the Stage-A MIR subset.
    Unsupported(String),
}

/// Resolve, lower to MIR, and run precisely. Assumes the program already checked.
fn lower_and_run(program: &ast::Program) -> MirRunResult {
    let mut diags = Vec::new();
    let items = resolve::resolve_program(program, &mut diags);
    // Integer-valued statics double as named array lengths (shared with the
    // oracle's `Layout`), so the MIR interpreter sizes aggregates identically.
    let mut consts = std::collections::HashMap::new();
    for it in &program.items {
        if let ast::Item::Static(st) = it {
            if let ast::ExprKind::IntLit { value, .. } = &st.value.kind {
                consts.insert(st.name.clone(), *value);
            }
        }
    }
    match mir::lower_checked(program, &items) {
        Ok(mp) => match mir::interp::run(&mp, &items, &consts) {
            Ok(run) => MirRunResult::Ok(run),
            Err(f) => MirRunResult::Fault(f),
        },
        Err(e) => MirRunResult::Unsupported(e.0),
    }
}

/// Throwaway (`.cn`) source through the MIR engine.
pub fn run_source_mir(src: &str) -> MirRunResult {
    let program = match parse_source(src) {
        Ok(p) => p,
        Err(d) => return MirRunResult::ParseError(d),
    };
    let diags = check::check_program(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run(&program)
}

/// Real (`.cnr`) source through the MIR engine (monomorphizing generics first).
pub fn run_source_real_mir(src: &str) -> MirRunResult {
    let program = match real::parse_source(src) {
        Ok(p) => p,
        Err(d) => return MirRunResult::ParseError(d),
    };
    if generics::is_generic_program(&program) {
        return run_generic_mir(&program);
    }
    let diags = check::check_program_real(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run(&program)
}

/// Check + monomorphize a generic program, then run the concrete result on MIR.
fn run_generic_mir(program: &ast::Program) -> MirRunResult {
    let (diags, insts, shapes) = check::check_generic_program(program, true);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    let mono = generics::monomorphize(program, &insts, &shapes);
    if mono.diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(mono.diags);
    }
    lower_and_run(&mono.program)
}

/// Module-tree (`.cnr` directory) through the MIR engine.
pub fn run_dir_mir(dir: &std::path::Path) -> MirRunResult {
    let build = match modules::build_tree(dir) {
        Ok(b) => b,
        Err(d) => return MirRunResult::ParseError(d),
    };
    let mut diags = build.diags;
    if generics::is_generic_program(&build.program) {
        if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
            return MirRunResult::CheckErrors(diags);
        }
        return run_generic_mir(&build.program);
    }
    diags.extend(check::check_program_real(&build.program));
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run(&build.program)
}

// ---------------------------------------------------------------------------
// Stage B — the native (Cranelift JIT) execution engine (design 0010 §5).
// Same parse+check+monomorphize front and the same MIR as the Stage-A engine;
// only the terminal step differs: `backend::run` compiles the whole MirProgram
// to native code and runs it, returning the identical `Run`/`Fault`. Out-of-
// subset programs still surface as `Unsupported` (never a silent divergence).
// ---------------------------------------------------------------------------

pub mod backend;

/// Resolve, lower to MIR, JIT-compile and run natively. Assumes checked input.
fn lower_and_run_native(program: &ast::Program, optimize: bool) -> MirRunResult {
    let mut diags = Vec::new();
    let items = resolve::resolve_program(program, &mut diags);
    let mut consts = std::collections::HashMap::new();
    for it in &program.items {
        if let ast::Item::Static(st) = it {
            if let ast::ExprKind::IntLit { value, .. } = &st.value.kind {
                consts.insert(st.name.clone(), *value);
            }
        }
    }
    match mir::lower_checked(program, &items) {
        Ok(mp) => {
            // Stage D (native-opt engine): run the validated MIR-level R1 rewrite
            // (INV-R1-ONLY, `mir::opt`) before lowering, then compile with the
            // Cranelift optimizer on. The four-engine gate proves the trace survives.
            let mp = if optimize { mir::opt::optimize(mp).prog } else { mp };
            match backend::run(&mp, &items, &consts, optimize) {
                Ok(run) => MirRunResult::Ok(run),
                Err(f) => MirRunResult::Fault(f),
            }
        }
        Err(e) => MirRunResult::Unsupported(e.0),
    }
}

fn run_generic_native(program: &ast::Program, optimize: bool) -> MirRunResult {
    let (diags, insts, shapes) = check::check_generic_program(program, true);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    let mono = generics::monomorphize(program, &insts, &shapes);
    if mono.diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(mono.diags);
    }
    lower_and_run_native(&mono.program, optimize)
}

/// Throwaway (`.cn`) source through the native engine (no optimization).
pub fn run_source_native(src: &str) -> MirRunResult {
    run_source_native_impl(src, false)
}
/// Stage D: throwaway (`.cn`) source through the OPTIMIZED native engine
/// (MIR R1 rewrite + Cranelift `opt_level=speed`).
pub fn run_source_native_opt(src: &str) -> MirRunResult {
    run_source_native_impl(src, true)
}
fn run_source_native_impl(src: &str, optimize: bool) -> MirRunResult {
    let program = match parse_source(src) {
        Ok(p) => p,
        Err(d) => return MirRunResult::ParseError(d),
    };
    let diags = check::check_program(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run_native(&program, optimize)
}

/// Real (`.cnr`) source through the native engine (monomorphizing generics first).
pub fn run_source_real_native(src: &str) -> MirRunResult {
    run_source_real_native_impl(src, false)
}
/// Stage D: real (`.cnr`) source through the OPTIMIZED native engine.
pub fn run_source_real_native_opt(src: &str) -> MirRunResult {
    run_source_real_native_impl(src, true)
}
fn run_source_real_native_impl(src: &str, optimize: bool) -> MirRunResult {
    let program = match real::parse_source(src) {
        Ok(p) => p,
        Err(d) => return MirRunResult::ParseError(d),
    };
    if generics::is_generic_program(&program) {
        return run_generic_native(&program, optimize);
    }
    let diags = check::check_program_real(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run_native(&program, optimize)
}

/// Module-tree (`.cnr` directory) through the native engine.
pub fn run_dir_native(dir: &std::path::Path) -> MirRunResult {
    run_dir_native_impl(dir, false)
}
/// Stage D: module-tree through the OPTIMIZED native engine.
pub fn run_dir_native_opt(dir: &std::path::Path) -> MirRunResult {
    run_dir_native_impl(dir, true)
}
fn run_dir_native_impl(dir: &std::path::Path, optimize: bool) -> MirRunResult {
    let build = match modules::build_tree(dir) {
        Ok(b) => b,
        Err(d) => return MirRunResult::ParseError(d),
    };
    let mut diags = build.diags;
    if generics::is_generic_program(&build.program) {
        if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
            return MirRunResult::CheckErrors(diags);
        }
        return run_generic_native(&build.program, optimize);
    }
    diags.extend(check::check_program_real(&build.program));
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run_native(&build.program, optimize)
}

// ---------------------------------------------------------------------------
// AOT compilation (design 0010 §5, Stage B's cranelift-object note): the same
// parse+check+monomorphize+lower-to-MIR front as the native engine, but the
// terminal step emits a linked native EXECUTABLE (`backend::object`) instead of
// JIT-running. `candor compile <file_or_dir> -o prog` drives this.
// ---------------------------------------------------------------------------

/// Format the first error diagnostics as a single message.
fn first_error_msg(diags: &[Diag]) -> String {
    diags
        .iter()
        .find(|d| matches!(d.severity, diag::Severity::Error))
        .map(|d| d.to_json())
        .unwrap_or_else(|| "check failed".to_string())
}

fn any_error(diags: &[Diag]) -> bool {
    diags.iter().any(|d| matches!(d.severity, diag::Severity::Error))
}

/// Monomorphize + check a generic program, returning the concrete AST to lower.
fn monomorphized_program(program: &ast::Program) -> Result<ast::Program, String> {
    let (diags, insts, shapes) = check::check_generic_program(program, true);
    if any_error(&diags) {
        return Err(first_error_msg(&diags));
    }
    let mono = generics::monomorphize(program, &insts, &shapes);
    if any_error(&mono.diags) {
        return Err(first_error_msg(&mono.diags));
    }
    Ok(mono.program)
}

/// Produce the checked (and, if generic, monomorphized) AST for a `.cn`/`.cnr`
/// file or a `.cnr` module-tree directory — the same dispatch the native `run`
/// engine performs, stopping just before MIR lowering.
fn checked_program_for_native(path: &std::path::Path) -> Result<ast::Program, String> {
    if path.is_dir() {
        let build = modules::build_tree(path).map_err(|d| d.to_json())?;
        let mut diags = build.diags;
        if generics::is_generic_program(&build.program) {
            if any_error(&diags) {
                return Err(first_error_msg(&diags));
            }
            return monomorphized_program(&build.program);
        }
        diags.extend(check::check_program_real(&build.program));
        if any_error(&diags) {
            return Err(first_error_msg(&diags));
        }
        return Ok(build.program);
    }
    let src = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let real = path.extension().map(|e| e == "cnr").unwrap_or(false);
    if real {
        let program = real::parse_source(&src).map_err(|d| d.to_json())?;
        if generics::is_generic_program(&program) {
            return monomorphized_program(&program);
        }
        let diags = check::check_program_real(&program);
        if any_error(&diags) {
            return Err(first_error_msg(&diags));
        }
        Ok(program)
    } else {
        let program = parse_source(&src).map_err(|d| d.to_json())?;
        let diags = check::check_program(&program);
        if any_error(&diags) {
            return Err(first_error_msg(&diags));
        }
        Ok(program)
    }
}

/// AOT-compile a `.cn`/`.cnr` file or `.cnr` module-tree at `path` into a linked
/// native executable at `out`. Returns `Err` with a message on parse/check errors,
/// out-of-subset MIR, or a backend/link failure.
pub fn compile_path(path: &std::path::Path, out: &std::path::Path) -> Result<(), String> {
    let (mp, items, consts) = lower_path_for_object(path)?;
    backend::object::emit_executable(&mp, &items, &consts, out)
}

/// Emit textual LLVM-IR for `path` (the LLVM-S1 scalar+aggregate backend).
/// Returns `Err` on parse/check errors or an out-of-subset construct.
pub fn emit_llvm_ir(path: &std::path::Path) -> Result<String, String> {
    let (mp, items, consts) = lower_path_for_object(path)?;
    backend::llvm::emit_ll(&mp, &items, &consts)
}

/// LLVM-S0 AOT: emit textual LLVM-IR from `path`'s MIR and build it with
/// `clang -O2`, linked against the same static C runtime as the Cranelift object,
/// into a standalone native executable at `out`. The first OPTIMIZED native code
/// Candor produces.
pub fn compile_path_llvm(path: &std::path::Path, out: &std::path::Path) -> Result<(), String> {
    let ll = emit_llvm_ir(path)?;
    backend::llvm::link_ll(&ll, out)
}

/// AOT-compile `path` into a FREESTANDING (no-libc) native executable at `out` —
/// the NN#6 proof artifact (design 0010 §5; P7/P9). Same MIR/object as
/// `compile_path`; the emitted ELF is `-nostdlib -static -no-pie` and depends on
/// nothing but the kernel (`ldd`: "not a dynamic executable").
pub fn compile_path_freestanding(
    path: &std::path::Path,
    out: &std::path::Path,
) -> Result<(), String> {
    let (mp, items, consts) = lower_path_for_object(path)?;
    backend::object::emit_executable_freestanding(&mp, &items, &consts, out)
}

/// Shared front for both AOT profiles: check `path`, resolve, and lower the whole
/// program to the monomorphized `MirProgram` the object backend consumes.
fn lower_path_for_object(
    path: &std::path::Path,
) -> Result<(mir::MirProgram, resolve::Items, std::collections::HashMap<String, u64>), String> {
    let program = checked_program_for_native(path)?;
    let mut diags = Vec::new();
    let items = resolve::resolve_program(&program, &mut diags);
    let mut consts = std::collections::HashMap::new();
    for it in &program.items {
        if let ast::Item::Static(st) = it {
            if let ast::ExprKind::IntLit { value, .. } = &st.value.kind {
                consts.insert(st.name.clone(), *value);
            }
        }
    }
    let mp = mir::lower_checked(&program, &items)
        .map_err(|e| format!("outside the native backend subset: {}", e.0))?;
    Ok((mp, items, consts))
}
