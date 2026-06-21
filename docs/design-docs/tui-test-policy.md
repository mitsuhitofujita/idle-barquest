# Test Policy for Active TUI Gameplay

- Status: Proposal
- Date: 2026-06-20

## Purpose

`idle-barquest` is a terminal idle RPG. The screen updates even when the player
does nothing: game state advances on ticks, and the TUI redraws progress bars on
a frame cadence. This policy defines how to test that kind of active TUI without
making tests slow, flaky, or dependent on a real terminal.

## Principles

1. Keep game logic deterministic and terminal-free.
2. Test time by injecting it, not by sleeping.
3. Treat rendering as a pure projection from state to a terminal buffer.
4. Use end-to-end terminal tests sparingly, only for integration risk that unit
   tests cannot cover.
5. Prefer stable semantic assertions over brittle full-screen snapshots.

The current architecture already supports the first principle: `barquest-core`
owns pure game state and `barquest-tui` owns wall-clock pacing, input, and
rendering.

## Test Pyramid

### 1. Core Unit Tests

Core tests are the default place for gameplay behavior.

Scope:

- Tick arithmetic and progress saturation.
- Completion, rewards, inventory, stats, unlocks, and future save/load behavior.
- Menu domain rules such as unique hotkeys.
- Balance invariants that should hold across many inputs.

Best practices:

- Keep `barquest-core` free of wall-clock reads, terminal I/O, random global
  state, and sleeps.
- Express time as ticks or explicit durations passed into functions.
- Add property-style tests when domain rules grow, for example "progress never
  exceeds the goal" or "offline elapsed time produces the same result as many
  small advances".
- Put broad simulations in `barquest-tools` when they are useful for design, but
  keep crisp assertions in normal `cargo test`.

### 2. App Model Tests

As the TUI grows, introduce a small testable app layer in `barquest-tui`.

Recommended shape:

```text
input event + elapsed ticks -> App::update() -> new App state + command
App state -> render() -> terminal buffer
```

This layer should not read from `crossterm::event`, call `Instant::now`, or draw
directly to the real terminal. Those effects should stay at the edge of the
binary.

Scope:

- Menu transitions: target selection, action selection, and assigning or
  restarting a target's quest.
- Quit behavior from every screen.
- Ignoring key-release and repeat events when only key presses should count.
- Progressing several targets' active quests concurrently by elapsed ticks.
- Idle targets showing no progress row, and a finished quest being removed and
  reported as a completion event (logged) rather than left on screen.

Best practices:

- Define a small internal input enum instead of testing directly against every
  `crossterm` detail.
- Use a fake clock or pass elapsed ticks/durations into update functions.
- Make tests drive multiple frames instantly by calling update repeatedly.
- Do not assert on real elapsed time.

### 3. Renderer Tests

Ratatui rendering can be tested without a real terminal by drawing into a
`ratatui::backend::TestBackend`.

Scope:

- The right screen is rendered for each app state.
- Important text is present: menu title, choices, quit hint, quest title,
  percent label, tick details.
- Layout remains usable at representative terminal sizes.
- Progress gauges render expected labels at 0%, partial progress, and 100%.

Best practices:

- Prefer semantic assertions such as "buffer contains `Select target`" or
  "buffer contains `50%`" over comparing the entire buffer.
- Use full-buffer snapshots only for stable, high-value screens. Full snapshots
  are useful, but they are noisy when copy or layout changes intentionally.
- Test a small size, a normal size, and a wide size for important screens.
- Keep renderer functions pure: `fn render(frame, app_state)` should not mutate
  game state or read input.

Suggested helper:

```rust
fn render_to_string(width: u16, height: u16, draw: impl FnOnce(&mut Frame<'_>)) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(draw).unwrap();
    terminal.backend().buffer().to_string()
}
```

The exact helper can live in `crates/tui/src/...` test modules or in a shared
test support module once more screens exist.

### 4. Terminal Integration Tests

Use real process or pseudo-terminal tests only for behavior that depends on the
terminal boundary.

Scope:

- The binary starts, switches the terminal into the expected mode, and restores
  it on exit.
- Basic input flow works through the real executable.
- Panic/error paths restore terminal state.

Best practices:

- Keep these tests few and clearly marked.
- Avoid long-running real-time progress tests. Prefer a test build flag or app
  configuration that makes frame duration and quest length tiny.
- If using a pseudo-terminal harness later, assert only key visible milestones
  and exit status.
- Do not make CI depend on a user's terminal capabilities.

## Testing Active Output

Active TUI output is tricky because the screen is repeatedly overwritten. Tests
should therefore observe state transitions and deterministic render buffers, not
the live stream of escape sequences.

Recommended approach:

1. Advance the app with explicit elapsed ticks or fake clock time.
2. Render one frame into `TestBackend`.
3. Assert important content in the resulting buffer.
4. Repeat for the next meaningful state.

For example, a quest progress test should check:

- After selecting `Hero` and `Forest Exploration`, the hero runs one quest while
  the other targets stay idle (no progress row).
- After 5 seconds of game time, the renderer shows roughly `50%`.
- After 10 seconds of game time, `advance` emits a `QuestCompleted` event, the
  quest is removed (its row disappears), and a completion line shows in the log.
- Any other started target's quest keeps progressing.

No part of this test should sleep for 10 real seconds.

## Handling Time

The project currently distinguishes game-time ticks from TUI frames:

- `core` tick: atomic game time, `TICKS_PER_SECOND = 1000`.
- TUI frame: render/update cadence, currently 100 ms.

Keep this distinction in tests.

Policy:

- Test `core` in ticks.
- Test app behavior with explicit elapsed ticks or `Duration` values.
- Test frame scheduling separately from gameplay progression.
- If drift compensation is important, isolate it behind a small scheduler type
  and test it with a fake clock.

Avoid:

- `std::thread::sleep` in normal tests.
- Assertions based on "it should finish within N milliseconds" unless the test
  is explicitly a performance or smoke test.
- Tests that wait for real progress bars to fill.

## Input Testing

Input handling should be split into two layers:

- Translation: `crossterm::event::Event` -> internal app input.
- Behavior: internal app input -> state transition.

Translation tests should cover:

- `q`, `Esc`, and `Ctrl-C` quit.
- Non-press events are ignored.
- First-letter hotkeys are lowercased.
- Unknown keys are ignored.

Behavior tests should cover:

- Valid menu choices select the expected target/action.
- Invalid choices leave the current screen unchanged.
- Quit works from menus and quest execution.

This split keeps most tests independent of `crossterm` details while preserving
coverage for the boundary where terminal events enter the app.

## Snapshot Policy

Snapshots are allowed for TUI rendering, but use them intentionally.

Good snapshot candidates:

- Stable menu screens.
- Stable quest screen at 0%, 50%, and 100%.
- Error or empty-state screens once they exist.

Snapshot rules:

- Keep snapshot dimensions fixed and named, for example `80x24`.
- Review snapshot diffs as UI changes, not as golden truth that cannot move.
- Pair snapshots with semantic assertions for critical text and state.
- Avoid snapshots for rapidly changing values unless the state is fully fixed.

## CI Policy

`just check` should remain the normal CI-equivalent command:

```sh
just check
```

It should include:

- Formatting check.
- Clippy with warnings denied.
- Workspace tests.

As the test suite grows, use explicit recipes for slower checks:

```sh
just test          # fast deterministic tests
just test-tui      # optional TUI integration tests, if added
just check         # CI-equivalent fast gate
```

Do not put slow pseudo-terminal or real-time soak tests into the default fast
gate unless they are proven reliable in CI.

## Near-Term Recommendations

1. Extract TUI state from `main.rs` into an `App` type.
2. Move menu/quest progression into `App::update(input, elapsed)`.
3. Keep `run()` as a thin loop that reads terminal events, computes elapsed time,
   calls `App::update`, and draws `render(frame, &app)`.
4. Add renderer tests with `TestBackend` for the current menu and quest screens.
5. Add input translation tests for quit keys, hotkeys, and ignored events.
6. Keep existing `core` tests as the main gameplay safety net and expand them as
   rewards/resources are introduced.

This gives the project fast feedback for almost all behavior while reserving
real terminal tests for the small amount of code that truly needs a terminal.
