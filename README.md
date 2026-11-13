# AgentCube

**A Rubik's Cube playground and racing arena for humans and AI agents, built with rust-js.**

Practice entirely in your browser, let an agent play through Chrome WebMCP,
or connect to a server on your Tailscale network and race agents on other machines.

## Status

Local play is implemented: a shared Rust cube engine, all 18 face turns,
shuffling, a six-face view, Three.js rendering with a working no-WebGL view,
and native WebMCP tools. Agents inspect the cube and submit legal solution moves.

Open [AgentCube](https://rust-js-lang.github.io/agent-cube/). Saved sessions, PWA
caching, and multiplayer remain planned; reloading currently starts a fresh cube.

See [Development](docs/development.md) for setup, checks, and the GitHub
Pages deployment workflow.

## Two modes, one game

| | Offline | Online |
|---|---|---|
| Referee | Browser | Native Rust server |
| Persistence | IndexedDB | SQLite |
| Network | No game backend required | Server reachable through Tailscale |
| Play | Local practice and agent challenges | Rooms, racing, spectators, and replay |
| Agent interface | Chrome WebMCP | The same Chrome WebMCP tools |

The game can work offline; an agent using a cloud model still needs its own
network connection. WebMCP exposes tools to a compatible agent client; it does
not supply the model or agent runtime.

## Technology direction

- Ordinary Rust for cube rules, match rules, and shared protocol types.
- rust-js for compiling the Rust browser application to readable JavaScript.
- React for the interface and Three.js for the animated cube.
- Native Rust with Axum, HTTP commands, and SSE events for multiplayer.
- IndexedDB for local sessions and SQLite for server sessions.
- A PWA for installed, cached offline play after initial asset acquisition.

Everything starts in this repository. Keep clear boundaries, and extract
libraries only when real consumers justify doing so.

## Design documents

- [Development](docs/development.md)
- [Architecture](docs/architecture.md)
- [Implementation plan](docs/implementation-plan.md)

The local cube is controlled by both humans and WebMCP agents. Multiplayer will
build on the same native-tested rules. See [test evidence](TESTS.md).
