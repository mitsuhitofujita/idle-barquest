# idle-barquest

A terminal idle RPG where every quest is a progress bar.

## Development

This repo is a Cargo workspace. Open it in the provided dev container
(`.devcontainer/`), which ships Rust 1.96 with `clippy`, `rustfmt`, and `just`.

### Layout

| Crate            | Path           | Role                                            |
| ---------------- | -------------- | ----------------------------------------------- |
| `barquest-core`  | `crates/core`  | Pure game logic (no terminal I/O)               |
| `barquest-tui`   | `crates/tui`   | The game binary (`barquest`); ratatui front-end |
| `barquest-tools` | `crates/tools` | Game-data & dev tools (`src/bin/`)              |

### Common tasks

Recipes are defined in the `Justfile`:

```sh
just run     # run the game        (cargo run -p barquest-tui)
just test    # run all tests       (cargo test --workspace)
just lint    # clippy, deny warns  (cargo clippy --workspace --all-targets -- -D warnings)
just fmt     # format all crates   (cargo fmt --all)
just check   # fmt-check + lint + test
```

Run a tool binary with `just tool <name>`, e.g. `just tool balance-sim`.
