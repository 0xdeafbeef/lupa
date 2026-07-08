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
const KOTLIN_SCRIPT_FIXTURE: &str = "tests/fixtures/source_shapes.kts";
const MARKDOWN_FIXTURE: &str = "tests/fixtures/duplicate_headings.md";
const NIX_FIXTURE: &str = "tests/fixtures/source_shapes.nix";
const NO_BLOCK_PLS_FIXTURE: &str = "tests/fixtures/no_block_pls_shapes.rs";
const PARSE_ERROR_FIXTURE: &str = "tests/fixtures/parse_error.rs";
const PYTHON_FIXTURE: &str = "tests/fixtures/source_shapes.py";
const RUST_ATTRIBUTES_FIXTURE: &str = "tests/fixtures/rust_attributes.rs";
const RUST_FIXTURE: &str = "tests/fixtures/rust_symbols.rs";
const SVELTE_FIXTURE: &str = "tests/fixtures/source_shapes.svelte";
const TOML_FIXTURE: &str = "tests/fixtures/source_shapes.toml";
const TS_FIXTURE: &str = "tests/fixtures/source_shapes.ts";
const TSX_FIXTURE: &str = "tests/fixtures/source_shapes.tsx";
const TYPST_FIXTURE: &str = "tests/fixtures/source_shapes.typ";
const UNSUPPORTED_FIXTURE: &str = "tests/fixtures/not_source.txt";
const YAML_FIXTURE: &str = "tests/fixtures/source_shapes.yaml";

const EXPECTED_STRESS_LANGUAGES: &[Language] = &[
    Language::Bash,
    Language::C,
    Language::Cmake,
    Language::Cpp,
    Language::Css,
    Language::Dockerfile,
    Language::Fish,
    Language::Go,
    Language::JavaScript,
    Language::Json,
    Language::Just,
    Language::Jsx,
    Language::Kotlin,
    Language::Lua,
    Language::Markdown,
    Language::Nginx,
    Language::Nix,
    Language::Proto,
    Language::Python,
    Language::Rust,
    Language::Svelte,
    Language::Tsx,
    Language::Toml,
    Language::Typst,
    Language::TypeScript,
    Language::Yaml,
];

struct StressFixture {
    language: Language,
    fixture: &'static str,
    syntax_only: bool,
    map_needles: &'static [&'static str],
    absent_map_needles: &'static [&'static str],
    show_cases: &'static [ShowCase],
}

struct ShowCase {
    keys: &'static [&'static str],
    needles: &'static [&'static str],
}

const STRESS_FIXTURES: &[StressFixture] = &[
    StressFixture {
        language: Language::Bash,
        fixture: "tests/fixtures/stress/script.bash",
        syntax_only: true,
        map_needles: &[
            " case_statement case \"${1:-build}\" in",
            " function_definition outer() {",
            " function_definition#2 helper() {",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::C,
        fixture: "tests/fixtures/stress/source.c",
        syntax_only: false,
        map_needles: &[
            " NestedConfig ",
            " NestedConfig.callback ",
            " Mode ",
            " install_callback ",
            " run_pipeline ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["run_pipeline"],
            needles: &[
                "# run_pipeline@",
                "int run_pipeline(NestedConfig *config, Mode mode) {\n",
                "return mode == ModeHot ? install_callback(total, double_value) : total;\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Cmake,
        fixture: "tests/fixtures/stress/CMakeLists.txt",
        syntax_only: true,
        map_needles: &[
            " normal_command cmake_minimum_required(VERSION 3.24)",
            " function_def function(add_stress_target name)",
            " macro_def macro(enable_feature target feature)",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Cpp,
        fixture: "tests/fixtures/stress/source.cpp",
        syntax_only: false,
        map_needles: &[
            " engine.Pipeline ",
            " Stage ",
            " engine.Pipeline.run ",
            " engine.Pipeline_T_.configure ",
            " engine.make_pipeline ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["engine.Pipeline.run", "engine.make_pipeline"],
            needles: &[
                "# engine.Pipeline.run@",
                "auto fold = [stage](int seed) { return seed + static_cast<int>(stage.value); };\n",
                "# engine.make_pipeline@",
                "Pipeline<int> make_pipeline() {\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Css,
        fixture: "tests/fixtures/stress/stylesheet.css",
        syntax_only: true,
        map_needles: &[
            " at_rule @property --gap {",
            " media_statement @media (min-width: 720px) {",
            " supports_statement @supports selector(:has(*)) {",
            " rule_set .panel[data-state=\"open\"] > .item::before",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Dockerfile,
        fixture: "tests/fixtures/stress/Dockerfile",
        syntax_only: true,
        map_needles: &[
            " from_instruction FROM rust:1.82 AS planner",
            " env_instruction ENV CARGO_TERM_COLOR=always",
            " run_instruction RUN --mount=type=cache",
            " copy_instruction COPY --from=planner",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Fish,
        fixture: "tests/fixtures/stress/activate.fish",
        syntax_only: true,
        map_needles: &[
            " function_definition function activate --argument-names root",
            " function_definition#2 function deactivate",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Go,
        fixture: "tests/fixtures/stress/source.go",
        syntax_only: false,
        map_needles: &[
            " Loader ",
            " Loader.Load ",
            " Cache ",
            " Cache.loader ",
            " Cache.Get ",
            " NewCache ",
            " wrapLoader ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["Cache.Get", "wrapLoader"],
            needles: &[
                "# Cache.Get@",
                "func (c *Cache[T]) Get(ctx context.Context, key string) (T, error) {\n",
                "# wrapLoader@",
                "return func(ctx context.Context, key string) (T, error) {\n",
            ],
        }],
    },
    StressFixture {
        language: Language::JavaScript,
        fixture: "tests/fixtures/stress/source.js",
        syntax_only: false,
        map_needles: &[
            " Registry ",
            " Registry.from ",
            " Registry.size ",
            " Registry.register ",
            " Registry.build ",
            " createRegistry ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["Registry.build", "createRegistry"],
            needles: &[
                "# Registry.build@",
                "return [...this.#items.entries()].map",
                "# createRegistry@",
                "export function createRegistry(entries = []) {\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Json,
        fixture: "tests/fixtures/stress/source.json",
        syntax_only: false,
        map_needles: &[
            " service ",
            " service.plugins ",
            " service.plugins.hooks ",
            " matrix ",
            " matrix.include.os ",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Just,
        fixture: "tests/fixtures/stress/justfile",
        syntax_only: false,
        map_needles: &[
            " PROFILE ",
            " b ",
            " default ",
            " build ",
            " deploy ",
            " test ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["deploy"],
            needles: &[
                "# deploy@",
                "deploy host +args: build\n",
                "    rsync -az target/{{host}} {{args}}\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Jsx,
        fixture: "tests/fixtures/stress/source.jsx",
        syntax_only: false,
        map_needles: &[" Dashboard ", " Shell "],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["Dashboard"],
            needles: &[
                "# Dashboard@",
                "function Row({ row }) {\n",
                "const cells = row.items.map((item) => render?.(item)",
            ],
        }],
    },
    StressFixture {
        language: Language::Kotlin,
        fixture: "tests/fixtures/stress/source.kt",
        syntax_only: false,
        map_needles: &[
            " Screen sealed interface Screen",
            " Screen.Loading data object Loading : Screen",
            " Screen.Ready data class Ready(val count: Int) : Screen",
            " Tone enum class Tone",
            " TradeWatchItem data class TradeWatchItem",
            " TradeWatchItem.priceText val priceText: String get()",
            " TradeWatchItem.Factory companion object Factory",
            " TradeWatchItem.Factory.fallback val fallback = TradeWatchItem(\"TON\", 0.0)",
            " TradeWatchItem.Factory.fromTicker fun fromTicker(ticker: String): TradeWatchItem",
            " TradeWatchItem.label fun label(prefix: String): String",
            " TradeWatchRegistry object TradeWatchRegistry",
            " TradeWatchRegistry.default val default = TradeWatchItem(\"TON\", 1.0)",
            " TradeWatchRegistry.Companion companion object",
            " TradeWatchRegistry.Companion.empty val empty = TradeWatchItem(\"NONE\", 0.0)",
            " TradeWatchRegistry.build fun build(): TradeWatchItem",
            " TradeWatchItem.tone fun TradeWatchItem.tone(): Tone",
            " appName val appName = \"Trade\"",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &[
                "TradeWatchItem.Factory",
                "TradeWatchRegistry.Companion",
                "TradeWatchItem.label",
                "TradeWatchItem.tone",
            ],
            needles: &[
                "# TradeWatchItem.Factory@",
                "companion object Factory {\n",
                "fun fromTicker(ticker: String): TradeWatchItem = TradeWatchItem(ticker, 1.0)\n",
                "# TradeWatchRegistry.Companion@",
                "companion object {\n",
                "val empty = TradeWatchItem(\"NONE\", 0.0)\n",
                "# TradeWatchItem.label@",
                "fun label(prefix: String): String {\n",
                "val normalize = { value: String -> value.trim().uppercase() }\n",
                "# TradeWatchItem.tone@",
                "fun TradeWatchItem.tone(): Tone = if (price > 0) Tone.Good else Tone.Bad\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Lua,
        fixture: "tests/fixtures/stress/probe.lua",
        syntax_only: true,
        map_needles: &[
            " hash_bang_line #!/usr/bin/env lua",
            " variable_declaration local ffi = require(\"ffi\")",
            " function_declaration local function memoize(name, loader)",
            " function_declaration#2 function M:configure(opts)",
            " function_call setmetatable(M.cache, mt)",
            " function_call#2 pcall(function()",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Markdown,
        fixture: "tests/fixtures/stress/document.md",
        syntax_only: false,
        map_needles: &[
            " Title ",
            " Title.Setup ",
            " Title.Setup.Deep ",
            " Title.Setup#2 ",
        ],
        absent_map_needles: &["Fake Heading"],
        show_cases: &[ShowCase {
            keys: &["Title.Setup#2"],
            needles: &[
                "# Title.Setup#2@",
                "## Setup\n",
                "Duplicate heading text.\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Nginx,
        fixture: "tests/fixtures/stress/nginx.conf",
        syntax_only: true,
        map_needles: &[
            " directive user nginx;",
            " directive#2 worker_processes auto;",
            " directive#3 events {",
            " attribute http {",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Nix,
        fixture: "tests/fixtures/stress/source.nix",
        syntax_only: false,
        map_needles: &[
            " mkService ",
            " mkService.settings ",
            " services.web ",
            " packages ",
            " packages.api.package ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["services.web"],
            needles: &[
                "# services.web@",
                "services.web = mkService \"web\" 8080;\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Proto,
        fixture: "tests/fixtures/stress/schema.proto",
        syntax_only: true,
        map_needles: &[
            " syntax syntax = \"proto3\";",
            " package package stress.v1;",
            " option option go_package = \"example.com/stress/v1;stressv1\";",
            " message message Envelope {",
            " service service Router {",
        ],
        absent_map_needles: &[],
        show_cases: &[],
    },
    StressFixture {
        language: Language::Python,
        fixture: "tests/fixtures/stress/source.py",
        syntax_only: false,
        map_needles: &[
            " Options ",
            " traced ",
            " Service ",
            " Service.__init__ ",
            " Service.start ",
            " build_service ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["Service.start", "build_service"],
            needles: &[
                "# Service.start@",
                "normalize = lambda value: self.Inner().normalize(value)\n",
                "def filter_value(value: str) -> bool:\n",
                "# build_service@",
            ],
        }],
    },
    StressFixture {
        language: Language::Rust,
        fixture: "tests/fixtures/stress/source.rs",
        syntax_only: false,
        map_needles: &[
            " pipeline.Stage ",
            " pipeline.Event ",
            " pipeline.Runner ",
            " pipeline.Runner.run ",
            " impl_Runner ",
            " pipeline.build_runner ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["pipeline.Runner.run"],
            needles: &[
                "# pipeline.Runner.run@",
                "pub async fn run(&self, events: &[Event]) -> Vec<String> {\n",
                "let format = |event: &Event| match event {\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Svelte,
        fixture: "tests/fixtures/stress/source.svelte",
        syntax_only: false,
        map_needles: &[
            " script <script lang=\"ts\">",
            " main <main data-count={count}>",
            " main.button <button onclick={increment}>Clicked {count}</button>",
            " main.Icon <Icon name=\"plus\" />",
            " main.if {#if count > 0}",
            " main.if.each {#each items as item, index (item.id)}",
            " main.if.each.Row <Row {item} {index} />",
            " main.await {#await ready}",
            " main.key {#key count}",
            " main.label {#snippet label(name)}",
            " main.render {@render label(\"total\")}",
            " main.const {@const doubled = count * 2}",
            " style <style>",
        ],
        absent_map_needles: &["syntax-only adapter", "parse error"],
        show_cases: &[ShowCase {
            keys: &[
                "script",
                "main.if.each",
                "main.await",
                "main.label",
                "style",
            ],
            needles: &[
                "# script@",
                "<script lang=\"ts\">\n",
                "function increment() {\n",
                "# main.if.each@",
                "{#each items as item, index (item.id)}\n",
                "# main.await@",
                "{#await ready}\n",
                "{:catch err}\n",
                "# main.label@",
                "{#snippet label(name)}\n",
                "# style@",
                "main {\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Tsx,
        fixture: "tests/fixtures/stress/source.tsx",
        syntax_only: false,
        map_needles: &[
            " PanelProps ",
            " PanelProps.render ",
            " Panel ",
            " Toolbar ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["Panel"],
            needles: &[
                "# Panel@",
                "export function Panel<T extends { id: string }>(props: PanelProps<T>) {\n",
                "function Item({ item }: { item: T }) {\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Toml,
        fixture: "tests/fixtures/stress/source.toml",
        syntax_only: false,
        map_needles: &[
            " service ",
            " service.limits ",
            " service.routes ",
            " service.routes.path ",
            " service.routes#2 ",
            " metadata.owner.name ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["service.routes"],
            needles: &[
                "# service.routes@",
                "[[service.routes]]\n",
                "handler = { name = \"root\", cache = true }\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Typst,
        fixture: "tests/fixtures/stress/source.typ",
        syntax_only: false,
        map_needles: &[" accent ", " badge(body) ", " Report ", " Report.Section "],
        absent_map_needles: &["Fake Raw Heading", "Fake Comment Heading"],
        show_cases: &[ShowCase {
            keys: &["Report.Section"],
            needles: &[
                "# Report.Section@",
                "== Section\n",
                "Text with #badge[inline] content.\n",
            ],
        }],
    },
    StressFixture {
        language: Language::TypeScript,
        fixture: "tests/fixtures/stress/source.ts",
        syntax_only: false,
        map_needles: &[
            " RecordSource ",
            " RecordSource.load ",
            " Store ",
            " Store.constructor ",
            " Store.load ",
            " Store.map ",
            " createStore ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["Store.map"],
            needles: &[
                "# Store.map@",
                "map<R>(items: T[], project: (item: T) => R): R[] {\n",
                "const normalize = (item: T) => ({ ...item, id: item.id.trim() });\n",
            ],
        }],
    },
    StressFixture {
        language: Language::Yaml,
        fixture: "tests/fixtures/stress/source.yaml",
        syntax_only: false,
        map_needles: &[
            " defaults ",
            " service ",
            " service.routes.path ",
            " jobs.name ",
            " metadata.owner ",
        ],
        absent_map_needles: &[],
        show_cases: &[ShowCase {
            keys: &["service.routes"],
            needles: &[
                "# service.routes@",
                "routes:\n",
                "handler: { name: root, cache: true }\n",
            ],
        }],
    },
];

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
fn language_detects_kotlin_files_and_scripts() {
    for path in ["Main.kt", "build.gradle.kts"] {
        assert_eq!(Language::from_path(Path::new(path)), Some(Language::Kotlin));
    }

    assert_eq!(Language::from_token("kotlin"), Some(Language::Kotlin));
}

#[test]
fn language_detects_svelte_files() {
    assert_eq!(
        Language::from_path(Path::new("Widget.svelte")),
        Some(Language::Svelte)
    );
    assert_eq!(Language::from_token("svelte"), Some(Language::Svelte));
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

    let stdout = run_lupa_stdin(
        &["map", "kotlin"],
        "class Widget {\n    fun render(): String = \"ok\"\n}\n",
    );
    assert_stdout_contains(&stdout, "# - [kotlin] 3L 49B 2S\n");
    assert_stdout_contains(&stdout, " Widget class Widget\n");
    assert_stdout_contains(&stdout, " Widget.render fun render(): String\n");

    let stdout = run_lupa_stdin(
        &["map", "svelte"],
        "<script>\n    let count = 0;\n</script>\n\n<button>{count}</button>\n",
    );
    assert_stdout_contains(&stdout, "# - [svelte] 5L 64B 2S\n");
    assert_stdout_contains(&stdout, " script <script>\n");
    assert_stdout_contains(&stdout, " button <button>{count}</button>\n");
}

#[test]
fn svelte_raw_text_ampersand_does_not_warn() {
    let stdout = run_lupa_stdin(
        &["map", "svelte"],
        "<button><MessageSquarePlus size={14} /> Edit & fork</button>\n",
    );

    assert_stdout_contains(&stdout, "# - [svelte] ");
    assert_stdout_contains(
        &stdout,
        " button <button><MessageSquarePlus size={14} /> Edit & fork</button>\n",
    );
    assert_stdout_lacks(&stdout, "parse error");
}

#[test]
fn svelte_structural_parse_error_still_warns() {
    let stdout = run_lupa_stdin(
        &["map", "svelte"],
        "<button><MessageSquarePlus size={14} /> Edit & {</button>\n",
    );

    assert_stdout_contains(&stdout, "# - [svelte] ");
    assert_stdout_contains(
        &stdout,
        "# warning: parse error at L1: parse error in ERROR\n",
    );
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
        "tests/fixtures/digest_tree/visible.kt",
        "tests/fixtures/digest_tree/visible.kts",
        "tests/fixtures/digest_tree/justfile",
        "tests/fixtures/digest_tree/visible.nix",
        "tests/fixtures/digest_tree/visible.py",
        "tests/fixtures/digest_tree/visible.svelte",
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
    assert_stdout_contains(&stdout, "visible.kt [kotlin]");
    assert_stdout_contains(&stdout, "visible.kts [kotlin]");
    assert_stdout_contains(&stdout, "visible.proto [proto]");
    assert_stdout_contains(&stdout, "visible.svelte [svelte]");
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
            SVELTE_FIXTURE,
            &[
                " script ",
                " section ",
                " section.h1 ",
                " section.Widget ",
                " section.if ",
                " section.if.each ",
                " section.await ",
                " section.key ",
                " section.summary ",
                " section.render ",
                " section.const ",
                " style ",
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
        (
            KOTLIN_SCRIPT_FIXTURE,
            &[" plugins ", " endpoint ", " android ", " dependencies "][..],
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
            SVELTE_FIXTURE,
            &["script", "section.if.each", "section.await", "style"][..],
            &[
                "# script@L1-L5\n",
                "<script lang=\"ts\">\n",
                "# section.if.each@L11-L13\n",
                "{#each items as item (item.id)}\n",
                "# section.await@L17-L21\n",
                "{:then result}\n",
                "# style@L32-L36\n",
                "<style>\n",
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
            SVELTE_FIXTURE,
            &[
                "script L1-L5\n",
                "section L7-L30\n",
                "section.if.each.Row L12\n",
                "section.await.p#2 L20\n",
                "section.summary.span L26\n",
                "style L32-L36\n",
            ][..],
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

#[test]
fn language_stress_fixtures_cover_supported_languages() {
    let languages: Vec<_> = STRESS_FIXTURES
        .iter()
        .map(|fixture| fixture.language)
        .collect();
    assert_eq!(languages.as_slice(), EXPECTED_STRESS_LANGUAGES);
    for (index, fixture) in STRESS_FIXTURES.iter().enumerate() {
        for previous in &STRESS_FIXTURES[..index] {
            assert_ne!(
                fixture.language, previous.language,
                "duplicate stress fixture language: {}",
                fixture.language
            );
        }
    }

    for fixture in STRESS_FIXTURES {
        let stdout = run_lupa(&["map", fixture.fixture]);
        let header = format!("# {} [{}] ", fixture.fixture, fixture.language);
        assert!(
            stdout.contains(&header),
            "{} stress fixture missing header {header:?}\nstdout:\n{stdout}",
            fixture.language
        );
        assert!(
            !stdout.contains("parse error"),
            "{} stress fixture parsed with errors: {}\nstdout:\n{stdout}",
            fixture.language,
            fixture.fixture
        );
        if fixture.syntax_only {
            assert!(
                stdout.contains("# warning: syntax-only adapter: top-level syntax nodes only\n"),
                "{} syntax-only stress fixture missing warning: {}\nstdout:\n{stdout}",
                fixture.language,
                fixture.fixture
            );
        } else {
            assert!(
                !stdout.contains("syntax-only adapter"),
                "{} typed stress fixture unexpectedly used syntax-only adapter: {}\nstdout:\n{stdout}",
                fixture.language,
                fixture.fixture
            );
        }
        for needle in fixture.map_needles {
            assert!(
                stdout.contains(needle),
                "{} stress fixture {} missing map needle {needle:?}\nstdout:\n{stdout}",
                fixture.language,
                fixture.fixture
            );
        }
        for needle in fixture.absent_map_needles {
            assert!(
                !stdout.contains(needle),
                "{} stress fixture {} unexpectedly contained map needle {needle:?}\nstdout:\n{stdout}",
                fixture.language,
                fixture.fixture
            );
        }

        for show_case in fixture.show_cases {
            let mut args = vec!["show", fixture.fixture];
            args.extend_from_slice(show_case.keys);
            let stdout = run_lupa(&args);
            for needle in show_case.needles {
                assert!(
                    stdout.contains(needle),
                    "{} stress fixture {} missing show needle {needle:?} for keys {:?}\nstdout:\n{stdout}",
                    fixture.language,
                    fixture.fixture,
                    show_case.keys
                );
            }
        }
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
