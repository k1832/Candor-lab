//! `candor.toml` manifest parser + closed-schema validation (design 0017 §2/§3/§4).

use candor_proto::manifest::{parse_manifest, Source, Version};

const FULL: &str = r#"
[package]
name = "http"
version = "0.3.1"
edition = "2026"
freestanding = true

[lib]

[[bin]]
name = "httpc"

[[bin]]
name = "httpd"

[dependencies]
json = { path = "../json" }
tls  = { git = "https://example.com/tls.git", rev = "a1b2c3d4", tag = "v1.0" }
codec = { path = "../serde-codec", package = "serde_codec" }
"#;

#[test]
fn parses_a_full_valid_manifest() {
    let m = parse_manifest(FULL).expect("valid manifest parses");

    assert_eq!(m.package.name, "http");
    assert_eq!(m.package.version, Version { major: 0, minor: 3, patch: 1 });
    assert_eq!(m.package.edition, "2026");
    assert!(m.package.freestanding);

    assert!(m.lib.is_some(), "[lib] present ⇒ library");

    let bins: Vec<&str> = m.bins.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(bins, vec!["httpc", "httpd"]);

    // Dependencies are keyed by local name, sorted for determinism.
    let names: Vec<&str> = m.dependencies.iter().map(|d| d.local_name.as_str()).collect();
    assert_eq!(names, vec!["codec", "json", "tls"]);

    let json = m.dependencies.iter().find(|d| d.local_name == "json").unwrap();
    assert_eq!(json.package, None);
    assert_eq!(json.source, Source::Path { path: "../json".into() });

    let tls = m.dependencies.iter().find(|d| d.local_name == "tls").unwrap();
    assert_eq!(
        tls.source,
        Source::Git {
            url: "https://example.com/tls.git".into(),
            rev: "a1b2c3d4".into(),
            tag: Some("v1.0".into()),
            branch: None,
        }
    );

    // The alias form (0017 §5): local name `codec`, underlying package `serde_codec`.
    let codec = m.dependencies.iter().find(|d| d.local_name == "codec").unwrap();
    assert_eq!(codec.package.as_deref(), Some("serde_codec"));
    assert_eq!(codec.source, Source::Path { path: "../serde-codec".into() });
}

#[test]
fn minimal_manifest_no_lib_no_bin_no_deps() {
    let m = parse_manifest("[package]\nname = \"core\"\nversion = \"1.2.3\"\nedition = \"2026\"\n")
        .expect("minimal manifest parses");
    assert!(!m.package.freestanding, "freestanding defaults to false");
    assert!(m.lib.is_none());
    assert!(m.bins.is_empty());
    assert!(m.dependencies.is_empty());
}

// --- Closed schema: unknown keys are errors, naming the key ------------------

#[test]
fn rejects_unknown_package_key() {
    let e = parse_manifest("[package]\nname=\"a\"\nversion=\"1.0.0\"\nedition=\"2026\"\nauthors=\"x\"\n")
        .unwrap_err();
    assert_eq!(e.code, "M0002");
    assert!(e.message.contains("authors"), "names the offending key: {}", e.message);
}

#[test]
fn rejects_unknown_top_level_table() {
    let e = parse_manifest("[package]\nname=\"a\"\nversion=\"1.0.0\"\nedition=\"2026\"\n[profile]\nx=1\n")
        .unwrap_err();
    assert_eq!(e.code, "M0002");
    assert!(e.message.contains("profile"), "names the offending table: {}", e.message);
}

#[test]
fn rejects_unknown_bin_key() {
    let e = parse_manifest("[package]\nname=\"a\"\nversion=\"1.0.0\"\nedition=\"2026\"\n[[bin]]\nname=\"a\"\npath=\"x\"\n")
        .unwrap_err();
    assert_eq!(e.code, "M0002");
    assert!(e.message.contains("path"), "names the offending key: {}", e.message);
}

// --- Version --------------------------------------------------------------

#[test]
fn rejects_two_component_version() {
    let e = parse_manifest("[package]\nname=\"a\"\nversion=\"1.0\"\nedition=\"2026\"\n").unwrap_err();
    assert_eq!(e.code, "M0101");
    assert!(e.message.contains("1.0"));
}

#[test]
fn rejects_non_numeric_version() {
    let e = parse_manifest("[package]\nname=\"a\"\nversion=\"x.y.z\"\nedition=\"2026\"\n").unwrap_err();
    assert_eq!(e.code, "M0101");
}

// --- Edition --------------------------------------------------------------

#[test]
fn rejects_unknown_edition() {
    let e = parse_manifest("[package]\nname=\"a\"\nversion=\"1.0.0\"\nedition=\"1999\"\n").unwrap_err();
    assert_eq!(e.code, "M0102");
    assert!(e.message.contains("1999") && e.message.contains("2026"), "{}", e.message);
}

// --- Name ------------------------------------------------------------------

#[test]
fn rejects_ill_formed_name() {
    for bad in ["", "Http", "1json", "-x", "a b"] {
        let src = format!("[package]\nname=\"{bad}\"\nversion=\"1.0.0\"\nedition=\"2026\"\n");
        let e = parse_manifest(&src).unwrap_err();
        assert_eq!(e.code, "M0100", "name `{bad}` should be rejected");
    }
}

// --- Missing required fields ----------------------------------------------

#[test]
fn rejects_missing_required_fields() {
    for (missing, src) in [
        ("name", "[package]\nversion=\"1.0.0\"\nedition=\"2026\"\n"),
        ("version", "[package]\nname=\"a\"\nedition=\"2026\"\n"),
        ("edition", "[package]\nname=\"a\"\nversion=\"1.0.0\"\n"),
        ("package", ""),
    ] {
        let e = parse_manifest(src).unwrap_err();
        assert_eq!(e.code, "M0002", "missing {missing}");
        assert!(e.message.contains(missing), "names the missing field {missing}: {}", e.message);
    }
}

// --- Dependency sources ----------------------------------------------------

fn dep_err(dep: &str) -> candor_proto::manifest::ManifestError {
    let src = format!("[package]\nname=\"a\"\nversion=\"1.0.0\"\nedition=\"2026\"\n[dependencies]\nd = {dep}\n");
    parse_manifest(&src).unwrap_err()
}

#[test]
fn rejects_git_without_rev() {
    let e = dep_err("{ git = \"https://x/y.git\" }");
    assert_eq!(e.code, "M0202");
    assert!(e.message.contains("rev"), "{}", e.message);
}

#[test]
fn rejects_empty_path() {
    let e = dep_err("{ path = \"\" }");
    assert_eq!(e.code, "M0202");
    assert!(e.message.contains("path"), "{}", e.message);
}

#[test]
fn rejects_unexpected_key_within_a_known_source() {
    let e = dep_err("{ path = \"../x\", rev = \"abc\" }");
    assert_eq!(e.code, "M0202");
    assert!(e.message.contains("rev"), "{}", e.message);
}

#[test]
fn rejects_ill_formed_alias() {
    let e = dep_err("{ path = \"../x\", package = \"Bad Name\" }");
    assert_eq!(e.code, "M0203");
}

#[test]
fn rejects_non_table_dependency() {
    let e = dep_err("\"1.0\"");
    assert_eq!(e.code, "M0200");
}

// Forward-compatibility: a future registry source shape must be rejected today
// with a clear "unknown source kind" — proving the closed schema catches it
// rather than silently accepting garbage, while the schema stays extensible.
#[test]
fn rejects_future_registry_source_kind_clearly() {
    let e = dep_err("{ registry = \"https://pkg.candor\", version = \"^1.2\" }");
    assert_eq!(e.code, "M0201");
    assert!(e.message.contains("unknown source kind"), "{}", e.message);
    assert!(e.message.contains("registry"), "names what it found: {}", e.message);
}

#[test]
fn rejects_empty_source_table() {
    let e = dep_err("{}");
    assert_eq!(e.code, "M0201");
}

#[test]
fn rejects_toml_syntax_error() {
    let e = parse_manifest("[package\nname=\"a\"\n").unwrap_err();
    assert_eq!(e.code, "M0002");
}
