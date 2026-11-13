# Development

## Bootstrap

```sh
git clone --recurse-submodules https://github.com/rust-js-lang/agent-cube.git
cd agent-cube
bun run setup
bun run dev
```

For a checkout created without submodules, first run:

```sh
git submodule update --init --recursive
```

Use Bun 1.4.4 and rustup. The root rust-toolchain.toml pins the application's
Rust release. The submodule pins the compiler and its rustc-dev requirements;
rustup installs those on the first compiler build. The first build compiles
rust-js; later builds are incremental. The compiler uses Cargo's available CPU
cores. A shared sccache installation can reduce repeated dependency compilation.

Setup and compilation apply the tracked downstream fixes in
[patches/rust-js](../patches/rust-js/README.md). Their upstream issues are filed;
no compiler source changes are pushed upstream by this project. The submodule
remains at a published pin, with the patch files as the authority for local edits.

The dev command compiles Rust once and starts Vite. After changing Rust, run
bun run compile in another terminal; Vite picks up the generated output. Do not
edit generated JavaScript. JS, JSX, and declarations are committed beside Rust;
source maps, dependency caches, and dist are not.

Commands below should be run with an external timeout in automation or local
agent sessions. Typical budgets: setup 120 seconds, first build 300 seconds,
incremental build/check 120 seconds, browser installation 120 seconds, tests
90 seconds. A timeout needs diagnosis, not an unexamined longer retry.

## Checks

```sh
bun run build
bun run check
bun run test:core
bunx --bun playwright install chromium
bun run test
bun run test:mutations
bun run test:browser-mutations
```

Build compiles Rust and bundles dist. Check verifies generated output against
Rust, runs rust-js's clippy, and checks Rust formatting. Tests serve the built
artifact on port 4173, use available CPU cores, and launch the pinned Chromium
with --enable-features=WebMCP enabled. They use Playwright's page.webmcp.tools() and page.webmcp.callTool(), backed by the real document.modelContext API,
not a substitute registry. Test failure must not silently skip unsupported APIs.

See [TESTS.md](../TESTS.md) for the 50 browser tests, sixteen native and generated-JavaScript tests, 64 parity probes, and sixteen mutation checks.

## Current scope

The local cube supports all 18 face turns, legal shuffles, solved detection, and
revision-checked batches. Human controls and WebMCP share the same referee. Tools
are get_cube_state, shuffle_cube, apply_moves, and set_camera_view. Agents obtain
all facelets and submit solutions; the app does not contain a solver shortcut.

A stable Three.js scene shows 27 rounded cubies with individually attached
stickers. Played turns rotate nine cubies as a group, then snap from exact state.
Ordered presentation finishes before a move tool returns; reduced motion and
interruption fast-forward presentation without changing accepted game state. If WebGL creation fails or
its context is lost, a CSS cube replaces the hidden canvas. Both consume exactly
the same state. The six-face view remains available independently of graphics.

IndexedDB saves accepted commands and deterministic replay receipts in a single
transaction. Reload restores state and move count; commands acknowledge only
after transaction completion. Mutating tools require commandId and
expectedRevision. Retry the exact request with the same ID after a lost response;
a receipt can refer to an earlier revision, so read again before a new command.

After solving, Replay N moves reconstructs the current challenge from the latest shuffle or
shared-position load and all accepted played moves. It changes presentation only;
new live commands interrupt it. Replay runs at normal turn speed, capped to a
60-second timeline for long histories, and stays available after a saved reload.

The address contains the current position in a #cube fragment. Share it to start
that position with zero moves in another browser. Invalid links and unsupported
saved formats block recovery and retain the previous data. Storage denial at
startup enables clearly labeled temporary play. PWA caching and multiplayer
remain later milestones.

## GitHub Pages deployment

The repository has one workflow, Deploy GitHub Pages, triggered on pushes to
main or workflow_dispatch. It checks out submodules recursively, installs frozen
Bun dependencies for both workspaces, builds the pinned compiler and app, uploads
dist, and deploys it. There is no separate Check workflow; run checks locally.
All action references pin the latest release's full commit SHA, with version
comments. Release tags and commits were verified through GitHub CLI on 2026-11-13.

GitHub Pages uses the GitHub Actions publishing source. The project URL is
https://rust-js-lang.github.io/agent-cube/.

Vite emits relative asset URLs so the static app works at both / and /agent-cube/.
This does not host the future native multiplayer referee. Deploy that server
separately on the Tailscale network.

## Updating rust-js

Choose a published upstream commit, update the submodule checkout, align the
root toolchain with its Rust release, reinstall submodule dependencies when its
lockfile changes, then rebuild, review generated output, and run all app checks.
Commit the pointer and any output changes together. Never pin unpublished local
compiler work: CI must be able to fetch the exact commit.

## Rendering cost

One local 14-worker software-rendering run with two-segment rounded geometry and
MeshStandardMaterial took 26.1 seconds for the original 24 tests and hit three
five-second startup/action limits. One-segment rounded geometry and glossy
MeshPhongMaterial reduced GPU/CPU cost: the expanded 33-test run passed in 9.6
seconds with the same 14 workers and existing timeouts. Keep both optimizations;
these are local diagnostic measurements, not hardware-independent benchmarks.
