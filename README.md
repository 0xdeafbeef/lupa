# Lupa

`lupa` is a small source-navigation CLI for coding agents.

It is a personal reimplementation of the parts of `ast-outline` that were
actually useful in my own agent workflow: compact maps, source slices by stable
keys, directory digests, key/range listings, and semantic context for `rg` hits.

Notice: this project is vibcoded. Expect sharp edges; trust the tests more
than the implementation style.

## Install

```bash
cargo install --git https://github.com/0xdeafbeef/lupa --locked
```

## AGENTS.md

Add this to your agent instructions:

```markdown
## Code exploration — prefer `lupa` over full reads

Use `lupa` to read structure for programming-language, config, and large
markdown files before opening full contents. For unsupported files, short docs,
or known line ranges, use `rg` with targeted source slices.

If `lupa` lacks language support or gives poor structure, fall back to targeted
reads. If the map header contains `# WARNING: N parse errors`, treat the map as
partial and read the affected raw source directly before relying on it.

Stop at the step that answers the question:

1. **Unfamiliar directory** — `lupa digest <dir>`: Capped compact map with one
   line per file, top-level keys, and capped child key lists.
2. **One file's shape** — `lupa map <file>`: Range, key, and signature lines, no
   bodies.
3. **One method, class, or section** — `lupa show <file> <key>...`: Copy keys
   from `lupa map`. Unique suffixes and relaxed parent suffixes are accepted
   when unambiguous. If output shows `# no` or `# amb`, retry with a full key
   listed in the candidates.
4. **Keys only** — `lupa keys <file>`: Use if map output is too noisy.
5. **Search hit context** — `rg -nH <pattern> <path> | lupa context` or
   `lupa context <path>:<line>`: Converts hits to enclosing symbol keys,
   parents, and siblings.

### Special Inputs & Constraints
- **Piped source/stdin:** Pass the language token instead of a path:
  `jj file show -r <rev> <path> | lupa map rust`. Stdin language mode is only
  for `map`, `show`, and `keys`; do not use it for `digest` or `context`.
- **Indentation warning:** `lupa show` strips common leading indentation. Before
  using `apply_patch` on indentation-sensitive code, re-read raw lines with
  `sed -n '<start>,<end>p' <file>`.
- **Markdown keys:** Markdown keys are heading text; duplicate headings get
  deterministic `#2` suffixes.
```

## Commands

```text
lupa map <file-or-dir>
lupa map <language>        # read source from stdin
lupa show <file> <key>...
lupa show <language> <key>...
lupa digest <dir>
lupa keys <file>
lupa keys <language>
lupa context <path:line>...
```

## Eval

The `eval/` directory contains Codex sub-agent A/B runners that compare
repository-understanding tasks with and without `lupa`. Tasks ask for
files-only evidence, not exact line citations, because line citation polishing
otherwise dominates the result.

Example runs:

```bash
./eval/run.py lupa --repo /path/to/lupa
./eval/analyze.py /tmp/lupa-codex-eval-targeted/summary.json

./eval/run.py codex --repo /path/to/codex
./eval/analyze.py /tmp/lupa-codex-eval-codex-repo/summary.json
```

Pilot results, using `without_lupa` as the baseline:

| Target | Wall | Input | Uncached input | Output |
| --- | ---: | ---: | ---: | ---: |
| `lupa` repo targeted tasks | 15.5% faster (222.3s vs 262.9s) | 8.0% less (567,064 vs 616,117) | 11.9% less (170,136 vs 193,077) | 5.2% less (6,447 vs 6,803) |
| Codex repo feature-flow tasks | 7.4% faster (359.5s vs 388.2s) | 16.0% less (1,209,178 vs 1,438,839) | 4.8% less (281,562 vs 295,671) | 6.5% less (9,941 vs 10,635) |

The `lupa` repo tasks check whether an agent can explain local source-navigation
behavior from a small named file set. The Codex repo tasks check larger
feature-flag/config flows in a big Rust workspace.

Checked tasks:

| Target | Task | What it checks |
| --- | --- | --- |
| `lupa` repo targeted tasks | `show_flow` | `lupa show` CLI dispatch through source rendering |
| `lupa` repo targeted tasks | `stdin_contract` | stdin language mode commands, token recognition, and invalid multi-arg behavior |
| `lupa` repo targeted tasks | `relaxed_matching` | relaxed `lupa show` key matching, exact-match priority, and ambiguity tests |
| `lupa` repo targeted tasks | `digest_behavior` | `lupa digest` supported-file selection, path skipping, and compact summaries |
| Codex repo feature-flow tasks | `cli_feature_toggle_flow` | feature toggles from CLI parsing through overrides or persisted enable/disable state |
| Codex repo feature-flow tasks | `web_search_feature_flow` | web search mode resolution from config, legacy flags, and permission-profile fallback |
| Codex repo feature-flow tasks | `multi_agent_v2_config_flow` | `features.multi_agent_v2` loading, validation, runtime config, and config-lock output |
| Codex repo feature-flow tasks | `managed_feature_requirements_flow` | feature requirement normalization, pinned values, and legacy or unknown requirement warnings |

`Uncached input` is `input_tokens - cached_input_tokens` from Codex
`turn.completed` usage events. The Codex repo run was parallel, so its wall
number is a per-job sum; elapsed time was about the slowest job. The strongest
pattern was that targeted `lupa map` plus `lupa show` helped, while mapping a
huge file such as `codex-rs/cli/src/main.rs` could lose to narrow `rg`.

Run local checks with:

```bash
just check
```
