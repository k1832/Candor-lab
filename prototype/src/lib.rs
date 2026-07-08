//! Candor Bet 5 validation prototype — Stage 1 (lexer, AST, parser).
//!
//! Structured as a library crate so later stages (checker, interpreter) add
//! modules alongside these without reshaping the front end. The `candor-proto`
//! binary is a thin CLI over `parse_source`.

pub mod ast;
pub mod audit;
pub mod build;
pub mod check;
pub mod count;
pub mod diag;
pub mod foreign;
pub mod generics;
pub mod interp;
pub mod lexer;
pub mod mir;
pub mod modules;
pub mod parser;
pub mod real;
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
fn lower_and_run_native(program: &ast::Program) -> MirRunResult {
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
        Ok(mp) => match backend::run(&mp, &items, &consts) {
            Ok(run) => MirRunResult::Ok(run),
            Err(f) => MirRunResult::Fault(f),
        },
        Err(e) => MirRunResult::Unsupported(e.0),
    }
}

fn run_generic_native(program: &ast::Program) -> MirRunResult {
    let (diags, insts, shapes) = check::check_generic_program(program, true);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    let mono = generics::monomorphize(program, &insts, &shapes);
    if mono.diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(mono.diags);
    }
    lower_and_run_native(&mono.program)
}

/// Throwaway (`.cn`) source through the native engine.
pub fn run_source_native(src: &str) -> MirRunResult {
    let program = match parse_source(src) {
        Ok(p) => p,
        Err(d) => return MirRunResult::ParseError(d),
    };
    let diags = check::check_program(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run_native(&program)
}

/// Real (`.cnr`) source through the native engine (monomorphizing generics first).
pub fn run_source_real_native(src: &str) -> MirRunResult {
    let program = match real::parse_source(src) {
        Ok(p) => p,
        Err(d) => return MirRunResult::ParseError(d),
    };
    if generics::is_generic_program(&program) {
        return run_generic_native(&program);
    }
    let diags = check::check_program_real(&program);
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run_native(&program)
}

/// Module-tree (`.cnr` directory) through the native engine.
pub fn run_dir_native(dir: &std::path::Path) -> MirRunResult {
    let build = match modules::build_tree(dir) {
        Ok(b) => b,
        Err(d) => return MirRunResult::ParseError(d),
    };
    let mut diags = build.diags;
    if generics::is_generic_program(&build.program) {
        if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
            return MirRunResult::CheckErrors(diags);
        }
        return run_generic_native(&build.program);
    }
    diags.extend(check::check_program_real(&build.program));
    if diags.iter().any(|d| matches!(d.severity, diag::Severity::Error)) {
        return MirRunResult::CheckErrors(diags);
    }
    lower_and_run_native(&build.program)
}
