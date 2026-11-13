# Tests

Updated 2026-11-13. Run checks locally; the Pages workflow only builds and deploys.

| Suite | Size | What it proves |
|---|---|---|
| Native Rust cube tests | 6 tests | All 18 moves and three algorithms match independent fixtures; move inverses, order four, sticker conservation, parsing, revisions, and atomic batches |
| The same tests compiled through rust-js | 6 tests | The Rust tests also pass as JavaScript with Bun |
| Cross-target/reference parity | 64 seeded 100-move sequences | Native Rust, generated JavaScript, and an independent cube model produce identical algorithms and all 54 facelets |
| Playwright against the production bundle | 7 tests | Visible sticker pixels, stable renderer lifetime, camera/move updates, all moves, WebMCP shuffle/solve, invalid/stale/concurrent commands, missing WebMCP, denied storage, failed WebGL, and actual context loss |
| Native mutations | 3 | Reversed clockwise turns, accepted stale revisions, and silently truncated batches must fail tests |
| Renderer lifetime mutation | 1 | Recreating the renderer on camera changes must fail the browser resource-lifetime assertion |

```sh
bun run build
bun run check
bun run test:core
bun run test
bun run test:mutations
bun run test:renderer-mutation
```

The browser suite uses native Playwright page.webmcp discovery and tool calls,
with Chromium's WebMCP feature enabled. It runs at /agent-cube/, matching Pages.
Pixel checks decode real screenshots and require substantial sticker-colored
areas: a blank canvas, white browser failure panel, or error message cannot pass
merely by being visible or changing pixels. Camera changes and moves must not
create additional WebGL contexts. Creation failure and context loss must hide
the canvas and show a playable CSS cube with the correct stickers.

The failure-path regression was first run against the old build and failed
because its broken canvas stayed visible. The renderer mutation was also caught.
The final pixel check additionally caught invisible sticker planes after a material
binding refactor; the fix uses one material per plane instead of a material array
(which Three.js renders through geometry groups). All seven tests passed after the fix.

The independent model/solver is vendored under tests/reference/cubejs with its
MIT license and upstream commit. It is never included in the app bundle. The
20-move shuffle-and-solve browser test computes a solution from observed facelets
alone, then submits it through apply_moves and checks both the tool result and
visible state. The app exposes no solver shortcut, scramble history, or seed.

Additional acceptance: Codex used the built-in browser's discovered WebMCP tools
to shuffle a cube by 20 moves, read its state, compute a solution outside the app,
and submit 20 legal solution moves. The browser and returned state both reported
solved. This verifies agent tool use, not unaided language-model solving ability.

Local sessions currently reset on reload. Persistence, durable retry receipts,
PWA reload caching, and multiplayer require their own future tests.
