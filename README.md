# AgentCube

**A Rubik's Cube playground and racing arena for humans and AI agents, built with rust-js.**

Practice entirely in your browser, let an agent play through Chrome WebMCP,
with saved local play and shareable cube links. Tailscale multiplayer is planned.

## Status

Local play is implemented: a shared Rust cube engine, all 18 face turns,
shuffling, a six-face view, animated Three.js cubies with a working no-WebGL view,
and native WebMCP tools. Rounded cubies and glossy plastic give each face turn
visible depth. Human and agent moves share ordered, eased layer animations;
reduced motion presents the final position immediately. Agents inspect the cube
and submit legal solution moves.

Open [AgentCube](https://rust-js-lang.github.io/agent-cube/). Sessions and retry
receipts are saved in IndexedDB. Copy the address or use **Copy challenge link**
to send the exact cube position to someone else; their challenge starts with
zero moves. Links use a validated #cube=<54 URFDLB stickers> fragment, without
move history or shuffle seeds. The fragment is not sent in HTTP requests.

Visiting without a cube position starts a fresh 20-move shuffle, even if a
previous game is saved. The address immediately contains that position.
Reloading that address resumes the saved cube, counts and receipts. A different valid link starts
a new challenge; an invalid or impossible position leaves saved play untouched.
If storage is denied, temporary play still works and the URL preserves the
position, but history and retry receipts are lost on reload. PWA asset caching
and multiplayer remain planned.

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
