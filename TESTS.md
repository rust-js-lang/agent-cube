# Tests

Updated 2026-11-13. Run checks locally; the Pages workflow only builds and deploys.

| Suite | Size | What it proves |
|---|---|---|
| Native Rust cube tests | 16 tests | All 18 moves and three algorithms match independent fixtures; move inverses, order four, sticker conservation, parsing, revisions, and atomic batches |
| The same tests compiled through rust-js | 16 tests | The Rust tests also pass as JavaScript with Bun |
| Cross-target/reference parity | 64 seeded 100-move sequences | Native Rust, generated JavaScript, and an independent cube model produce identical algorithms and all 54 facelets |
| Playwright against the production bundle | 50 tests | Visible sticker pixels, stable renderer lifetime, camera/move updates, all moves, WebMCP shuffle/solve, invalid/stale/concurrent commands, missing WebMCP, denied storage, failed WebGL, and actual context loss |
| Native mutations | 8 | Reversed turns, stale revisions, truncated batches, impossible permutation/twist/flip, and missing retry recognition must fail tests |
| Browser mutations | 8 | Renderer recreation, aborted-transaction acknowledgement, stationary geometry, replay identity, missing acceleration, and unsolved Replay visibility must fail the corresponding production-browser assertions |

```sh
bun run build
bun run check
bun run test:core
bun run test
bun run test:mutations
bun run test:browser-mutations
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

Persistence/link coverage now includes exact reload recovery, camera preference,
original receipts after later commands, collision rejection, shuffle retries,
two-tab revision races and simultaneous retries, post-put transaction aborts,
unreadable/future records and newer database schemas, copied clipboard links,
and a recipient in a separate browser context solving the shared position.
Denied storage preserves position through the URL while explicitly losing
history on reload. Impossible URL positions cannot overwrite existing play.
The domain suite also imports 128 seeded 100-move positions and rejects
impossible pieces, corner twists, edge flips, parity, counts and centers.

Eight native mutations and four production-browser mutations were caught. The
aborted-commit mutation deliberately ignores the transaction completion error;
the test rejects the resulting false success acknowledgment. PWA reload caching
and multiplayer still require their own future tests.

Entry regressions first failed against the old build. Bare visits now create
a fresh 20-move shuffle in both durable and temporary modes, place it in the URL,
and preserve it on reload. Root/home navigation and a new tab also start fresh,
with existing tabs notified. Rule fixtures explicitly supply their solved
position through the supported link format.

Animation coverage controls RAF while leaving IndexedDB and React scheduling real.
It requires visible halfway-turn pixel movement, compares the endpoint against a
freshly rendered authoritative position, and checks all 18 directions/endpoints
against the independent cube model. It also checks ordered queued commands,
non-repeating retries, a maximum 128-move batch, reduced motion, stalled-frame
deadlines, preference changes, hidden pages and actual WebGL context loss during
rotation. Tool completion waits for presentation. Native and generated Rust tests
verify accepted turn events reconstruct the committed state, with no event on retry.
A mutation zeroing actual layer rotation (while retaining angle/status metadata)
failed the halfway-pixel assertion. Thus status attributes alone cannot prove motion.

Codex also invoked a four-move sequence through the built-in browser’s native
WebMCP tools. The call completed with the canvas idle at the committed visual
revision, and the visible game showed four moves. A demo records real
production-canvas frames; it is not a separately drawn animation.

Replay coverage solves the known 22-move challenge, verifies the original start
and readable playback beyond the live-batch speed budget, repeats replay at the
same game revision, and reloads before replaying again. The saved JSON bytes,
URL, revision, move count and native WebMCP observation remain identical. Other
tests join separate batches without duplicate retries, reset challenge history,
interrupt replay with new play or graphics loss, and cover reduced motion and
human controls without WebMCP. Native/JS tests verify reset boundaries, complete
move order, and corrupt-history rejection. New mutations select the wrong reset
boundary or make replay depend on unchanged game revision; their tests fail.

Shuffle tests start from a mixed position, require intermediate layer rotation,
and reconstruct the accepted zero-move challenge. A twelve-click burst checks
that every click commits in order, accelerates the visible layer, and finishes
at the independent reference position. Replay stays hidden until solved.
Browser mutations remove acceleration and solved-only visibility to prove
these assertions detect regressions.

Camera tests require changed rendered pixels on both Three.js and CSS views,
nearest-angle snapping after release, mouse and real Chromium touch input,
cancellation and release outside the view, keyboard control, repeated presets,
and orbiting during a face turn. Saved bytes, URL, revision and move count remain
unchanged. Camera-position and snap-step mutations must fail these assertions.
