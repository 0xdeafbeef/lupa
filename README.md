# Lupa

`lupa` is a small source-navigation CLI for coding agents.

It is a personal reimplementation of the parts of `ast-outline` that were
actually useful in my own agent workflow: compact maps, source slices by stable
keys, directory digests, and key/range listings.

Notice: this project is vibcoded. Expect sharp edges; trust the tests more
than the implementation style.


```text
lupa map <file-or-dir>
lupa show <file> <key>...
lupa digest <dir>
lupa keys <file>
```

Run local checks with:

```bash
just check
```
