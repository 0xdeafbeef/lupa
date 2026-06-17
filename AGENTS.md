# AGENTS.md

## Tooling

- This repo uses `jj`; do not use Git workflow commands for local history.
- Prefer `just` recipes over raw tool commands whenever a matching recipe exists.
- Use `just fmt` for formatting, `just fmt-check` for format checks, `just test` for tests, `just clippy` for Clippy, and `just check` for the full local verification gate.
- Run `just --list` before suggesting or using a recipe you have not already verified in this session.
- Keep changes narrow and remove stale tests, fixtures, imports, and dependency features when removing behavior.

## Lupa Conventions

- `lupa` is an agent-facing source navigation CLI. Preserve deterministic text output and copy-pasteable keys.
- Add or keep parser support only when realistic fixtures produce useful structure without parse-error warnings. Do not keep weak fallback parsers that only work on toy snippets.
- For broad parser-backed language support, prefer explicit `Language` mappings and focused fixtures over auto-detection or large registries.
