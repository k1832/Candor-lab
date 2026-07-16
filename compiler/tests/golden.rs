//! Golden tests: every design 0001 §11 worked example must parse successfully.
//! Parsing only — undefined names are fine (no symbol table, NN#13).

use candor::parse_source;

fn assert_parses(name: &str) {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    match parse_source(&src) {
        Ok(program) => assert!(
            !program.items.is_empty(),
            "{name} parsed but produced no items"
        ),
        Err(diag) => panic!("{name} failed to parse: {}", diag.to_json()),
    }
}

#[test]
fn ex_11_1_allocator_parses() {
    assert_parses("11_1_allocator.cn");
}

#[test]
fn ex_11_2_scheduler_parses() {
    assert_parses("11_2_scheduler.cn");
}

#[test]
fn ex_11_3_mmio_parses() {
    assert_parses("11_3_mmio.cn");
}

#[test]
fn ex_11_4_parser_parses() {
    assert_parses("11_4_parser.cn");
}

#[test]
fn ex_11_5_arena_parses() {
    assert_parses("11_5_arena.cn");
}

#[test]
fn item_counts_match_expectations() {
    // Sanity: the allocator fixture has 3 structs + 3 fns.
    let path = format!(
        "{}/tests/fixtures/11_1_allocator.cn",
        env!("CARGO_MANIFEST_DIR")
    );
    let src = std::fs::read_to_string(path).unwrap();
    let prog = parse_source(&src).unwrap();
    assert_eq!(prog.items.len(), 6);
}
