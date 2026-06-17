use std::path::Path;

use assert_cmd::Command;
use lupa::{Language, SymbolKind};

const DIGEST_FIXTURE: &str = "tests/fixtures/digest_tree";
const FALLBACK_BASH_FIXTURE: &str = "tests/fixtures/fallback/script.bash";
const FALLBACK_CMAKE_FIXTURE: &str = "tests/fixtures/fallback/CMakeLists.txt";
const FALLBACK_CSS_FIXTURE: &str = "tests/fixtures/fallback/stylesheet.css";
const FALLBACK_DOCKERFILE_FIXTURE: &str = "tests/fixtures/fallback/Dockerfile";
const FALLBACK_FISH_FIXTURE: &str = "tests/fixtures/fallback/activate.fish";
const FALLBACK_LUA_FIXTURE: &str = "tests/fixtures/fallback/probe.lua";
const FALLBACK_NGINX_FIXTURE: &str = "tests/fixtures/fallback/nginx.conf";
const FALLBACK_PROTO_FIXTURE: &str = "tests/fixtures/fallback/parser.proto";
const C_FIXTURE: &str = "tests/fixtures/source_shapes.c";
const CC_FIXTURE: &str = "tests/fixtures/source_shapes.cc";
const CPP_FIXTURE: &str = "tests/fixtures/source_shapes.cpp";
const CXX_FIXTURE: &str = "tests/fixtures/source_shapes.cxx";
const GO_FIXTURE: &str = "tests/fixtures/source_shapes.go";
const H_FIXTURE: &str = "tests/fixtures/source_shapes.h";
const HH_FIXTURE: &str = "tests/fixtures/source_shapes.hh";
const HPP_FIXTURE: &str = "tests/fixtures/source_shapes.hpp";
const HXX_FIXTURE: &str = "tests/fixtures/source_shapes.hxx";
const JS_FIXTURE: &str = "tests/fixtures/source_shapes.js";
const JSON_FIXTURE: &str = "tests/fixtures/source_shapes.json";
const JUST_FIXTURE: &str = "tests/fixtures/justfile";
const JSX_FIXTURE: &str = "tests/fixtures/source_shapes.jsx";
const MARKDOWN_FIXTURE: &str = "tests/fixtures/duplicate_headings.md";
const NIX_FIXTURE: &str = "tests/fixtures/source_shapes.nix";
const NO_BLOCK_PLS_FIXTURE: &str = "tests/fixtures/no_block_pls_shapes.rs";
const PARSE_ERROR_FIXTURE: &str = "tests/fixtures/parse_error.rs";
const PYTHON_FIXTURE: &str = "tests/fixtures/source_shapes.py";
const RUST_ATTRIBUTES_FIXTURE: &str = "tests/fixtures/rust_attributes.rs";
const RUST_FIXTURE: &str = "tests/fixtures/rust_symbols.rs";
const TOML_FIXTURE: &str = "tests/fixtures/source_shapes.toml";
const TS_FIXTURE: &str = "tests/fixtures/source_shapes.ts";
const TSX_FIXTURE: &str = "tests/fixtures/source_shapes.tsx";
const TYPST_FIXTURE: &str = "tests/fixtures/source_shapes.typ";
const UNSUPPORTED_FIXTURE: &str = "tests/fixtures/not_source.txt";
const YAML_FIXTURE: &str = "tests/fixtures/source_shapes.yaml";

#[test]
fn map_prints_exact_keys_that_show_accepts() {
    let stdout = run_lupa(&["map", RUST_FIXTURE]);

    for key in [
        " Alpha ",
        " Alpha.new ",
        " Alpha.greet ",
        " Beta ",
        " Beta.new ",
        " parse_config ",
    ] {
        assert_stdout_contains(&stdout, key);
    }

    let stdout = run_lupa(&[
        "show",
        RUST_FIXTURE,
        "Alpha.new",
        "Alpha.greet",
        "Beta.new",
        "parse_config",
    ]);

    for key in [
        "# Alpha.new@",
        "# Alpha.greet@",
        "# Beta.new@",
        "# parse_config@",
    ] {
        assert_stdout_contains(&stdout, key);
    }
}

#[test]
fn direct_file_invocation_aliases_to_map() {
    let stdout = run_lupa(&[RUST_FIXTURE]);

    assert_stdout_contains(
        &stdout,
        "# tests/fixtures/rust_symbols.rs [rust] 25L 299B 9S\n",
    );
    assert_stdout_contains(&stdout, " Alpha.new ");
}

#[test]
fn show_accepts_multiple_keys_and_prints_source_slices() {
    let stdout = run_lupa(&["show", RUST_FIXTURE, "Alpha.new", "Beta.new"]);

    for line in [
        "# Alpha.new@L6-L8\n",
        "pub fn new(name: String) -> Self {\n",
        "    Self { name }\n",
        "}\n",
        "# Beta.new@L18-L20\n",
        "pub fn new() -> Self {\n",
        "    Self\n",
        "}\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn ambiguous_suffix_reports_all_candidates() {
    let stdout = run_lupa(&["show", RUST_FIXTURE, "new"]);

    for line in [
        "# amb new\n",
        "# Alpha.new@L6-L8 pub fn new(name: String) -> Self\n",
        "# Beta.new@L18-L20 pub fn new() -> Self\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }

    assert_stdout_lacks(&stdout, "    Self { name }\n");
    assert_stdout_lacks(&stdout, "    Self\n");
}

#[test]
fn show_relaxes_parent_segments_when_unambiguous() {
    let stdout = run_lupa_stdin(
        &["show", "rust", "Storage.open"],
        r"
pub struct CoreStorage;

impl CoreStorage {
    pub fn open() -> Self {
        Self
    }

    pub fn open_stats(&self) -> usize {
        1
    }
}
",
    );

    assert_stdout_contains(&stdout, "# CoreStorage.open@");
    assert_stdout_contains(&stdout, "pub fn open() -> Self {\n");
    assert_stdout_lacks(&stdout, "# CoreStorage.open_stats@");
    assert_stdout_lacks(&stdout, "pub fn open_stats(&self) -> usize {\n");
}

#[test]
fn show_reports_ambiguous_relaxed_parent_segments() {
    let stdout = run_lupa_stdin(
        &["show", "rust", "Storage.open"],
        r"
pub struct CoreStorage;
pub struct FileStorage;

impl CoreStorage {
    pub fn open() -> usize {
        1
    }
}

impl FileStorage {
    pub fn open() -> usize {
        2
    }
}
",
    );

    for line in [
        "# amb Storage.open\n",
        "# CoreStorage.open@L6-L8 pub fn open() -> usize\n",
        "# FileStorage.open@L12-L14 pub fn open() -> usize\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }

    assert_stdout_lacks(&stdout, "    1\n");
    assert_stdout_lacks(&stdout, "    2\n");
}

#[test]
fn exact_show_key_wins_before_relaxed_parent_segments() {
    let stdout = run_lupa_stdin(
        &["show", "rust", "Storage.open"],
        r"
pub struct Storage;
pub struct CoreStorage;

impl Storage {
    pub fn open() -> usize {
        1
    }
}

impl CoreStorage {
    pub fn open() -> usize {
        2
    }
}
",
    );

    assert_stdout_contains(&stdout, "# Storage.open@");
    assert_stdout_contains(&stdout, "    1\n");
    assert_stdout_lacks(&stdout, "# CoreStorage.open@");
    assert_stdout_lacks(&stdout, "    2\n");
}

#[test]
fn relaxed_show_key_requires_exact_leaf_segment() {
    let stdout = run_lupa_stdin(
        &["show", "rust", "Storage.open"],
        r"
pub struct CoreStorage;

impl CoreStorage {
    pub fn open_stats(&self) -> usize {
        1
    }
}
",
    );

    assert_stdout_contains(&stdout, "# no Storage.open\n");
    assert_stdout_contains(&stdout, "# candidates\n");
    assert_stdout_contains(&stdout, "# CoreStorage.open_stats@");
    assert_stdout_lacks(&stdout, "    1\n");
}

#[test]
fn markdown_duplicate_headings_get_deterministic_keys() {
    let stdout = run_lupa(&["map", MARKDOWN_FIXTURE]);
    let repeated_stdout = run_lupa(&["map", MARKDOWN_FIXTURE]);

    assert_stdout_contains(&stdout, " Install # Install\n");
    assert_stdout_contains(&stdout, " Install#2 # Install\n");
    assert_eq!(stdout, repeated_stdout);
}

#[test]
fn keys_prints_key_range_lines() {
    let stdout = run_lupa(&["keys", RUST_FIXTURE]);

    for line in [
        "Alpha L1-L3\n",
        "Alpha.new L6-L8\n",
        "Alpha.greet L10-L12\n",
        "Beta L15\n",
        "Beta.new L18-L20\n",
        "parse_config L23-L25\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }

    assert_stdout_lacks(&stdout, "key=");
}

#[test]
fn stdin_map_accepts_canonical_language_token() {
    let stdout = run_lupa_stdin(&["map", "rust"], include_str!("fixtures/rust_symbols.rs"));

    assert_stdout_contains(&stdout, "# - [rust] 25L 299B 9S\n");
    assert_stdout_contains(&stdout, " Alpha.new ");
}

#[test]
fn language_detects_justfile_names_and_extension() {
    for path in ["justfile", "Justfile", "JUSTFILE", "tasks.just"] {
        assert_eq!(Language::from_path(Path::new(path)), Some(Language::Just));
    }
}

#[test]
fn fallback_language_list_detects_paths_and_tokens() {
    for (token, language, paths) in [
        ("bash", Language::Bash, &["script.sh", "script.bash"][..]),
        ("fish", Language::Fish, &["script.fish"][..]),
        (
            "dockerfile",
            Language::Dockerfile,
            &[
                "Dockerfile",
                "dockerfile",
                "service.docker",
                "service.dockerfile",
            ][..],
        ),
        (
            "proto",
            Language::Proto,
            &["service.proto", "service.protobuf"][..],
        ),
        ("nginx", Language::Nginx, &["nginx.conf", "site.nginx"][..]),
        (
            "cmake",
            Language::Cmake,
            &["CMakeLists.txt", "module.cmake"][..],
        ),
        ("css", Language::Css, &["style.css"][..]),
        ("lua", Language::Lua, &["init.lua"][..]),
    ] {
        assert_eq!(Language::from_token(token), Some(language), "{token}");
        for path in paths {
            assert_eq!(
                Language::from_path(Path::new(path)),
                Some(language),
                "{path}"
            );
        }
    }

    assert_eq!(SymbolKind::Node.to_string(), "node");
}

#[test]
fn stdin_show_accepts_canonical_language_token() {
    let stdout = run_lupa_stdin(
        &["show", "rust", "Alpha.new"],
        include_str!("fixtures/rust_symbols.rs"),
    );

    assert_stdout_contains(&stdout, "# Alpha.new@L6-L8\n");
    assert_stdout_contains(&stdout, "pub fn new(name: String) -> Self {\n");
    assert_stdout_lacks(&stdout, "# - [rust]");
}

#[test]
fn stdin_keys_accepts_canonical_language_token() {
    let stdout = run_lupa_stdin(&["keys", "rust"], include_str!("fixtures/rust_symbols.rs"));

    for line in [
        "Alpha L1-L3\n",
        "Alpha.new L6-L8\n",
        "parse_config L23-L25\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn stdin_map_dispatches_non_rust_language_tokens() {
    let stdout = run_lupa_stdin(
        &["map", "go"],
        "package main\n\ntype Server struct {}\n\nfunc NewServer() Server { return Server{} }\n",
    );

    assert_stdout_contains(&stdout, "# - [go] 5L 81B 2S\n");
    assert_stdout_contains(&stdout, " Server type Server struct\n");
    assert_stdout_contains(&stdout, " NewServer func NewServer() Server\n");

    let stdout = run_lupa_stdin(&["map", "json"], r#"{"service": {"name": "api"}}"#);
    assert_stdout_contains(&stdout, "# - [json] 1L 28B 2S\n");
    assert_stdout_contains(&stdout, " service ");
    assert_stdout_contains(&stdout, " service.name ");

    let stdout = run_lupa_stdin(&["map", "just"], "build:\n    cargo build\n");
    assert_stdout_contains(&stdout, "# - [just] 2L 23B 1S\n");
    assert_stdout_contains(&stdout, " build build:\n");

    let stdout = run_lupa_stdin(
        &["map", "toml"],
        "title = \"Lupa\"\n[service]\nname = \"api\"\n",
    );
    assert_stdout_contains(&stdout, "# - [toml] 3L 38B 3S\n");
    assert_stdout_contains(&stdout, " title title = \"Lupa\"\n");
    assert_stdout_contains(&stdout, " service [service]\n");

    let stdout = run_lupa_stdin(&["map", "nix"], "{ service.enable = true; }\n");
    assert_stdout_contains(&stdout, "# - [nix] 1L 27B 1S\n");
    assert_stdout_contains(&stdout, " service.enable service.enable = true;\n");

    let stdout = run_lupa_stdin(&["map", "yaml"], "service:\n  name: api\n");
    assert_stdout_contains(&stdout, "# - [yaml] 2L 21B 2S\n");
    assert_stdout_contains(&stdout, " service service:\n");
    assert_stdout_contains(&stdout, " service.name name: api\n");

    let stdout = run_lupa_stdin(&["map", "typst"], "#let title = \"Lupa\"\n\n= Intro\n");
    assert_stdout_contains(&stdout, "# - [typst] 3L 29B 2S\n");
    assert_stdout_contains(&stdout, " title #let title = \"Lupa\"\n");
    assert_stdout_contains(&stdout, " Intro = Intro\n");
}

#[test]
fn stdin_map_dispatches_fallback_language_tokens() {
    for (token, source, needles) in [
        (
            "bash",
            include_str!("fixtures/fallback/script.bash"),
            &[
                " function_definition get_exclusion_list()",
                " function_definition#2 calculate_excluded_size()",
            ][..],
        ),
        (
            "fish",
            include_str!("fixtures/fallback/activate.fish"),
            &[
                " function_definition function deactivate",
                " if_statement if set -q PYTHONHOME",
            ][..],
        ),
        (
            "dockerfile",
            include_str!("fixtures/fallback/Dockerfile"),
            &[
                " from_instruction FROM rust:1",
                " run_instruction RUN cargo --version",
            ][..],
        ),
        (
            "proto",
            include_str!("fixtures/fallback/parser.proto"),
            &[" message RegisterRequest", " service ParserService"][..],
        ),
        (
            "nginx",
            include_str!("fixtures/fallback/nginx.conf"),
            &[" directive user nginx;", " attribute http {"][..],
        ),
        (
            "cmake",
            include_str!("fixtures/fallback/CMakeLists.txt"),
            &[
                " normal_command cmake_minimum_required",
                " foreach_loop foreach (SUBPROJ",
            ][..],
        ),
        (
            "css",
            include_str!("fixtures/fallback/stylesheet.css"),
            &[" at_rule @font-face", " rule_set .button {"][..],
        ),
        (
            "lua",
            include_str!("fixtures/fallback/probe.lua"),
            &[
                " hash_bang_line",
                " variable_declaration local ffi",
                " function_call pcall",
            ][..],
        ),
    ] {
        let stdout = run_lupa_stdin(&["map", token], source);
        assert_stdout_contains(&stdout, &format!("# - [{token}] "));
        assert_stdout_contains(
            &stdout,
            "# warning: syntax-only adapter: top-level syntax nodes only\n",
        );
        assert_stdout_lacks(&stdout, "parse error");
        for needle in needles {
            assert_stdout_contains(&stdout, needle);
        }
    }
}

#[test]
fn stdin_language_mode_rejects_extension_aliases() {
    let stdout = run_lupa_stdin(&["map", "rs"], include_str!("fixtures/rust_symbols.rs"));

    assert_eq!(stdout, "# error: path not found: rs\n");
}

#[test]
fn stdin_language_mode_rejects_mixed_map_args() {
    let stdout = run_lupa_stdin(
        &["map", "rust", RUST_FIXTURE],
        include_str!("fixtures/rust_symbols.rs"),
    );

    assert_eq!(
        stdout,
        "# error: stdin language mode accepts exactly one language token\n"
    );
}

#[test]
fn language_token_without_stdin_falls_back_to_path_mode() {
    let stdout = run_lupa(&["map", "rust"]);

    assert_eq!(stdout, "# error: path not found: rust\n");
}

#[test]
fn rust_map_prints_attributes_before_signatures() {
    let stdout = run_lupa(&["map", RUST_ATTRIBUTES_FIXTURE]);

    for line in [
        "L1-L6 TupleReader #[derive(Debug, Clone)] #[pyclass] pub struct TupleReader\n",
        "  L4-L5 TupleReader.items #[pyo3(get)] items: Vec<u8>\n",
        "  L10-L13 TupleReader.new #[new] pub fn new(items: Vec<u8>) -> Self\n",
        "  L15-L18 TupleReader.remaining #[getter] pub fn remaining(&self) -> usize\n",
        "L8-L19 impl_TupleReader #[pymethods] impl TupleReader\n",
        "L21-L24 WireValue pub enum WireValue\n",
        "  L22-L23 WireValue.Cell #[serde(rename = \"cell\")] Cell\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn rust_show_includes_attached_attributes() {
    let stdout = run_lupa(&["show", RUST_ATTRIBUTES_FIXTURE, "TupleReader.new"]);

    for line in [
        "# TupleReader.new@L10-L13\n",
        "#[new]\n",
        "pub fn new(items: Vec<u8>) -> Self {\n",
        "    Self { items }\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn rust_context_prints_attributes_in_semantic_anchor() {
    let stdout = run_lupa(&["context", "tests/fixtures/rust_attributes.rs:10"]);

    assert_stdout_contains(
        &stdout,
        "tests/fixtures/rust_attributes.rs TupleReader.new@L10-L13 hits L10 #[new] pub fn new(items: Vec<u8>) -> Self\n",
    );
    assert_stdout_contains(
        &stdout,
        "  parent TupleReader@L1-L6 #[derive(Debug, Clone)] #[pyclass] pub struct TupleReader\n",
    );
}

#[test]
fn rust_attrs_survive_intervening_comments() {
    let stdout = run_lupa(&["show", RUST_ATTRIBUTES_FIXTURE, "documented_attr"]);

    for line in [
        "# documented_attr@L26-L28\n",
        "#[outer]\n",
        "// keep attr attached\n",
        "pub fn documented_attr() {}\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn rust_tuple_fields_group_attributes_visibility_and_comments() {
    let stdout = run_lupa(&["map", RUST_ATTRIBUTES_FIXTURE]);

    for line in [
        "L30-L36 TupleAttrs pub struct TupleAttrs\n",
        "  L31-L32 TupleAttrs.0 #[first] 0: pub u8\n",
        "  L34-L35 TupleAttrs.1 #[second] 1: pub(crate) String\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }

    for bogus_field in [
        "TupleAttrs.0 #[first] 0: pub\n",
        "TupleAttrs.1 1: u8\n",
        "TupleAttrs.2 2: // separator\n",
    ] {
        assert_stdout_lacks(&stdout, bogus_field);
    }
}

#[test]
fn rust_module_attrs_do_not_leak_to_children() {
    let stdout = run_lupa(&["map", RUST_ATTRIBUTES_FIXTURE]);

    assert_stdout_contains(&stdout, "L40-L41 attr_mod.inner #[test] fn inner()\n");
    assert_stdout_lacks(&stdout, "#[cfg(test)] fn inner()");
    assert_stdout_lacks(&stdout, " attr_mod #[cfg(test)] mod attr_mod");
}

#[test]
fn context_maps_direct_line_hits_to_semantic_symbols() {
    let stdout = run_lupa(&["context", "tests/fixtures/rust_symbols.rs:7"]);

    assert_stdout_contains(
        &stdout,
        "tests/fixtures/rust_symbols.rs Alpha.new@L6-L8 hits L7 pub fn new(name: String) -> Self\n",
    );
    assert_stdout_contains(&stdout, "  parent Alpha@L1-L3 pub struct Alpha\n");
    assert_stdout_contains(&stdout, "  siblings Alpha.name@L2 Alpha.greet@L10-L12\n");
}

#[test]
fn context_reads_rg_hits_from_stdin_and_deduplicates_lines() {
    let stdout = run_lupa_stdin(
        &["context"],
        "tests/fixtures/rust_symbols.rs:7:        Self { name }\n\
         tests/fixtures/rust_symbols.rs:8:    }\n\
         tests/fixtures/rust_symbols.rs:7:duplicate\n",
    );

    assert_stdout_contains(
        &stdout,
        "tests/fixtures/rust_symbols.rs Alpha.new@L6-L8 hits L7,L8 pub fn new(name: String) -> Self\n",
    );
}

#[test]
fn context_accepts_vimgrep_columns_and_non_rust_symbols() {
    let stdout = run_lupa(&[
        "context",
        "tests/fixtures/source_shapes.py:7:15:async def start",
    ]);

    assert_stdout_contains(
        &stdout,
        "tests/fixtures/source_shapes.py Service.start@L7-L8 hits L7 async def start(self, retries: int = 1) -> str\n",
    );
    assert_stdout_contains(&stdout, "  parent Service@L1-L8 class Service\n");
}

#[test]
fn context_reports_malformed_and_outside_symbol_hits() {
    let stdout = run_lupa(&["context", "not-a-hit", "tests/fixtures/rust_symbols.rs:14"]);

    assert_stdout_contains(&stdout, "# error: malformed context hit: not-a-hit\n");
    assert_stdout_contains(
        &stdout,
        "tests/fixtures/rust_symbols.rs no-symbol hits L14\n",
    );
    assert_stdout_contains(
        &stdout,
        "  nearby before impl_Alpha@L5-L13 after Beta@L15\n",
    );
}

#[test]
fn context_keeps_separate_no_symbol_gaps() {
    let stdout = run_lupa(&[
        "context",
        "tests/fixtures/rust_symbols.rs:4",
        "tests/fixtures/rust_symbols.rs:14",
    ]);

    assert_stdout_contains(
        &stdout,
        "tests/fixtures/rust_symbols.rs no-symbol hits L4\n",
    );
    assert_stdout_contains(
        &stdout,
        "  nearby before Alpha@L1-L3 after impl_Alpha@L5-L13\n",
    );
    assert_stdout_contains(
        &stdout,
        "tests/fixtures/rust_symbols.rs no-symbol hits L14\n",
    );
    assert_stdout_contains(
        &stdout,
        "  nearby before impl_Alpha@L5-L13 after Beta@L15\n",
    );
    assert_stdout_lacks(&stdout, "no-symbol hits L4,L14");
}

#[test]
fn digest_skips_ignored_directories() {
    let stdout = run_lupa(&["digest", DIGEST_FIXTURE]);

    assert_stdout_contains(&stdout, "tests/fixtures/digest_tree/visible.rs");
    assert_stdout_lacks(&stdout, "target/ignored.rs");
    assert_stdout_lacks(&stdout, "ignored.rs [rust]");
}

#[test]
fn digest_includes_polyglot_source_extensions() {
    let stdout = run_lupa(&["digest", DIGEST_FIXTURE]);

    for path in [
        "tests/fixtures/digest_tree/visible.c",
        "tests/fixtures/digest_tree/visible.cc",
        "tests/fixtures/digest_tree/visible.cpp",
        "tests/fixtures/digest_tree/visible.cxx",
        "tests/fixtures/digest_tree/visible.go",
        "tests/fixtures/digest_tree/visible.h",
        "tests/fixtures/digest_tree/visible.hh",
        "tests/fixtures/digest_tree/visible.hpp",
        "tests/fixtures/digest_tree/visible.hxx",
        "tests/fixtures/digest_tree/visible.js",
        "tests/fixtures/digest_tree/visible.json",
        "tests/fixtures/digest_tree/visible.just",
        "tests/fixtures/digest_tree/visible.jsx",
        "tests/fixtures/digest_tree/justfile",
        "tests/fixtures/digest_tree/visible.nix",
        "tests/fixtures/digest_tree/visible.py",
        "tests/fixtures/digest_tree/visible.ts",
        "tests/fixtures/digest_tree/visible.toml",
        "tests/fixtures/digest_tree/visible.tsx",
        "tests/fixtures/digest_tree/visible.typ",
        "tests/fixtures/digest_tree/visible.yaml",
        "tests/fixtures/digest_tree/visible.yml",
        "tests/fixtures/digest_tree/visible.bash",
        "tests/fixtures/digest_tree/visible.cmake",
        "tests/fixtures/digest_tree/visible.css",
        "tests/fixtures/digest_tree/visible.dockerfile",
        "tests/fixtures/digest_tree/visible.fish",
        "tests/fixtures/digest_tree/visible.lua",
        "tests/fixtures/digest_tree/visible.nginx",
        "tests/fixtures/digest_tree/visible.proto",
    ] {
        assert_stdout_contains(&stdout, path);
    }
    assert_stdout_contains(&stdout, "visible.dockerfile [dockerfile]");
    assert_stdout_contains(&stdout, "visible.proto [proto]");
    assert_stdout_contains(&stdout, "syntax-only");
}

#[test]
fn parse_error_warning_appears_with_partial_output() {
    let stdout = run_lupa(&["map", PARSE_ERROR_FIXTURE]);

    assert_stdout_contains(&stdout, "# tests/fixtures/parse_error.rs [rust]");
    assert_stdout_contains(
        &stdout,
        "# warning: parse error at L1: parse error in ERROR\n",
    );
}

#[test]
fn retained_fallback_fixtures_map_useful_top_level_nodes() {
    for (fixture, language, needles) in [
        (
            FALLBACK_BASH_FIXTURE,
            "bash",
            &[
                " function_definition get_exclusion_list()",
                " function_definition#2 calculate_excluded_size()",
            ][..],
        ),
        (
            FALLBACK_CMAKE_FIXTURE,
            "cmake",
            &[
                " normal_command cmake_minimum_required",
                " foreach_loop foreach (SUBPROJ",
            ][..],
        ),
        (
            FALLBACK_CSS_FIXTURE,
            "css",
            &[" at_rule @font-face", " rule_set .button {"][..],
        ),
        (
            FALLBACK_DOCKERFILE_FIXTURE,
            "dockerfile",
            &[
                " from_instruction FROM rust:1",
                " run_instruction RUN cargo --version",
            ][..],
        ),
        (
            FALLBACK_FISH_FIXTURE,
            "fish",
            &[
                " function_definition function deactivate",
                " if_statement if set -q PYTHONHOME",
            ][..],
        ),
        (
            FALLBACK_LUA_FIXTURE,
            "lua",
            &[
                " hash_bang_line",
                " variable_declaration local ffi",
                " function_call pcall",
            ][..],
        ),
        (
            FALLBACK_NGINX_FIXTURE,
            "nginx",
            &[" directive user nginx;", " attribute http {"][..],
        ),
        (
            FALLBACK_PROTO_FIXTURE,
            "proto",
            &[" message RegisterRequest", " service ParserService"][..],
        ),
    ] {
        let stdout = run_lupa(&["map", fixture]);
        assert_stdout_contains(&stdout, &format!("# {fixture} [{language}] "));
        assert_stdout_contains(
            &stdout,
            "# warning: syntax-only adapter: top-level syntax nodes only\n",
        );
        assert_stdout_lacks(&stdout, "parse error");
        for needle in needles {
            assert_stdout_contains(&stdout, needle);
        }
    }
}

#[test]
fn unsupported_file_type_is_recoverable_error() {
    let stdout = run_lupa(&["map", UNSUPPORTED_FIXTURE]);

    assert_eq!(
        stdout,
        "# error: unsupported file type: tests/fixtures/not_source.txt\n"
    );

    let stdout = run_lupa(&["map", "tests/fixtures/not_source.sass"]);
    assert_eq!(
        stdout,
        "# error: unsupported file type: tests/fixtures/not_source.sass\n"
    );

    let stdout = run_lupa(&["map", "tests/fixtures/not_source.conf"]);
    assert_eq!(
        stdout,
        "# error: unsupported file type: tests/fixtures/not_source.conf\n"
    );
}

#[test]
fn help_exits_successfully() {
    let stdout = run_lupa(&["--help"]);

    assert_stdout_contains(&stdout, "Usage: lupa <COMMAND>\n");
    assert_stdout_contains(&stdout, "Commands:\n");
}

#[test]
fn no_block_pls_shapes_map_to_stable_keys() {
    let stdout = run_lupa(&["map", NO_BLOCK_PLS_FIXTURE]);

    for key in [
        " Receiver.recv ",
        " Broadcaster.run ",
        " Storage.remove_outdated_states ",
        " poll_impl ",
    ] {
        assert_stdout_contains(&stdout, key);
    }
}

#[test]
fn no_block_pls_shapes_show_generic_impl_and_long_functions() {
    let stdout = run_lupa(&[
        "show",
        NO_BLOCK_PLS_FIXTURE,
        "Receiver.recv",
        "Storage.remove_outdated_states",
        "poll_impl",
    ]);

    for line in [
        "Receiver.recv@L20-L22\n",
        "async fn recv(&mut self) -> Option<T> {\n",
        "Storage.remove_outdated_states@L51-L66\n",
        "#[tracing::instrument(skip(self))]\n",
        "pub async fn remove_outdated_states(&self, mc_seqno: u32) -> Result<(), Error> {\n",
        "poll_impl@L83-L98\n",
        "fn poll_impl<'cx, Fut>(\n",
        "where\n",
    ] {
        assert_stdout_contains(&stdout, line);
    }
}

#[test]
fn polyglot_map_prints_expected_keys() {
    for (fixture, keys) in [
        (
            C_FIXTURE,
            &[
                " Config ",
                " Config.timeout_ms ",
                " Config.name ",
                " Mode ",
                " helper ",
                " run_loop ",
            ][..],
        ),
        (
            H_FIXTURE,
            &[" HeaderConfig ", " HeaderConfig.retries ", " make_config "][..],
        ),
        (CC_FIXTURE, &[" util.add "][..]),
        (
            CPP_FIXTURE,
            &[
                " net.Client ",
                " net.Client.connect ",
                " net.Client.connect#2 ",
                " net.Client.operator_eq ",
                " net.identity ",
            ][..],
        ),
        (CXX_FIXTURE, &[" math.square "][..]),
        (
            HPP_FIXTURE,
            &[" api.Widget ", " api.Widget.render ", " api.make_widget "][..],
        ),
        (HH_FIXTURE, &[" HeaderOnly ", " HeaderOnly.value "][..]),
        (HXX_FIXTURE, &[" headers.Kind "][..]),
        (
            PYTHON_FIXTURE,
            &[
                " Service ",
                " Service.__init__ ",
                " Service.start ",
                " build_service ",
            ][..],
        ),
        (
            JS_FIXTURE,
            &[
                " Widget ",
                " Widget.constructor ",
                " Widget.render ",
                " makeWidget ",
            ][..],
        ),
        (
            JSON_FIXTURE,
            &[
                " service ",
                " service.name ",
                " service.limits ",
                " service.limits.timeout_ms ",
                " scripts.build ",
            ][..],
        ),
        (
            JUST_FIXTURE,
            &[
                " profile ",
                " PROFILE ",
                " t ",
                " build ",
                " prepare ",
                " test ",
            ][..],
        ),
        (JSX_FIXTURE, &[" Card ", " Shell "][..]),
        (
            NIX_FIXTURE,
            &[
                " local ",
                " services.demo.enable ",
                " packages ",
                " nested.value ",
            ][..],
        ),
        (
            TOML_FIXTURE,
            &[
                " title ",
                " service ",
                " service.name ",
                " service.limits ",
                " service.limits.retries ",
                " plugins.enabled ",
                " metadata.owner.name ",
            ][..],
        ),
        (
            YAML_FIXTURE,
            &[
                " service ",
                " service.name ",
                " service.limits.timeout_ms ",
                " plugins.name ",
                " metadata.owner ",
            ][..],
        ),
        (
            TS_FIXTURE,
            &[
                " Repository ",
                " Repository.get ",
                " User ",
                " UserService ",
                " UserService.constructor ",
                " UserService.load ",
                " formatUser ",
            ][..],
        ),
        (
            TSX_FIXTURE,
            &[
                " ButtonProps ",
                " ButtonProps.label ",
                " ButtonProps.onClick ",
                " Button ",
                " Toolbar ",
            ][..],
        ),
        (
            GO_FIXTURE,
            &[
                " Server ",
                " Server.name ",
                " Server.Handler ",
                " Server.clock ",
                " Server.Start ",
                " Handler ",
                " Handler.Handle ",
                " Handler.Close ",
                " Clock ",
                " Clock.Now ",
                " Alias ",
                " NewServer ",
                " helper ",
            ][..],
        ),
        (
            TYPST_FIXTURE,
            &[
                " title ",
                " accent-color ",
                " string_source ",
                " Resume ",
                " Resume.Summary ",
                " Resume.Experience ",
            ][..],
        ),
    ] {
        let stdout = run_lupa(&["map", fixture]);
        for key in keys {
            assert_stdout_contains(&stdout, key);
        }
    }

    let stdout = run_lupa(&["keys", TYPST_FIXTURE]);
    for key in [
        " Raw Fake ",
        " raw_fake ",
        " Comment Fake ",
        " comment_fake ",
        " local_fake ",
        " String Fake ",
        " string_fake ",
    ] {
        assert_stdout_lacks(&stdout, key);
    }
}

#[test]
fn polyglot_show_prints_selected_symbols() {
    for (fixture, keys, expected) in [
        (
            C_FIXTURE,
            &["run_loop", "Config"][..],
            &[
                "# run_loop@L15-L17\n",
                "int run_loop(Config *config) {\n",
                "# Config@L1-L4\n",
                "typedef struct Config {\n",
            ][..],
        ),
        (
            H_FIXTURE,
            &["make_config"][..],
            &[
                "# make_config@L5\n",
                "int make_config(HeaderConfig *config);\n",
            ][..],
        ),
        (
            CPP_FIXTURE,
            &["net.Client.connect#2", "net.identity"][..],
            &[
                "# net.Client.connect#2@L17-L19\n",
                "int Client::connect(int timeout) {\n",
                "# net.identity@L25-L28\n",
                "template <class T>\n",
            ][..],
        ),
        (
            PYTHON_FIXTURE,
            &["Service.start", "build_service"][..],
            &[
                "# Service.start@L7-L8\n",
                "async def start(self, retries: int = 1) -> str:\n",
                "# build_service@L10-L11\n",
                "def build_service(label: str) -> Service:\n",
            ][..],
        ),
        (
            JS_FIXTURE,
            &["Widget.render", "makeWidget"][..],
            &[
                "# Widget.render@L6-L8\n",
                "render(target) {\n",
                "# makeWidget@L11-L13\n",
                "export function makeWidget(name) {\n",
            ][..],
        ),
        (
            JSON_FIXTURE,
            &["service.limits", "scripts.build"][..],
            &[
                "# service.limits@L4-L7\n",
                "\"limits\": {\n",
                "# scripts.build@L10\n",
                "\"build\": \"cargo build\",\n",
            ][..],
        ),
        (
            JUST_FIXTURE,
            &["build", "PROFILE"][..],
            &[
                "# build@L7-L9\n",
                "build target=\"debug\": prepare\n",
                "# PROFILE@L4\n",
                "export PROFILE := \"release\"\n",
            ][..],
        ),
        (
            JSX_FIXTURE,
            &["Card", "Shell"][..],
            &[
                "# Card@L1-L3\n",
                "export function Card({ title }) {\n",
                "# Shell@L5-L7\n",
                "export const Shell = () => {\n",
            ][..],
        ),
        (
            TOML_FIXTURE,
            &["service", "metadata.owner.name"][..],
            &[
                "# service@L3-L5\n",
                "[service]\n",
                "# metadata.owner.name@L15\n",
                "owner = { name = \"ops\", team = \"tools\" }\n",
            ][..],
        ),
        (
            NIX_FIXTURE,
            &["services.demo.enable", "nested.value"][..],
            &[
                "# services.demo.enable@L5\n",
                "services.demo.enable = true;\n",
                "# nested.value@L7\n",
                "nested = { value = local; };\n",
            ][..],
        ),
        (
            YAML_FIXTURE,
            &["service.limits", "metadata.owner"][..],
            &[
                "# service.limits@L3-L4\n",
                "limits:\n",
                "# metadata.owner@L8\n",
                "metadata: { owner: ops, team: tools }\n",
            ][..],
        ),
        (
            TS_FIXTURE,
            &["UserService.load", "formatUser"][..],
            &[
                "# UserService.load@L13-L15\n",
                "async load(id: string): Promise<User> {\n",
                "# formatUser@L18-L20\n",
                "export function formatUser(user: User): string {\n",
            ][..],
        ),
        (
            TSX_FIXTURE,
            &["Button", "Toolbar"][..],
            &[
                "# Button@L6-L8\n",
                "export function Button(props: ButtonProps) {\n",
                "# Toolbar@L10-L12\n",
                "export const Toolbar = () => {\n",
            ][..],
        ),
        (
            GO_FIXTURE,
            &["Server.Start", "NewServer"][..],
            &[
                "# Server.Start@L26-L28\n",
                "func (s *Server) Start(ctx context.Context) error {\n",
                "# NewServer@L22-L24\n",
                "func NewServer(name string, handler Handler) *Server {\n",
            ][..],
        ),
        (
            TYPST_FIXTURE,
            &["Resume.Summary", "accent-color"][..],
            &[
                "# Resume.Summary@L20-L22\n",
                "== Summary\n",
                "# accent-color@L2\n",
                "#let accent-color = rgb(\"#4a90e2\")\n",
            ][..],
        ),
    ] {
        let mut args = vec!["show", fixture];
        args.extend_from_slice(keys);
        let stdout = run_lupa(&args);
        for line in expected {
            assert_stdout_contains(&stdout, line);
        }
    }
}

#[test]
fn toml_table_show_does_not_include_next_table() {
    let stdout = run_lupa(&["show", TOML_FIXTURE, "service", "plugins"]);

    assert_stdout_contains(&stdout, "# service@L3-L5\n");
    assert_stdout_contains(&stdout, "timeout_ms = 5000\n");
    assert_stdout_lacks(&stdout, "[service.limits]\n");
    assert_stdout_contains(&stdout, "# plugins@L10-L12\n");
    assert_stdout_contains(&stdout, "enabled = true\n");
    assert_stdout_lacks(&stdout, "[metadata]\n");
}

#[test]
fn polyglot_keys_print_expected_ranges() {
    for (fixture, expected) in [
        (
            C_FIXTURE,
            &[
                "Config L1-L4\n",
                "Config.timeout_ms L2\n",
                "run_loop L15-L17\n",
            ][..],
        ),
        (
            CPP_FIXTURE,
            &[
                "net.Client L2-L11\n",
                "net.Client.connect L6\n",
                "net.Client.connect#2 L17-L19\n",
                "net.identity L25-L28\n",
            ][..],
        ),
        (
            HPP_FIXTURE,
            &[
                "api.Widget L2-L5\n",
                "api.Widget.render L4\n",
                "api.make_widget L7\n",
            ][..],
        ),
        (
            PYTHON_FIXTURE,
            &[
                "Service L1-L8\n",
                "Service.start L7-L8\n",
                "build_service L10-L11\n",
            ][..],
        ),
        (
            JS_FIXTURE,
            &[
                "Widget L1-L9\n",
                "Widget.render L6-L8\n",
                "makeWidget L11-L13\n",
            ][..],
        ),
        (
            JSON_FIXTURE,
            &[
                "service L2-L8\n",
                "service.name L3\n",
                "service.limits.timeout_ms L5\n",
                "scripts.build L10\n",
            ][..],
        ),
        (
            JUST_FIXTURE,
            &[
                "profile L3\n",
                "PROFILE L4\n",
                "t L5\n",
                "build L7-L9\n",
                "prepare L10-L12\n",
                "test L13-L14\n",
            ][..],
        ),
        (JSX_FIXTURE, &["Card L1-L3\n", "Shell L5-L7\n"][..]),
        (
            TOML_FIXTURE,
            &[
                "title L1\n",
                "service L3-L5\n",
                "service.limits L7-L8\n",
                "plugins.enabled L12\n",
                "metadata.owner.name L15\n",
            ][..],
        ),
        (
            NIX_FIXTURE,
            &[
                "local L3\n",
                "services.demo.enable L5\n",
                "packages L6\n",
                "nested.value L7\n",
            ][..],
        ),
        (
            YAML_FIXTURE,
            &[
                "service L1-L4\n",
                "service.name L2\n",
                "service.limits.timeout_ms L4\n",
                "plugins.name L6\n",
                "metadata.owner L8\n",
            ][..],
        ),
        (
            TS_FIXTURE,
            &[
                "Repository L1-L3\n",
                "UserService.load L13-L15\n",
                "formatUser L18-L20\n",
            ][..],
        ),
        (
            TSX_FIXTURE,
            &["ButtonProps L1-L4\n", "Button L6-L8\n", "Toolbar L10-L12\n"][..],
        ),
        (
            GO_FIXTURE,
            &[
                "Server L5-L9\n",
                "Server.name L6\n",
                "Server.Handler L7\n",
                "Server.Start L26-L28\n",
                "Handler.Handle L12\n",
                "NewServer L22-L24\n",
            ][..],
        ),
        (
            TYPST_FIXTURE,
            &[
                "title L1\n",
                "accent-color L2\n",
                "Resume L5-L24\n",
                "Resume.Summary L20-L22\n",
                "Resume.Experience L23-L24\n",
            ][..],
        ),
    ] {
        let stdout = run_lupa(&["keys", fixture]);
        for line in expected {
            assert_stdout_contains(&stdout, line);
        }
        assert_stdout_lacks(&stdout, "key=");
    }
}

fn run_lupa(args: &[&str]) -> String {
    run_lupa_inner(args, None)
}

fn run_lupa_stdin(args: &[&str], stdin: &str) -> String {
    run_lupa_inner(args, Some(stdin))
}

fn run_lupa_inner(args: &[&str], stdin: Option<&str>) -> String {
    let mut command = Command::cargo_bin("lupa").expect("lupa binary should build");
    command.args(args);
    if let Some(stdin) = stdin {
        command.write_stdin(stdin);
    }
    let output = command.output().expect("lupa command should run");
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf-8");

    assert!(
        output.status.success(),
        "lupa {args:?} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(stderr, "", "lupa {args:?} wrote stderr");

    stdout
}

fn assert_stdout_contains(stdout: &str, needle: &str) {
    assert!(
        stdout.contains(needle),
        "stdout missing {needle:?}\nstdout:\n{stdout}"
    );
}

fn assert_stdout_lacks(stdout: &str, needle: &str) {
    assert!(
        !stdout.contains(needle),
        "stdout unexpectedly contained {needle:?}\nstdout:\n{stdout}"
    );
}
