//! The P20 reference-project generator (design 0010 §3).
//!
//! Emits a deterministic, self-contained Candor module tree at the reference
//! scale the ratified P20 targets are defined against (N ≈ 200 modules,
//! M ≈ 50 000 lines), plus a parallel C translation-unit set of comparable scale
//! for the `--release ≤ 2× C baseline` clause. Both trees and a manifest are
//! COMMITTED under `benches/p20-reference/` (the frozen-instrument discipline:
//! the measurement subject is reviewable and frozen, like the corelib basket) and
//! regenerated only by re-running this binary. Deterministic across runs: a fixed
//! seed drives a splitmix64 PRNG, so the output is byte-identical every run.
//!
//! Run: `cargo run --release --bin p20-gen` (writes into `benches/p20-reference/`).
//!
//! The generated Candor is idiomatic, checkable, NON-`alloc` (pure scalar core:
//! structs/enums/generics-with-bounds/interfaces+impls/match/`read`-`write`
//! borrows/contracts — the checker surface design 0010 §3 names, minus the heap,
//! so the 50 kL subject is robustly clean rather than a fragile allocator maze).
//! The DAG is layered with cross-layer fan-in/fan-out (not a chain).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Fixed seed ("P20REFER" as bytes) — determinism anchor.
const SEED: u64 = 0x5032_3052_4546_4552;
const NUM_MODULES: usize = 200;
/// Per-layer module widths (sum = NUM_MODULES): a diamond profile — fan-out to a
/// wide middle, then fan-in — so the import DAG is genuinely layered.
const LAYERS: &[usize] = &[8, 16, 24, 30, 34, 30, 24, 18, 12, 4];

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // splitmix64
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Inclusive range [lo, hi].
    fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + (self.next() as usize) % (hi - lo + 1)
    }
    fn chance(&mut self, num: usize, den: usize) -> bool {
        (self.next() as usize) % den < num
    }
}

/// The numeric scalar field types used in generated structs (all `copy`, so every
/// struct is drop-inert and the tree stays off the `alloc` floor).
const NUM_TYS: &[(&str, &str)] = &[("i64", "0"), ("u32", "0u32"), ("usize", "0usize")];

fn layer_of(id: usize) -> usize {
    let mut acc = 0;
    for (l, w) in LAYERS.iter().enumerate() {
        acc += w;
        if id < acc {
            return l;
        }
    }
    LAYERS.len() - 1
}

fn mod_path(id: usize) -> String {
    format!("l{}::m{id:03}", layer_of(id))
}
fn mod_file(id: usize) -> String {
    format!("l{}/m{id:03}.cnr", layer_of(id))
}

fn gen_group(s: &mut String, id: usize, g: usize, rng: &mut Rng) {
    // --- struct with `a: i64`, seeded numeric extras, a `flag: bool` ----------
    let extras = rng.range(1, 3);
    let mut fields: Vec<(String, &str, &str)> = vec![("a".to_string(), "i64", "0")];
    for e in 0..extras {
        let (ty, zero) = NUM_TYS[rng.range(0, NUM_TYS.len() - 1)];
        fields.push((format!("n{e}"), ty, zero));
    }
    writeln!(s, "pub struct S{id:03}_{g} {{").unwrap();
    for (name, ty, _) in &fields {
        writeln!(s, "    {name}: {ty},").unwrap();
    }
    s.push_str("    flag: bool,\n}\n");

    // constructor: seed the `a` field, zero the rest.
    writeln!(s, "pub fn mk{id:03}_{g}(a: i64) -> S{id:03}_{g} {{").unwrap();
    write!(s, "    return S{id:03}_{g} {{ a: a").unwrap();
    for (name, _, zero) in fields.iter().skip(1) {
        write!(s, ", {name}: {zero}").unwrap();
    }
    s.push_str(", flag: true };\n}\n");

    // interface impl: Probe sums the numeric fields (bool excluded).
    writeln!(s, "impl Probe for S{id:03}_{g} {{").unwrap();
    s.push_str("    fn probe(read self) -> i64 {\n");
    write!(s, "        return self.a").unwrap();
    for (name, ty, _) in fields.iter().skip(1) {
        if *ty == "i64" {
            write!(s, " + self.{name}").unwrap();
        } else {
            write!(s, " + conv i64 self.{name}").unwrap();
        }
    }
    s.push_str(";\n    }\n}\n");

    // a read-borrow reader and a write-borrow mutator.
    let k = rng.range(2, 7) as i64;
    writeln!(
        s,
        "pub fn read{id:03}_{g}(s: read S{id:03}_{g}) -> i64 {{\n    return s.a * {k}i64;\n}}"
    )
    .unwrap();
    writeln!(
        s,
        "pub fn bump{id:03}_{g}(s: write S{id:03}_{g}, d: i64) -> unit {{\n    s.*.a = s.a + d;\n}}"
    )
    .unwrap();

    // --- enum + exhaustive, all-arms-return classifier ------------------------
    let variants = rng.range(2, 4);
    writeln!(s, "pub enum E{id:03}_{g} {{").unwrap();
    s.push_str("    V0,\n    V1(i64),\n");
    if variants >= 3 {
        s.push_str("    V2(i64, i64),\n");
    }
    if variants >= 4 {
        s.push_str("    V3(i64),\n");
    }
    s.push_str("}\n");
    writeln!(s, "pub fn classify{id:03}_{g}(e: E{id:03}_{g}) -> i64 {{").unwrap();
    s.push_str("    match e {\n");
    s.push_str("        E{ID}_{G}::V0 => {\n            return 0i64;\n        },\n".replace("{ID}", &format!("{id:03}")).replace("{G}", &format!("{g}")).as_str());
    s.push_str("        E{ID}_{G}::V1(a) => {\n            return a;\n        },\n".replace("{ID}", &format!("{id:03}")).replace("{G}", &format!("{g}")).as_str());
    if variants >= 3 {
        s.push_str("        E{ID}_{G}::V2(a, b) => {\n            return a + b;\n        },\n".replace("{ID}", &format!("{id:03}")).replace("{G}", &format!("{g}")).as_str());
    }
    if variants >= 4 {
        s.push_str("        E{ID}_{G}::V3(a) => {\n            return a * 2i64;\n        },\n".replace("{ID}", &format!("{id:03}")).replace("{G}", &format!("{g}")).as_str());
    }
    s.push_str("    }\n}\n");

    // --- generic fn with an interface bound -----------------------------------
    // Definition-only: `pick` is CHECKED (every def-site body is type-checked,
    // bounds and all) but NOT called from `f<id>`, so it never reaches codegen.
    // Reaching ~700 distinct `pick<i64>` monomorphizations would abort `build`/
    // `--release` with E1099: the monomorphization backstop (src/generics.rs
    // `drive`) increments its counter PER work-item and never resets, so it caps
    // TOTAL instantiations at 64 (spec 10.4 intends a depth/chain bound). Codegen
    // therefore monomorphizes only the shared `maxof`/`minof`; the full generic
    // surface is exercised by `check`, which does not monomorphize.
    let use_min = rng.chance(1, 2);
    let combiner = if use_min { "minof" } else { "maxof" };
    writeln!(
        s,
        "pub fn pick{id:03}_{g}[T: Ord2 + copy](a: T, b: T) -> T {{\n    return {combiner}(a, b);\n}}"
    )
    .unwrap();

    // --- a while-loop accumulator ---------------------------------------------
    let step = rng.range(2, 5) as i64;
    writeln!(s, "pub fn accum{id:03}_{g}(n: i64) -> i64 {{").unwrap();
    s.push_str("    let mut acc: i64 = 0i64;\n    let mut i: i64 = 0i64;\n");
    s.push_str("    while i < n {\n");
    writeln!(s, "        acc = acc + i * {step}i64;").unwrap();
    s.push_str("        i = i + 1i64;\n    }\n    return acc;\n}\n");

    // --- a contract-carrying function -----------------------------------------
    let bias = rng.range(1, 9) as i64;
    writeln!(
        s,
        "pub fn guard{id:03}_{g}(x: i64) requires(x >= 0i64) ensures(result >= x) -> i64 {{\n    return x + {bias}i64;\n}}"
    )
    .unwrap();
}

fn gen_base() -> String {
    let mut s = String::new();
    s.push_str(
        "// The P20 reference project's prelude: the shared interfaces + scalar\n\
         // ordering the generated module tree builds on. Non-`alloc`, drop-inert.\n\n",
    );
    s.push_str("pub enum Cmp {\n    Lt,\n    Eq,\n    Gt,\n}\n\n");
    s.push_str("pub interface Probe {\n    fn probe(read self) -> i64;\n}\n\n");
    s.push_str("pub interface Ord2 {\n    fn compare(read self, other: Self) -> Cmp;\n}\n\n");
    for ty in ["i64", "u32", "usize"] {
        writeln!(s, "impl Ord2 for {ty} {{").unwrap();
        s.push_str("    fn compare(read self, other: Self) -> Cmp {\n");
        s.push_str("        if self.* < other {\n            return Cmp::Lt;\n        }\n");
        s.push_str("        if self.* > other {\n            return Cmp::Gt;\n        }\n");
        s.push_str("        return Cmp::Eq;\n    }\n}\n\n");
    }
    s.push_str(
        "pub fn maxof[T: Ord2 + copy](a: T, b: T) -> T {\n    match a.compare(b) {\n\
         \x20       Cmp::Lt => {\n            return b;\n        },\n\
         \x20       Cmp::Eq => {\n            return a;\n        },\n\
         \x20       Cmp::Gt => {\n            return a;\n        },\n    }\n}\n\n",
    );
    s.push_str(
        "pub fn minof[T: Ord2 + copy](a: T, b: T) -> T {\n    match a.compare(b) {\n\
         \x20       Cmp::Lt => {\n            return a;\n        },\n\
         \x20       Cmp::Eq => {\n            return a;\n        },\n\
         \x20       Cmp::Gt => {\n            return b;\n        },\n    }\n}\n\n",
    );
    s.push_str(
        "pub fn clampi(x: i64) -> i64 {\n    if x > 1000000i64 {\n        return 1000000i64;\n    }\n\
         \x20   if x < 0i64 - 1000000i64 {\n        return 0i64 - 1000000i64;\n    }\n    return x;\n}\n",
    );
    s
}

fn gen_main(roots: &[usize]) -> String {
    let mut s = String::new();
    s.push_str("// The P20 reference project entry. Drives a spread of the module DAG's\n");
    s.push_str("// canonical functions and returns a sentinel.\n");
    for &r in roots {
        writeln!(s, "use {}::{{f{r:03}}};", mod_path(r)).unwrap();
    }
    s.push_str("\nfn main() -> i64 {\n    let mut acc: i64 = 0i64;\n");
    for (k, &r) in roots.iter().enumerate() {
        writeln!(s, "    acc = acc + f{r:03}({}i64);", (k as i64 % 7) + 1).unwrap();
    }
    s.push_str("    return acc;\n}\n");
    s
}

// --------------------------------------------------------------------------
// Parallel C tree (the `cc -O2` baseline for the `--release ≤ 2× C` clause).
// Structurally comparable: one TU per Candor module, equivalent structs, a
// tagged-union switch classifier, a while-accumulator, and the same f<id> call
// graph across TUs — an honest same-shape, same-scale -O2 workload.
// --------------------------------------------------------------------------

fn gen_c_module(id: usize, deps: &[usize], groups: usize, rng: &mut Rng) -> String {
    let mut s = String::new();
    writeln!(s, "/* GENERATED C mirror of reference module m{id:03}. */").unwrap();
    s.push_str("#include \"proto.h\"\n\n");
    for g in 0..groups {
        // Mirror the Candor group's shape (struct + numeric extras + flag) and its
        // function set (constructor / probe / reader / mutator / classifier /
        // accumulator / contract-guard) so the C TU set is a same-shape workload.
        let extras = rng.range(1, 3);
        writeln!(s, "typedef struct {{").unwrap();
        s.push_str("    long a;\n");
        for e in 0..extras {
            writeln!(s, "    long n{e};").unwrap();
        }
        writeln!(s, "    int flag;\n}} S{id}_{g};\n").unwrap();

        writeln!(s, "static S{id}_{g} mk{id}_{g}(long a) {{").unwrap();
        writeln!(s, "    S{id}_{g} s;").unwrap();
        s.push_str("    s.a = a;\n");
        for e in 0..extras {
            writeln!(s, "    s.n{e} = 0;").unwrap();
        }
        s.push_str("    s.flag = 1;\n    return s;\n}\n");

        writeln!(s, "static long probe{id}_{g}(const S{id}_{g} *s) {{").unwrap();
        write!(s, "    return s->a").unwrap();
        for e in 0..extras {
            write!(s, " + s->n{e}").unwrap();
        }
        s.push_str(";\n}\n");

        let k = rng.range(2, 7) as i64;
        writeln!(s, "static long read{id}_{g}(const S{id}_{g} *s) {{").unwrap();
        writeln!(s, "    return s->a * {k};\n}}").unwrap();

        writeln!(s, "static void bump{id}_{g}(S{id}_{g} *s, long d) {{").unwrap();
        s.push_str("    s->a = s->a + d;\n}\n");

        writeln!(s, "static long classify{id}_{g}(int tag, long a, long b) {{").unwrap();
        s.push_str("    switch (tag) {\n    case 0:\n        return 0;\n\
                     \x20   case 1:\n        return a;\n    case 2:\n        return a + b;\n\
                     \x20   default:\n        return a * 2;\n    }\n}\n");

        let step = rng.range(2, 5) as i64;
        writeln!(s, "static long accum{id}_{g}(long n) {{").unwrap();
        s.push_str("    long acc = 0;\n    long i = 0;\n");
        writeln!(s, "    for (i = 0; i < n; i++) {{").unwrap();
        writeln!(s, "        acc += i * {step};").unwrap();
        s.push_str("    }\n    return acc;\n}\n");

        let bias = rng.range(1, 9) as i64;
        writeln!(s, "static long guard{id}_{g}(long x) {{").unwrap();
        writeln!(s, "    return x + {bias};\n}}\n").unwrap();
    }
    writeln!(s, "long f{id:03}(long x) {{").unwrap();
    s.push_str("    long acc = x;\n");
    for (k, &d) in deps.iter().enumerate() {
        writeln!(s, "    acc += f{d:03}(x + {});", (k as i64) + 1).unwrap();
    }
    for g in 0..groups {
        writeln!(s, "    S{id}_{g} s{g} = mk{id}_{g}(acc);").unwrap();
        writeln!(s, "    bump{id}_{g}(&s{g}, {});", rng.range(1, 9)).unwrap();
        writeln!(s, "    acc += probe{id}_{g}(&s{g});").unwrap();
        writeln!(s, "    acc += read{id}_{g}(&s{g});").unwrap();
        writeln!(s, "    acc += classify{id}_{g}(1, acc, acc);").unwrap();
        writeln!(s, "    acc += accum{id}_{g}({});", rng.range(3, 9)).unwrap();
        writeln!(s, "    acc += guard{id}_{g}(acc);").unwrap();
    }
    s.push_str("    return clampi(acc);\n}\n");
    s
}

fn gen_c_base() -> String {
    "#include \"proto.h\"\n\n\
     long clampi(long x) {\n    if (x > 1000000) return 1000000;\n\
     \x20   if (x < -1000000) return -1000000;\n    return x;\n}\n"
        .to_string()
}

fn gen_c_proto() -> String {
    let mut s = String::new();
    s.push_str("#ifndef P20_PROTO_H\n#define P20_PROTO_H\nlong clampi(long x);\n");
    for id in 0..NUM_MODULES {
        writeln!(s, "long f{id:03}(long);").unwrap();
    }
    s.push_str("#endif\n");
    s
}

fn gen_c_main(roots: &[usize]) -> String {
    let mut s = String::new();
    s.push_str("#include \"proto.h\"\n\nint main(void) {\n    long acc = 0;\n");
    for (k, &r) in roots.iter().enumerate() {
        writeln!(s, "    acc += f{r:03}({});", (k as i64 % 7) + 1).unwrap();
    }
    s.push_str("    return (int)(acc & 0xff);\n}\n");
    s
}

fn write_file(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn count_lines(src: &str) -> usize {
    src.lines().count()
}

fn main() {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/p20-reference");
    let candor_dir = root.join("candor");
    let c_dir = root.join("c");
    let _ = std::fs::remove_dir_all(&candor_dir);
    let _ = std::fs::remove_dir_all(&c_dir);

    let mut rng = Rng(SEED);

    // 1. Assign dependency edges (each module draws deps from strictly-lower layers).
    let mut deps_of: Vec<Vec<usize>> = vec![Vec::new(); NUM_MODULES];
    let mut dependents: Vec<usize> = vec![0; NUM_MODULES];
    let mut group_count: Vec<usize> = vec![0; NUM_MODULES];
    // Precompute per-module group counts from a dedicated RNG stream so the C and
    // Candor trees agree on structure without interleaving their draws.
    for gc in group_count.iter_mut() {
        *gc = rng.range(3, 5);
    }
    for (id, slot) in deps_of.iter_mut().enumerate() {
        let layer = layer_of(id);
        if layer == 0 {
            continue;
        }
        let lower: Vec<usize> = (0..id).filter(|&j| layer_of(j) < layer).collect();
        let want = rng.range(1, 4).min(lower.len());
        let mut chosen: Vec<usize> = Vec::new();
        let mut guard = 0;
        while chosen.len() < want && guard < 64 {
            let cand = lower[rng.range(0, lower.len() - 1)];
            if !chosen.contains(&cand) {
                chosen.push(cand);
                dependents[cand] += 1;
            }
            guard += 1;
        }
        chosen.sort_unstable();
        *slot = chosen;
    }

    // 2. Generate the Candor tree. Group counts are fixed; the group-body RNG is a
    //    fresh deterministic stream so module bodies don't depend on dep draws.
    let mut body_rng = Rng(SEED ^ 0xA5A5_A5A5_A5A5_A5A5);
    let mut candor_lines = 0usize;
    for id in 0..NUM_MODULES {
        // consume exactly the group count decided above by regenerating with it
        let src = gen_candor_module_fixed(id, &deps_of[id], group_count[id], &mut body_rng);
        let lines = count_lines(&src);
        candor_lines += lines;
        write_file(&candor_dir.join(mod_file(id)), &src);
    }
    let base_src = gen_base();
    candor_lines += count_lines(&base_src);
    write_file(&candor_dir.join("base/prelude.cnr"), &base_src);

    // main drives every top-layer module plus a scattered sample (real fan-in).
    let top_start: usize = LAYERS[..LAYERS.len() - 1].iter().sum();
    let mut roots: Vec<usize> = (top_start..NUM_MODULES).collect();
    let mut r = top_start;
    while r > 0 {
        r = r.saturating_sub(17);
        roots.push(r);
    }
    roots.sort_unstable();
    roots.dedup();
    let main_src = gen_main(&roots);
    candor_lines += count_lines(&main_src);
    write_file(&candor_dir.join("main.cnr"), &main_src);

    // 3. Generate the parallel C tree with a matching structural RNG stream.
    let mut c_rng = Rng(SEED ^ 0xC0C0_C0C0_C0C0_C0C0);
    let mut c_lines = 0usize;
    for id in 0..NUM_MODULES {
        let src = gen_c_module(id, &deps_of[id], group_count[id], &mut c_rng);
        c_lines += count_lines(&src);
        write_file(&c_dir.join(format!("m{id:03}.c")), &src);
    }
    let cbase = gen_c_base();
    c_lines += count_lines(&cbase);
    write_file(&c_dir.join("base.c"), &cbase);
    let cproto = gen_c_proto();
    c_lines += count_lines(&cproto);
    write_file(&c_dir.join("proto.h"), &cproto);
    let cmain = gen_c_main(&roots);
    c_lines += count_lines(&cmain);
    write_file(&c_dir.join("main.c"), &cmain);

    // 4. The edit target for the T1/T2 measurement: the low-layer module with the
    //    most dependents — a body edit there is the strongest zero-downstream test.
    let edit_target = (0..NUM_MODULES).max_by_key(|&i| (dependents[i], usize::MAX - i)).unwrap();
    let edit_path = mod_file(edit_target);

    let manifest = format!(
        "{{\n  \"generator\": \"p20_gen.rs\",\n  \"seed\": \"{SEED:#018x}\",\n\
         \x20 \"candor_modules\": {},\n  \"candor_lines\": {candor_lines},\n\
         \x20 \"c_translation_units\": {},\n  \"c_lines\": {c_lines},\n\
         \x20 \"edit_target\": \"{edit_path}\",\n  \"edit_target_dependents\": {},\n\
         \x20 \"main_roots\": {}\n}}\n",
        NUM_MODULES + 2,
        NUM_MODULES + 2,
        dependents[edit_target],
        roots.len(),
    );
    write_file(&root.join("p20-manifest.json"), &manifest);

    eprintln!(
        "generated p20-reference: {} candor modules ({} lines), {} C TUs ({} lines); edit target {edit_path} ({} dependents)",
        NUM_MODULES + 2,
        candor_lines,
        NUM_MODULES + 2,
        c_lines,
        dependents[edit_target],
    );
}

/// As `gen_candor_module` but with a caller-fixed group count (so the Candor and C
/// trees share module shapes deterministically).
fn gen_candor_module_fixed(id: usize, deps: &[usize], groups: usize, rng: &mut Rng) -> String {
    let mut s = String::new();
    let layer = layer_of(id);
    writeln!(
        s,
        "// GENERATED reference module m{id:03} (layer {layer}). Part of the P20\n\
         // measurement subject (design 0010 §3). DO NOT EDIT BY HAND — regenerate\n\
         // via `cargo run --release --bin p20-gen`. Non-`alloc` scalar core."
    )
    .unwrap();
    s.push_str("use base::prelude::{Probe, Ord2, maxof, minof, clampi, Cmp};\n");
    for &d in deps {
        writeln!(s, "use {}::{{f{d:03}}};", mod_path(d)).unwrap();
    }
    s.push('\n');
    for g in 0..groups {
        gen_group(&mut s, id, g, rng);
    }
    writeln!(s, "pub fn f{id:03}(x: i64) -> i64 {{").unwrap();
    s.push_str("    let mut acc: i64 = x;\n");
    for (k, &d) in deps.iter().enumerate() {
        writeln!(s, "    acc = acc + f{d:03}(x + {}i64);", (k as i64) + 1).unwrap();
    }
    for g in 0..groups {
        writeln!(s, "    let mut s{g}: S{id:03}_{g} = mk{id:03}_{g}(acc);").unwrap();
        writeln!(s, "    bump{id:03}_{g}(write s{g}, {}i64);", rng.range(1, 9)).unwrap();
        writeln!(s, "    acc = acc + s{g}.probe();").unwrap();
        writeln!(s, "    acc = acc + read{id:03}_{g}(read s{g});").unwrap();
        writeln!(s, "    acc = acc + classify{id:03}_{g}(E{id:03}_{g}::V1(acc));").unwrap();
        writeln!(s, "    acc = acc + accum{id:03}_{g}({}i64);", rng.range(3, 9)).unwrap();
    }
    s.push_str("    acc = maxof(acc, x);\n");
    s.push_str("    acc = minof(acc, x + 1i64);\n");
    s.push_str("    return clampi(acc);\n");
    s.push_str("}\n");
    s
}
