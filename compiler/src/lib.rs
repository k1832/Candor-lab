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
pub mod dispatch_trace;
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

/// Gate (d) (design 0018 §5.2): the set of interfaces the CHECKER resolved for
/// interface-method calls in `src` — `resolve` in the `dispatch = resolve`
/// invariant (§4.1). Read from the checker's recorded monomorphization shapes
/// (`Shape::Method`), which is the type-checker's own selection, computed
/// independently of the runtime dispatch tables. A dispatch that drifts from
/// resolution (the §2.2 bug — checker resolves `A`, every engine runs `B`) shows
/// up as this set disagreeing with the executed dispatch keys recorded by
/// [`dispatch_trace`], which the five-engine differential gate cannot see.
pub fn resolved_interfaces_real(src: &str) -> Result<std::collections::BTreeSet<String>, Diag> {
    let program = real::parse_source(src)?;
    let (_diags, _insts, shapes) = check::check_generic_program(&program, true);
    Ok(shapes
        .into_values()
        .filter_map(|s| match s {
            generics::Shape::Method(iface) => Some(iface),
            _ => None,
        })
        .collect())
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
// P15 edition migrator (1.0-gate item 1, docs/1.0-GATE-TRIAGE.md row 1): the
// automatic migrator for the REHEARSAL edition transition (2026 ->
// 2027-rehearsal). The rehearsal's only breaking change is a keyword rename
// (`mut` -> `mutable`, `manifest::REHEARSAL_EDITION`), so the migration is a
// mechanical, formatting-preserving token splice plus a manifest edition bump.
// Fully automatic, idempotent, and semantics-preserving — the migrated package
// checks and runs byte-identically to the original (verified across engines).
// ---------------------------------------------------------------------------

/// The outcome of an edition migration ([`migrate_edition_dir`]).
pub struct EditionMigration {
    /// The `.cnr` files whose text was rewritten (empty on an already-migrated,
    /// no-op re-run).
    pub files_rewritten: Vec<std::path::PathBuf>,
    /// Whether the manifest's `edition` field was bumped (false on a no-op re-run).
    pub manifest_bumped: bool,
}

/// Rewrite one 2026 real-syntax source string to the 2027-rehearsal edition:
/// respell every `mut` keyword as `mutable`, preserving all other bytes
/// (comments, whitespace, layout). The rewrite is token-driven — it lexes under
/// the 2026 edition and splices only the byte spans of `mut` *keyword* tokens, so
/// a `mut` inside a string, comment, or a longer identifier is never touched.
///
/// Idempotent: applied to already-migrated text (which spells the keyword
/// `mutable`, an ordinary identifier under the 2026 lexer) it finds no `mut`
/// keyword tokens and returns the input unchanged.
pub fn migrate_edition_source(src: &str) -> Result<String, Diag> {
    use real::token::{RKw, RTok};
    let tokens = real::lexer::lex_in(src, manifest::Edition::E2026)?;
    let mut out = String::with_capacity(src.len());
    let mut last = 0usize;
    for t in &tokens {
        if t.kind == RTok::Kw(RKw::Mut) {
            out.push_str(&src[last..t.span.start]);
            out.push_str("mutable");
            last = t.span.end;
        }
    }
    out.push_str(&src[last..]);
    Ok(out)
}

/// Migrate a whole 2026 package directory to the 2027-rehearsal edition: rewrite
/// every `.cnr` under `src/` ([`migrate_edition_source`]) and bump the manifest's
/// `edition` field. Fully automatic and idempotent — a package already on the
/// rehearsal edition is a no-op (both fields of the returned [`EditionMigration`]
/// report nothing changed).
///
/// Known rehearsal-scope limitation (an honest note on keyword-rename migration):
/// a complete migrator would also alpha-rename any existing identifier spelled
/// `mutable` (it becomes a keyword in the new edition). The rehearsal fixtures use
/// no such identifier, so this migrator does not; a shipped keyword rename would
/// need that additional pass.
pub fn migrate_edition_dir(dir: &std::path::Path) -> Result<EditionMigration, String> {
    let manifest_path = dir.join("candor.toml");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("cannot read `{}`: {e}", manifest_path.display()))?;
    let parsed = manifest::parse_manifest(&manifest_text).map_err(|e| e.to_string())?;

    // Idempotency: a package already on the rehearsal edition needs no work.
    if parsed.package.edition_kind() == manifest::Edition::E2027Rehearsal {
        return Ok(EditionMigration { files_rewritten: Vec::new(), manifest_bumped: false });
    }

    let src_root = dir.join("src");
    let files = modules::discover_module_files(&src_root).map_err(|d| d.to_json())?;
    let mut files_rewritten = Vec::new();
    for (_path, file) in files {
        let src = std::fs::read_to_string(&file)
            .map_err(|e| format!("cannot read `{}`: {e}", file.display()))?;
        let migrated = migrate_edition_source(&src).map_err(|d| d.to_json())?;
        if migrated != src {
            std::fs::write(&file, &migrated)
                .map_err(|e| format!("cannot write `{}`: {e}", file.display()))?;
            files_rewritten.push(file);
        }
    }
    files_rewritten.sort();

    let bumped = bump_manifest_edition(&manifest_text).ok_or_else(|| {
        format!(
            "manifest `{}` has no `edition = \"{}\"` field to migrate",
            manifest_path.display(),
            manifest::CURRENT_EDITION,
        )
    })?;
    std::fs::write(&manifest_path, &bumped)
        .map_err(|e| format!("cannot write `{}`: {e}", manifest_path.display()))?;

    Ok(EditionMigration { files_rewritten, manifest_bumped: true })
}

/// Bump a manifest's `[package] edition` from the stable default to the rehearsal
/// edition, preserving every other byte (comments, layout, key order). Returns
/// `None` when no `edition = "2026"` field is present. Line-oriented rather than a
/// TOML round-trip so the manifest is not reformatted.
fn bump_manifest_edition(text: &str) -> Option<String> {
    let needle = format!("edition = \"{}\"", manifest::CURRENT_EDITION);
    let replacement = format!("edition = \"{}\"", manifest::REHEARSAL_EDITION);
    let mut done = false;
    let out: Vec<String> = text
        .lines()
        .map(|line| {
            if !done && line.trim_start().starts_with("edition") && line.contains(&needle) {
                done = true;
                line.replacen(&needle, &replacement, 1)
            } else {
                line.to_string()
            }
        })
        .collect();
    if !done {
        return None;
    }
    let mut joined = out.join("\n");
    if text.ends_with('\n') {
        joined.push('\n');
    }
    Some(joined)
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
