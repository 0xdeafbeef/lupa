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

For `.rs`, `.py`, `.ts`, `.tsx`, `.js`, `.jsx`, `.go`, `.c`, `.cc`, `.cpp`,
`.cxx`, `.h`, `.hh`, `.hpp`, `.hxx`, and large `.md` files, read structure with
`lupa` before opening full contents. For unsupported languages, short docs, or
known line ranges, use `rg` plus targeted source slices directly.
Pull method bodies only once you know which ones you need.

Stop at the step that answers the question:

1. **Unfamiliar directory** — `lupa digest <dir>`: capped compact map.
2. **One file's shape** — `lupa map <file>`: compact signatures with line ranges.
3. **One symbol or markdown section** — `lupa show <file> <key>...`.
   Copy keys exactly from `lupa map`; multiple keys can be shown at once.
4. **Only accepted keys and ranges** — `lupa keys <file>`.
5. **Search hit context** — `rg -nH pattern src | lupa context`.
   Converts raw `path:line` hits to enclosing symbol keys, parents, and siblings.

Fall back to a full read only when you need context beyond the body `show`
returned, or when `lupa` gives poor structure for that file or language.
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

Run local checks with:

```bash
just check
```
