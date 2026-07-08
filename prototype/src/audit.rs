//! `candor-proto audit <dir_or_file>` — the boundary-module audit surface
//! (design 0011 §6; 0008 §4's `candor audit --boundaries`). A structural query
//! over the boundary modules: their externs (full signature + `foreign` effect +
//! every trust predicate, justification, and span), their exports, and the
//! **effect reach** — which functions *discharge* `foreign` and which *propagate*
//! it. P4-structured: machine-readable JSON from one walk.

use std::path::Path;

use serde::Serialize;

use crate::ast::*;
use crate::diag::Diag;

#[derive(Serialize)]
pub struct AuditReport {
    pub boundary_modules: Vec<ModuleReport>,
    pub effect_reach: Vec<ReachEntry>,
    pub summary: Summary,
}

#[derive(Serialize)]
pub struct ModuleReport {
    pub module: String,
    pub file: String,
    pub abi: Vec<String>,
    pub externs: Vec<ExternReport>,
    pub exports: Vec<ExportReport>,
}

#[derive(Serialize)]
pub struct ExternReport {
    pub name: String,
    pub signature: String,
    pub foreign: bool,
    pub trust: Option<TrustReport>,
}

#[derive(Serialize)]
pub struct TrustReport {
    pub justification: String,
    pub span: SpanRep,
    pub predicates: Vec<PredReport>,
}

#[derive(Serialize)]
pub struct PredReport {
    pub predicate: String,
    pub args: Vec<String>,
    pub span: SpanRep,
}

#[derive(Serialize)]
pub struct ExportReport {
    pub symbol: String,
    pub signature: String,
    pub candor_fn: String,
    pub inbound_fault: String,
}

#[derive(Serialize)]
pub struct ReachEntry {
    pub function: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct Summary {
    pub boundary_modules: usize,
    pub externs: usize,
    pub trust_predicates: usize,
    pub undischarged_foreign_wrappers: usize,
    pub exports: usize,
}

#[derive(Serialize, Clone, Copy)]
pub struct SpanRep {
    pub start: usize,
    pub end: usize,
}

impl From<crate::span::Span> for SpanRep {
    fn from(s: crate::span::Span) -> SpanRep {
        SpanRep { start: s.start, end: s.end }
    }
}

/// Audit a `.cnr` file or a directory of them. Returns pretty JSON (design 0011
/// §6), or a hard parse/IO error.
pub fn audit_path(path: &Path) -> Result<String, Diag> {
    let files = collect_files(path)?;
    let mut modules = Vec::new();
    let mut total_externs = 0usize;
    let mut total_preds = 0usize;
    let mut total_exports = 0usize;

    for (module, file, display) in &files {
        let src = std::fs::read_to_string(file)
            .map_err(|e| Diag::error("E0900", format!("cannot read `{}`: {e}", file.display()), crate::span::Span::point(0)))?;
        let (prog, boundary) = crate::real::parse_with_boundary(&src)?;
        if !boundary {
            continue;
        }
        let mut abi: Vec<String> = Vec::new();
        let mut externs = Vec::new();
        let mut exports = Vec::new();
        for item in &prog.items {
            match item {
                Item::Extern(eb) => {
                    if !abi.contains(&eb.abi) {
                        abi.push(eb.abi.clone());
                    }
                    for ef in &eb.fns {
                        total_externs += 1;
                        let trust = ef.trust.as_ref().map(|t| {
                            total_preds += t.predicates.len();
                            TrustReport {
                                justification: t.justification.clone(),
                                span: t.span.into(),
                                predicates: t
                                    .predicates
                                    .iter()
                                    .map(|p| PredReport {
                                        predicate: p.name.clone(),
                                        args: p.args.clone(),
                                        span: p.span.into(),
                                    })
                                    .collect(),
                            }
                        });
                        externs.push(ExternReport {
                            name: ef.name.clone(),
                            signature: extern_sig_str(ef),
                            foreign: true,
                            trust,
                        });
                    }
                }
                Item::Export(ex) => {
                    total_exports += 1;
                    exports.push(ExportReport {
                        symbol: ex.symbol.clone(),
                        signature: export_sig_str(ex),
                        candor_fn: ex.candor_fn.clone(),
                        inbound_fault: "root policy (abort; no unwinding across C frames)".to_string(),
                    });
                }
                _ => {}
            }
        }
        modules.push(ModuleReport {
            module: module.clone(),
            file: display.clone(),
            abi,
            externs,
            exports,
        });
    }

    // Effect reach: run the checker over the merged/single program.
    let (foreign_report, undischarged) = effect_reach(path)?;
    let boundary_count = modules.len();

    Ok(serde_json::to_string_pretty(&AuditReport {
        boundary_modules: modules,
        effect_reach: foreign_report,
        summary: Summary {
            boundary_modules: boundary_count,
            externs: total_externs,
            trust_predicates: total_preds,
            undischarged_foreign_wrappers: undischarged,
            exports: total_exports,
        },
    })
    .expect("AuditReport serializes"))
}

/// Run the checker over the whole program and return the boundary-module effect
/// reach (functions that discharge or propagate `foreign`) plus the undischarged
/// count. Names are module-qualified for a directory build.
fn effect_reach(path: &Path) -> Result<(Vec<ReachEntry>, usize), Diag> {
    let prog = if path.is_dir() {
        crate::modules::build_tree(path)?.program
    } else {
        let src = std::fs::read_to_string(path)
            .map_err(|e| Diag::error("E0900", format!("cannot read `{}`: {e}", path.display()), crate::span::Span::point(0)))?;
        crate::real::parse_source(&src)?
    };
    let (_diags, report) = crate::check::check_program_real_foreign(&prog);
    let mut out = Vec::new();
    let mut undischarged = 0usize;
    for r in &report {
        if !r.boundary {
            continue;
        }
        if r.discharges {
            out.push(ReachEntry { function: r.name.clone(), status: "discharges foreign".to_string() });
        } else if r.propagates {
            undischarged += 1;
            out.push(ReachEntry { function: r.name.clone(), status: "propagates foreign (undischarged)".to_string() });
        }
    }
    Ok((out, undischarged))
}

/// Discover the `.cnr` files to audit with their module paths (design 0008 §1).
fn collect_files(path: &Path) -> Result<Vec<(String, std::path::PathBuf, String)>, Diag> {
    if path.is_dir() {
        let mut out = Vec::new();
        discover(path, &[], &mut out);
        out.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(out)
    } else {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("module").to_string();
        let display = path.file_name().and_then(|s| s.to_str()).unwrap_or("module.cnr").to_string();
        Ok(vec![(stem, path.to_path_buf(), display)])
    }
}

fn discover(dir: &Path, prefix: &[String], out: &mut Vec<(String, std::path::PathBuf, String)>) {
    let mut entries: Vec<std::path::PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok().map(|e| e.path())).collect(),
        Err(_) => return,
    };
    entries.sort();
    for p in entries {
        if p.is_dir() {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                let mut sub = prefix.to_vec();
                sub.push(name.to_string());
                discover(&p, &sub, out);
            }
        } else if p.extension().and_then(|s| s.to_str()) == Some("cnr") {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                let mut mp = prefix.to_vec();
                mp.push(stem.to_string());
                let display = format!("{}.cnr", mp.join("/"));
                out.push((mp.join("::"), p, display));
            }
        }
    }
}

// ----- signature rendering (design 0011 §6 output form) --------------------

fn extern_sig_str(ef: &ExternFn) -> String {
    let ps: Vec<String> = ef.params.iter().map(|p| ty_c(&p.ty)).collect();
    let ret = ef.ret.as_ref().map(|r| ty_c(&r.ty)).unwrap_or_else(|| "unit".to_string());
    format!("fn {}({}) foreign -> {}", ef.name, ps.join(", "), ret)
}

fn export_sig_str(ex: &ExportDecl) -> String {
    let ps: Vec<String> = ex.params.iter().map(|p| ty_c(&p.ty)).collect();
    let ret = ex.ret.as_ref().map(|r| ty_c(&r.ty)).unwrap_or_else(|| "unit".to_string());
    format!("fn {}({}) -> {}", ex.symbol, ps.join(", "), ret)
}

/// Render a boundary type in C-facing form for the audit (design 0011 §6).
fn ty_c(ty: &Ty) -> String {
    use crate::ast::TyKind::*;
    match &ty.kind {
        Scalar(s) => format!("{s:?}").to_lowercase(),
        Named(n) => n.clone(),
        RawPtr(e) => format!("rawptr {}", ty_c(e)),
        Array { elem, .. } => format!("[N]{}", ty_c(elem)),
        _ => "?".to_string(),
    }
}
