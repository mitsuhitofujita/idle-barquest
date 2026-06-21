# ADR 0010: Split `barquest-tui` into modules with an App/Input layer

- Status: Accepted
- Date: 2026-06-21

## Context

`crates/tui/src/main.rs` had grown to ~844 lines — the terminal lifecycle, the
wall-clock frame loop, input handling, the five-region render dispatch, and the
two larger render regions (progress rows, user choices) all in one file, with
every test appended at the bottom. `docs/design-docs/tui-test-policy.md` had
already named the exit criterion in its "Near-Term Recommendations": extract an
`App` type, move progression into `App::update`, keep `run()` thin, and split
input into a *translate* layer and a *behaviour* layer. It had outgrown one file,
mirroring [ADR 0009](0009-core-module-structure.md) for `core`.

## Decision

Split `tui` along its concerns into focused modules:

- `input` — internal `Input { Quit, Select(usize), Ignored }` enum and
  `translate(&Event) -> Input` (the former `is_quit` / `digit_index`,
  `crossterm` translation only — no game state).
- `app` — `App { catalog, state, menu, log }`, the `Menu` enum, `App::update`
  (behaviour: applies one `Input`, returns "should quit"), `App::advance`
  (steps the world, logs completion events), and `push_event`. Reads neither the
  wall clock nor the terminal.
- `render` — the pure projection from `&App` to a `TestBackend`-able buffer: the
  five-region layout, ASCII chrome, `separator`/`log_lines`/`fit`, with the two
  large regions in submodules `render::progress` and `render::choices`.

`main.rs` keeps only the crate docs, module declarations, the `FRAME` constant,
and a thin `run()` loop: draw → poll → `app.update(input::translate(event))` →
`app.advance(TICKS_PER_FRAME)`. Each module owns its own `#[cfg(test)] mod tests`.

## Consequences

- No behaviour change: the layout contract from
  [ADR 0008](0008-full-screen-tui-layout-and-events.md) and the frame scheduling
  from [ADR 0006](0006-concurrent-multi-target-progress-loop.md) are untouched
  (verified by `just check`). `main.rs` shrank 844 → 74 lines.
- Module dependencies stay acyclic: `render` → `app`, `input` → (none in-crate),
  `app` → `input`; `main` wires all three. `app` and `input` have no terminal or
  clock dependency, so behaviour and translation are unit-tested directly.
- Tests now live next to the code they cover; coverage grew while moving
  (input translation for quit/digit/release/unknown keys, an `App::update`
  quit-signal test, and an `App::advance` completion-and-clear test driven by
  injected ticks): tui tests 23 → 29.

## Related

- Enacts recommendations 1–5 of `docs/design-docs/tui-test-policy.md` (now
  marked Accepted); the App/Input/render seam is the shape that document
  prescribed.
- Mirrors [ADR 0009](0009-core-module-structure.md): split once a crate outgrows
  a single file, tests beside their code, public/observable behaviour unchanged.
