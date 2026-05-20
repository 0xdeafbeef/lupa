# Lupa Specification

## 1. Purpose

`lupa` is a source navigation CLI for coding agents.

The primary job is to let an agent inspect a codebase with fewer tokens and fewer failed follow-up commands than plain `cat`, `sed`, or ad hoc grep. It should return compact structural views, exact source slices, and copy-paste symbol keys that can be passed back to the tool.

This is not a general search engine in v1. It is a deterministic source map and source slice tool.

## 2. Evidence From Prior Usage

The intended user is an agent, not a human typing commands manually.

Observed Codex history under `~/.codex/sessions/**/*.jsonl` showed thousands of `ast-outline` invocations by agents:

- `show`: most common path.
- `map`: heavily used.
- `digest`: used for directory scouting.
- `search`, `implements`, and `find-related`: rare.
- `map --json`: effectively not used for routine navigation.

Therefore v1 optimizes for text output that is easy for agents to read and reuse. JSON, semantic search, dependency graph commands, and MCP integration are not part of v1 unless a concrete consumer appears.

## 3. Design Principles

1. Text is the machine interface.
   Output must be deterministic, compact, and copy-pasteable. Do not rely on JSON for the normal agent loop.

2. Every navigable item has a key.
   If `map` shows a symbol, it must also show the exact key accepted by `show`.

3. Source output is compact by default.
   `show` prints source lines without added prefixes and strips common leading indentation from the selected range.

4. Failed commands should teach the next command.
   No-match output must include likely valid keys when possible.

5. Future write commands must be stricter than read commands.
   v1 is read-only. If edit support is added later, it needs a separate stale-input protection design.

6. No hidden repo mutation in v1.
   v1 commands must not create repo-local indexes or cache directories. Any future cache must be explicit.

## 4. Command Summary

```text
lupa map <file-or-dir>...          # structural file map
lupa show <file> <key>...          # source slices by symbol key
lupa digest <dir>...               # compact directory/module overview
lupa keys <file>                   # print only show keys and ranges
lupa help                          # command help
```

Non-goals for v1:

```text
lupa search
lupa find-related
lupa graph
lupa deps
lupa --json
lupa mcp
lupa edit
```

`edit` may be designed later, but `lupa` v1 should first prove the read-side navigation model.

## 5. Core Data Model

The internal model must separate parser facts from presentation facts.

```rust
struct FileMap {
    path: PathBuf,
    language: Language,
    line_count: u32,
    byte_count: u64,
    parse_errors: Vec<ParseError>,
    symbols: Vec<Symbol>,
}

struct Symbol {
    key: SymbolKey,
    kind: SymbolKind,
    name: String,
    signature: String,
    visibility: Option<String>,
    range: LineSpan,
    body_range: Option<LineSpan>,
    parent_key: Option<SymbolKey>,
    children: Vec<Symbol>,
}

struct SymbolKey(String);

struct LineSpan {
    start_line: u32,
    end_line: u32,
}
```

Parser adapters produce `Symbol` data. Renderers never infer keys from display strings. `key` is a first-class field.

## 6. Symbol Keys

Symbol keys are the stable command surface for `show`.

Examples:

```text
Person
Person.new
Person.hello
Storage.get
module.submodule.function_name
README.Install
```

Rules:

- `map` must print each symbol key as its own whitespace-delimited field.
- `show <file> <key>` must accept exactly the keys printed by `map`.
- Unique suffixes are allowed for convenience, but ambiguity must be reported.
- Ambiguous suffix errors must list matching full keys.
- Keys must be deterministic for a file content snapshot.
- Keys must not be derived from formatted signatures.

Example ambiguity:

```text
# amb new
# Person.new@L10-L18 pub fn new() -> Self
# Session.new@L44-L55 pub fn new() -> Self
```

## 7. Line Ranges

Line ranges are plain one-based source line ranges.

Format:

```text
L<start>-L<end>
```

Examples:

```text
L10
L10-L18
```

Rules:

- Single-line symbols use `L10`.
- Multi-line symbols use `L10-L18`.
- Line ranges are for navigation and diagnostics, not stale-change protection.
- `show` by symbol key is the stable v1 command surface.

## 8. `map`

`map` prints file structure without method bodies.

Usage:

```bash
lupa map src/lib.rs
lupa map crates/foo/src crates/bar/src
lupa src/lib.rs
```

Output format:

```text
# src/lib.rs [rust] 240L 8120B 19S
L10-L18 Person pub struct Person
  L11 Person.name pub name: String
  L22-L28 Person.new pub fn new(name: String) -> Self
  L30-L34 Person.hello fn hello(&self) -> String
L70-L96 parse_config fn parse_config(path: &Path) -> Result<Config>
```

Requirements:

- Include language, line count, byte count, and symbol counts in the header.
- Print parse warnings immediately after the header.
- Print exact keys accepted by `show`.
- Print line ranges for every symbol.
- Default output includes private symbols because agents often need implementation detail.
- Do not add readability-only blank lines between top-level symbols.

Markdown behavior:

- Headings are symbols.
- Heading keys are based on heading text.
- Duplicate headings get deterministic suffixes, e.g. `Install#2`.
- Code blocks are not symbols in v1 unless they have a stable derived key.

## 9. `show`

`show` prints source for one or more symbol keys.

Usage:

```bash
lupa show src/lib.rs Person.new
lupa show src/lib.rs Person.new Person.hello
```

Output format:

```text
# Person.new@L22-L28
pub fn new(name: String) -> Self {
    Self { name }
}
```

Requirements:

- Multiple requested symbols are supported in one command.
- Output sections are separated by compact `# key@range` headers.
- Source lines have no added line-number or separator prefix.
- `show` strips common leading indentation from the selected range to reduce repeated whitespace tokens.
- The section header range is the line anchor; use `sed -n '<range>p' <file>` or `nl -ba <file> | sed -n '<range>p'` when exact per-line citations are needed.
- If a key is missing, print a no-match diagnostic and close candidates.
- If a key is ambiguous, print all matching full keys and do not guess.
- The command already names the file; do not repeat the file path or symbol kind in normal `show` headers.

No-match example:

```text
# no PipelineExecutor.execute
# candidates
# execute@L120-L244 fn execute(...)
# impl_PipelineExecutor.execute@L120-L244 fn execute(...)
```

## 10. `digest`

`digest` is for scouting unfamiliar directories.

Usage:

```bash
lupa digest crates/fleet-core/src
lupa digest src
```

Output format:

```text
crates/fleet-core/src/executor.rs [rust] 920L 54S PipelineExecutor@L20-L80[execute,validation_hosts_for_queue_worktree,run_step][+6] load_executor_config@L120-L244
crates/fleet-core/src/store.rs [rust] 710L 38S SqliteStateStore@L10-L90[open,load_latest_report,save_pipeline_steps]
```

Requirements:

- One compact line per file.
- Show top-level symbols and capped child lists.
- Use fixed default caps and print explicit `+N` truncation markers.
- Include parse warning counts.
- Do not print method bodies.
- Do not recurse into ignored/generated/vendor directories by default.

## 11. `keys`

`keys` prints only navigation keys and ranges.

Usage:

```bash
lupa keys src/lib.rs
```

Output:

```text
Person L10-L18
Person.name L11
Person.new L22-L28
Person.hello L30-L34
parse_config L70-L96
```

This is useful after `show` reports ambiguity or no-match.

## 12. Supported Languages

v1 should support the languages that showed up in agent guidance and normal repo work:

- Rust
- Python
- TypeScript / TSX
- JavaScript / JSX
- Go
- C / C++ / headers
- Markdown

Later:

- Java
- Kotlin
- Scala
- C#
- Ruby
- PHP
- SQL

Language adapters must be additive. A weak adapter is acceptable only if it clearly reports limited support.

## 13. Parser Adapter Contract

Each adapter must provide:

```rust
trait LanguageAdapter {
    fn language(&self) -> Language;
    fn parse(&self, path: &Path, source: &[u8]) -> FileMap;
}
```

Adapter obligations:

- Produce deterministic `Symbol.key`.
- Produce accurate `signature`.
- Produce `range` for the full declaration.
- Produce `body_range` when available.
- Preserve source line numbers.
- Report parse errors instead of silently omitting large regions.

Adapter output must not depend on renderer behavior.

## 14. Ignore Rules

Default ignored directories:

```text
.git
.jj
target
node_modules
vendor
dist
build
.next
.turbo
.cache
__pycache__
```

The tool may support `.lupaignore` later. v1 can rely on default ignores plus explicit paths.

## 15. Error Handling

All user-facing failures should be concise and recoverable.

Rules:

- Missing path: exit 0 with `# error:` line for agent flow compatibility.
- Unsupported file type: exit 0 with `# error:` line.
- Internal bug: exit non-zero.
- Parse errors: exit 0, print warning, return partial output if possible.

The CLI should avoid panic output in normal failure modes.

## 16. Output Stability

Output is part of the contract.

Rules:

- Headers start with `#`.
- Machine-reusable values are whitespace-delimited when possible; avoid `key=value` unless disambiguation needs it.
- Successful `show` source lines have no added prefix; locations live in the `# key@range` header.
- Ranges use `L<start-line>-L<end-line>`.
- No ANSI color by default when stdout is not a TTY.
- `NO_COLOR=1` disables color.
- Do not wrap source lines.

## 17. Performance Targets

Baseline targets on a warm filesystem:

- `lupa map <single 1000-line Rust file>`: under 100 ms.
- `lupa show <single symbol>`: under 50 ms.
- `lupa digest <100 source files>`: under 1 s.

No persistent index is required for v1. Parallel directory walking is allowed.

## 18. Implementation Plan

Preferred implementation language: Rust.

Suggested crates:

- `clap` for CLI parsing.
- `ignore` for walking and ignore rules.
- `tree-sitter` parsers or `ast-grep` for language parsing.
- `similar` only for tests or future relocation diagnostics if needed.

Initial modules:

```text
src/main.rs
src/cli.rs
src/model.rs
src/render.rs
src/walk.rs
src/adapters/mod.rs
src/adapters/rust.rs
src/adapters/python.rs
src/adapters/typescript.rs
src/adapters/markdown.rs
```

## 19. Rust Style Gate

The initial Rust scaffold should use strict linting and rustfmt settings.

`rustfmt.toml`:

```toml
format_code_in_doc_comments = true
imports_granularity = "Module"
normalize_comments = true
overflow_delimited_expr = true
group_imports = "StdExternalCrate"
```

Root `Cargo.toml` must define workspace lints:

```toml
[workspace.lints.rust]
future_incompatible = "warn"
nonstandard_style = "warn"
rust_2018_idioms = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
await_holding_lock = "warn"
char_lit_as_u8 = "warn"
checked_conversions = "warn"
dbg_macro = "warn"
debug_assert_with_mut_call = "warn"
disallowed_methods = "warn"
doc_markdown = "warn"
empty_enums = "warn"
enum_glob_use = "warn"
exit = "warn"
expl_impl_clone_on_copy = "warn"
explicit_deref_methods = "warn"
explicit_into_iter_loop = "warn"
fallible_impl_from = "warn"
filter_map_next = "warn"
flat_map_option = "warn"
float_cmp_const = "warn"
fn_params_excessive_bools = "warn"
from_iter_instead_of_collect = "warn"
if_let_mutex = "warn"
implicit_clone = "warn"
imprecise_flops = "warn"
inefficient_to_string = "warn"
invalid_upcast_comparisons = "warn"
large_digit_groups = "warn"
large_futures = "warn"
large_stack_arrays = "warn"
large_types_passed_by_value = "warn"
let_unit_value = "warn"
linkedlist = "warn"
lossy_float_literal = "warn"
macro_use_imports = "warn"
manual_ok_or = "warn"
map_err_ignore = "warn"
map_flatten = "warn"
map_unwrap_or = "warn"
match_same_arms = "warn"
match_wild_err_arm = "warn"
match_wildcard_for_single_variants = "warn"
mem_forget = "warn"
missing_enforced_import_renames = "warn"
mut_mut = "warn"
mutex_integer = "warn"
needless_borrow = "warn"
needless_continue = "warn"
needless_for_each = "warn"
option_option = "warn"
path_buf_push_overwrite = "warn"
ptr_as_ptr = "warn"
print_stdout = "warn"
print_stderr = "warn"
rc_mutex = "warn"
ref_option_ref = "warn"
rest_pat_in_fully_bound_structs = "warn"
same_functions_in_if_condition = "warn"
semicolon_if_nothing_returned = "warn"
string_add_assign = "warn"
string_add = "warn"
string_lit_as_bytes = "warn"
todo = "warn"
trait_duplication_in_bounds = "warn"
unimplemented = "warn"
unnested_or_patterns = "warn"
unused_self = "warn"
useless_transmute = "warn"
verbose_file_reads = "warn"
zero_sized_map_values = "warn"
```

Every crate in the workspace must opt in:

```toml
[lints]
workspace = true
```

Verification commands:

```bash
just fmt-check
just clippy
just test
just check
```

`just fmt-check` and `just fmt` intentionally use `cargo +nightly fmt`
because the rustfmt settings rely on unstable rustfmt options.

If a lint is too noisy during bootstrap, prefer fixing the design or narrowing the code. Do not add broad `allow` attributes without a specific rationale in code.

## 20. Acceptance Tests

Minimum v1 tests:

1. `map` prints exact keys accepted by `show`.
2. `show` accepts multiple keys.
3. `show` prints source lines without added line-number prefixes.
4. Ambiguous suffix reports all candidate keys.
5. Markdown duplicate headings get deterministic keys.
6. Parse error warning appears when a parser reports partial output.
7. Direct file invocation aliases to `map`.
8. Directory `digest` skips ignored directories.
9. `just fmt-check` passes.
10. `just clippy` passes.

## 21. Future Work

Possible later features:

- Anchor-based `lupa edit` if read-side keys are not enough.
- Explicit `--json` if a real consumer appears.
- Semantic search after deterministic navigation is solid.
- Dependency graph commands.
- MCP integration.
- Repo-local cache with explicit `lupa index`, never implicit.

Future edit command shape:

```bash
lupa edit src/lib.rs --replace-key Person.new --with /tmp/new-body.txt
```

Edit commands need a separate stale-input protection design before implementation.
