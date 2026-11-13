# Implementation plan

This is a phased plan, not a claim that any feature is implemented.
Keep everything in one repository. Build the smallest complete vertical slice
at each phase; do not postpone correctness behind presentation work.

## 1. Bootstrap and prove the browser boundary

Implemented locally on 2026-11-13. The app builds from a pinned rust-js
submodule; four production-browser tests cover the Rust UI, Three.js, IndexedDB,
and Playwright WebMCP. Codex separately discovered and invoked a live tool
through Playwright CLI and checked visible state. See [test evidence](../TESTS.md).
GitHub Pages builds and deploys on pushes to main; tests run locally.

- Establish the Rust workspace, Bun scripts, rust-js browser build, and Pages deployment.
- Pin the compiler/toolchain and document reproducible build commands.
- Compile a small React page from Rust and inspect the generated JavaScript.
- Prove the required IndexedDB, Three.js, and WebMCP binding shapes early.
- Verify tool discovery and invocation with an actual compatible Chrome agent.
- Fix binding/compiler gaps in their proper owners before building on them.

Exit: a reproducible build and one real agent-invoked browser action.

## 2. Correct cube engine and flat playable view

Implemented locally on 2026-11-13: native and generated-JavaScript tests, 64
seeded native/JS/reference comparisons, all legal turns, a six-face view, and
WebMCP shuffle/read/apply commands. Renderer failures now use a playable CSS cube.
Codex also shuffled and solved a 20-move cube through the built-in browser tools.


- Define coordinate, facelet, orientation, and notation conventions.
- Implement exact cubie state, legal moves, parsing, sticker projection,
  solved checks, and reproducible seeded scrambles.
- Add known fixtures, inverse/four-turn laws, invariants, and generated sequences.
- Compare identical inputs through native Rust and generated JavaScript.
- Render all six faces, with keyboard/buttons and a move history.

Exit: a human can scramble and solve a cube; independent expected fixtures and
cross-target comparisons pass.

## 3. Standalone session, persistence, and agent play

The in-memory referee and agent tools are implemented. IndexedDB persistence,
command receipts, durable replay, and reload recovery remain outstanding.


- Define commands, snapshots, accepted events, revision checks, and receipts.
- Implement the local referee and one ordered command acceptance path.
- Implement IndexedDB persistence, schema versioning, and atomic acceptance.
- Add reload recovery and separate small UI preferences into localStorage.
- Expose get_cube_state, apply_moves, and local-practice shuffle_cube through WebMCP.
- Validate whole batches before execution and define cancellation/completion.
- Keep challenge seeds and histories out of the agent observation surface.
- Exercise an agent on short scrambles without an integrated solver shortcut.

Exit: an agent can play a persisted standalone session; lost responses and
retries never duplicate moves, and reload restores the committed state.

## 4. Three.js presentation and offline installation

- Render the 3D cube with typed bindings and camera controls.
- Animate face turns from accepted events, snapping to exact state afterward.
- Retain the flat view, keyboard controls, and accessible status messages.
- Add timing, playback, speed controls, and reduced-motion behavior.
- Build the graphite competition visual style with responsive layouts.
- Add manifest, service worker, versioned asset caching, and safe update prompts.
- Document initial loading/install and agent model connectivity requirements.

Exit: a polished standalone app works after installation with network disabled;
animation, reload, and replay agree with the authoritative cube state.

## 5. Native referee and two-player online race

- Implement the same game-session behavior with a native Rust server.
- Add SQLite schema/migrations and transactional command/event/receipt storage.
- Add typed HTTP commands and sequenced SSE, resume cursors, and snapshot recovery.
- Implement player sessions, room links, permissions, and spectator access.
- Add start gating, identical hidden-until-start scrambles, completion checks,
  and one explicit bounded-batch policy.
- Define tie, disconnect, restart, move-count, and timing rules before ranking.
- Connect browser clients through the remote session implementation.
- Add HTTPS/Tailscale hosting instructions without public internet exposure.

Exit: two machines on the tailnet race with a synchronized start; the server
verifies the winner, and reconnect/replay reproduce the accepted result.

## 6. Agent races and spectator experience

- Run two real WebMCP agents on different machines against one room.
- Verify identical tool schemas across local and remote sessions.
- Add side-by-side cubes, live counts, estimated timers, and connection states.
- Add replay navigation and an accepted-command/event activity view.
- Separate human, agent, and mixed categories with published assistance rules.
- Test concurrency, dropped replies, stream gaps, slow spectators, stale plans,
  unauthorized moves, and commands submitted before the start.

Exit: a complete, watchable, recoverable agent-vs-agent match across machines.

## Later, only when justified

- Persistent accounts, match history browsing, ratings, and tournaments.
- Replay export/import and additional cube sizes.
- An explicitly separate assisted solver mode.
- Transport changes supported by latency/throughput measurements.
- Extract infrastructure or UI libraries when another real consumer needs them.

## Decisions to resolve during implementation

These do not block the documentation bootstrap:

- Additional agent clients beyond the verified Playwright CLI and Chromium setup.
- Initial tailnet hosting machine, HTTPS setup, and player-session lifecycle.
- Exact batch limits, tie rules, clock/restart policy, and assistance categories.
- Durable format/migration policy and replay retention limits.
- rust-js dependency pin and the smallest required browser/library binding work.

Do not treat these as reasons to introduce alternate implementations or hidden
fallback behavior. Record concrete decisions as the vertical slices prove them.
