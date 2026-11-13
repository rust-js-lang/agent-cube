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
bunx --bun playwright install chromium
bun run test
```

Build compiles Rust and bundles dist. Check verifies generated output against
Rust, runs rust-js's clippy, and checks Rust formatting. Tests serve the built
artifact on port 4173, use available CPU cores, and launch the pinned Chromium
with --enable-features=WebMCP enabled. They use Playwright's page.webmcp.tools() and page.webmcp.callTool(), backed by the real document.modelContext API,
not a substitute registry. Test failure must not silently skip unsupported APIs.

The four-test suite covers a visibly rendered canvas and changed pixels, responsive
layout, IndexedDB enumeration, native WebMCP discovery/execution/readback,
invalid input, manual controls when WebMCP is absent, and denied storage/WebGL. These are bootstrap
integration checks, not cube-engine or persistence correctness tests.

Playwright WebMCP calls prove browser integration. They do not prove that a
natural-language agent selects the right tool. The separate manual acceptance
is: open the app with a compatible agent client, ask “Show me the front of the
cube,” verify the camera changes, and ask the agent to read the preview state.
Record the client/browser versions when that acceptance is performed.

## Current scope

The preview is a solved cube visualization, not a playable cube engine. Human
buttons and set_camera_view use the same action. get_preview_state reads that
state. The GoogleChromeLabs use-webmcp-tool hook owns browser feature detection,
registration errors, callback freshness, and AbortSignal cleanup. Its npm API
is bound from Rust; AgentCube does not implement a browser registry or polyfill.

The IndexedDB check enumerates databases. It does not save sessions, establish
write permission, or promise permanent storage. Actual transactional persistence
belongs to milestone three. Unsupported WebMCP leaves the manual controls usable.
The app has no backend and no model API keys; cloud agents need their own network.
PWA installation and cached offline reload remain milestone four.

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
