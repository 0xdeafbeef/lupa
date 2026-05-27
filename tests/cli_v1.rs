use assert_cmd::Command;

const DIGEST_FIXTURE: &str = "tests/fixtures/digest_tree";
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
const JSX_FIXTURE: &str = "tests/fixtures/source_shapes.jsx";
const MARKDOWN_FIXTURE: &str = "tests/fixtures/duplicate_headings.md";
const NO_BLOCK_PLS_FIXTURE: &str = "tests/fixtures/no_block_pls_shapes.rs";
const PARSE_ERROR_FIXTURE: &str = "tests/fixtures/parse_error.rs";
const PYTHON_FIXTURE: &str = "tests/fixtures/source_shapes.py";
const RUST_ATTRIBUTES_FIXTURE: &str = "tests/fixtures/rust_attributes.rs";
const RUST_FIXTURE: &str = "tests/fixtures/rust_symbols.rs";
const TS_FIXTURE: &str = "tests/fixtures/source_shapes.ts";
const TSX_FIXTURE: &str = "tests/fixtures/source_shapes.tsx";
const UNSUPPORTED_FIXTURE: &str = "tests/fixtures/not_source.txt";

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
        "tests/fixtures/digest_tree/visible.jsx",
        "tests/fixtures/digest_tree/visible.py",
        "tests/fixtures/digest_tree/visible.ts",
        "tests/fixtures/digest_tree/visible.tsx",
    ] {
        assert_stdout_contains(&stdout, path);
    }
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
fn unsupported_file_type_is_recoverable_error() {
    let stdout = run_lupa(&["map", UNSUPPORTED_FIXTURE]);

    assert_eq!(
        stdout,
        "# error: unsupported file type: tests/fixtures/not_source.txt\n"
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
        (JSX_FIXTURE, &[" Card ", " Shell "][..]),
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
    ] {
        let stdout = run_lupa(&["map", fixture]);
        for key in keys {
            assert_stdout_contains(&stdout, key);
        }
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
        (JSX_FIXTURE, &["Card L1-L3\n", "Shell L5-L7\n"][..]),
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
