# Architecture

## Product

AgentCube is a Rubik's Cube playground and competition arena. A human or AI
agent manipulates a real game state, and every accepted move is observable
and replayable. Start with the standard 3x3 cube.

Support local practice and challenges first, then online rooms with human vs
human, agent vs agent, and human vs agent races. Keep human, agent, and mixed
results in separate competition categories. Start with an open agent category:
ordinary browser traffic cannot prove that a participant used no external solver.

## One repository

Keep all application code and supporting infrastructure here initially.
The proposed layout is a target, not existing implementation:

```text
agent-cube/
  Cargo.toml                Rust workspace
  package.json              Bun, browser build, and checks
  crates/
    cube-core/              Cube state, moves, notation, sticker projection
    match-core/             Match transitions and accepted game events
    protocol/               Typed commands, snapshots, events, endpoint contracts
    storage/                Persistence contract and target-specific adapters
    server/                 Native referee, Axum routes, sessions, event delivery
    web/                    rust-js application, local referee, WebMCP, React, Three.js
  web/                      Browser entry point, assets, CSS, PWA configuration
  tests/                    Cross-target and browser integration coverage
  docs/
```

Dependencies point inward toward domain rules. Cube rules know nothing about
React, Three.js, WebMCP, storage, or HTTP. Native-only dependencies must not leak
into browser builds. The exact crate split can evolve as implementation proves it.
Do not create a general framework before the application needs it.

## Two execution modes

```text
Human controls / AI agent through WebMCP
                    |
              Game-session API
              /              \
      Local referee       Remote session client
             |                    |
      Shared game rules      HTTP commands
             |                    |
         IndexedDB         Native Rust referee
                                  |
                           Shared game rules
                                  |
                                SQLite

Snapshots and accepted events -> UI, Three.js animation, history, replay
```

The game-session boundary supports reading a snapshot, submitting a command,
and subscribing to accepted events. Model command completion and cancellation
explicitly. Both modes expose the same domain behavior and WebMCP tool schema.

Offline mode runs its referee in the browser. After assets are cached, the game
requires no backend connection. Online mode sends commands to the server, which
alone validates state transitions, timestamps completion, and declares winners.
Client prediction, if introduced later, is presentation only.

An online disconnect disables new submissions until recovery. Never silently
switch an online match to a local referee. Show the active mode and connection
state clearly. Abandoning an online session for local practice creates a separate
local session and cannot produce an online result.

## Rules and representation

Use eight corner positions/orientations and twelve edge positions/orientations
as the canonical cube representation, with fixed centers. The implemented
orientation uses exact integer sticker normals on those pieces; it never reads
back rendering transforms. Derive the 54 visible
stickers from it. Centralize legal move permutations and notation parsing.

Start with U, D, L, R, F, B and their inverse and double turns. Specify face order,
facelet order, colors, coordinate axes, and clockwise viewing convention in the
implementation and agent-facing schema. Camera movement never changes notation.

Store discrete state using exact values. Animate temporary transforms, then snap
rendered pieces back to the exact authoritative state; never recover cube state
from floating-point Three.js transforms.

The shared match rules own lifecycle transitions such as lobby, countdown,
running, and finished. Referees provide clocks, randomness, player identity, and
persistence. Renderers never decide legality or completion.

## Persistence and execution are separate

A storage adapter answers where accepted state is saved. A referee answers
whether a command counts. The UI and WebMCP use the game-session boundary, not
raw storage APIs. In online mode, the browser never accesses SQLite directly.

Use IndexedDB for offline sessions, command receipts, move logs, and replay.
Use localStorage only for small preferences. Use SQLite for corresponding server
records. Keep application records and schema migrations owned by AgentCube.

Define persistence operations around domain needs: create/load a session,
atomically accept a command, and read ordered events for replay or recovery.
Do not expose a generic key/value abstraction that loses the required guarantees.

For a successful command, update session revision, append accepted move events,
and save its retry receipt in one transaction. Acknowledge only after commit.
Serialize command acceptance per session, including simultaneous browser tabs.
Bind command identity to session, authenticated actor, and stable request content.
The same command ID with different content must fail; a retry with the same
content returns the original result without applying moves again.

Prepare a proposed transition without mutating authoritative in-memory state,
commit it, then publish it. A failed transaction must not leave memory ahead of
storage. Offline reload and server restart must restore a coherent revision.

IndexedDB and SQLite provide different APIs; test the same application-level
atomicity and retry contract against both. Persist format versions and define
migration behavior. Browser storage may be cleared or evicted; never promise
permanent retention. Replay export/import is a later durability improvement.

## Online transport and recovery

Start with typed HTTP command endpoints and an SSE stream of accepted events.
Rubik's moves are discrete; measure actual interaction latency before adding a
different transport. No WebSocket implementation is required initially.

Each command carries a command ID and expected state revision. Each accepted
event carries a sequence number within its session. Revisions guard stale
commands; command IDs guard duplicate execution. Specify ordering when a batch
produces multiple move events.

Reconnect from the last received sequence. Detect duplicates and gaps. When
history is unavailable, send a snapshot tied to a precise sequence and continue
after it, without a race between snapshot acquisition and stream subscription.
Slow consumers must recover explicitly rather than silently miss moves.

Durable events are the recovery source. Commit before notifying; notification
failure cannot erase an accepted move. UI animations and transient broadcasts
are not the source of truth.

## Timing and fair competition

Give each player an independent cube with the same scramble. Reveal challenge
state at the server-controlled start, not during a pre-start planning window.
Seeded scrambles are reproducible; do not claim uniform random-state sampling
from a random move sequence.

Publish one bounded-batch submission policy for each competition category.
The server serializes accepted actions and records the winning transition using
its clock. Specify tie handling, move-count metric, disconnect behavior, and
server-restart policy before ranked play. Display timers are estimates; animation
speed does not determine results. Network latency affects close finishes.

The event log supports replay and verification of results. Do not expose a
challenge's scramble seed, move history, or solution to competing agents while
the challenge is active. Initially agents solve from the current cube state;
an optional assisted solver mode must be explicitly separate from challenges.

## WebMCP

Initial conceptual tools:

| Tool | Behavior |
|---|---|
| get_cube_state | Return all facelets, fixed face order, revision, move count, and solved status |
| apply_moves | Apply a bounded notation string against expectedRevision, atomically |
| shuffle_cube | Create a new local random-move challenge, default length 20; unavailable in future active races |

Register tools using the browser's current supported imperative WebMCP API.
At design time Chrome documents document.modelContext.registerTool; verify the
target Chrome release before implementation. Feature-detect availability and
keep human controls usable when WebMCP is absent. Clean up registrations and
handle cancellation according to the supported browser contract.

WebMCP is a browser interface, not the multiplayer transport, AI model, or
backend MCP server. An agent client must independently discover and invoke the
page's tools. Use an actual compatible client in end-to-end validation.

Keep local/remote tool semantics consistent. A move call returns after the
accepted sequence is presented, with authoritative state and revision. Rendering
must not block server acceptance or alter competition timing. If presentation is
interrupted after acceptance, report what was committed; never imply rollback.
On cancellation, stop unsubmitted work; already committed moves remain accepted.
Choose a bounded animation policy so spectators and agents cannot lag indefinitely.

## Browser interface

Use rust-js with React for controls and Three.js directly through typed bindings
for the 3D scene. Include a six-face flat view for inspection and accessibility.
Pass both human and agent actions through the same command handler.

The proposed visual direction is a quiet competition stage: graphite surfaces,
bright cube colors, large readable timers, and restrained controls. Local play
focuses on one cube; races show player cubes side by side with participant names,
move counts, connection status, and an accepted-event timeline. Include keyboard
controls, visible focus, reduced motion, and labels beyond color alone.

Installable offline support caches versioned application assets. An active
session must not be unexpectedly reloaded by a service-worker update. Agent model
availability is separate: a cloud model still needs internet access.

## Private network hosting

Host the server on a machine reachable through Tailscale. Configure a trustworthy
HTTPS origin for browser APIs and PWA capabilities, and verify WebMCP on that
origin. Tailscale is connectivity, not application-level player authorization.

Issue player sessions, bind commands to their owners, enforce room permissions,
and separate spectators from participants. Keep cookies/tokens out of URLs and
logs. Define origin checks, CSRF protection where applicable, input/batch limits,
and per-player command limits before enabling remote mutation.
Public internet hosting is outside the initial scope.

## Reuse and rust-js

The agent-hub source review identified typed endpoint contracts, SQLite
idempotency, browser transport/recovery, and optional process supervision as
useful designs. This repository starts self-contained: no extraction or shared
library dependency is required now. Reuse ideas deliberately; do not copy hub
business schemas or create a dependency on the messaging application.

Later extraction should be driven by working consumers. Keep semantic ownership
clear now so sharing does not require a rewrite. The reviewed supervisor's
documented platform support is macOS Apple Silicon, so it is not an initial
hosting requirement.

Write idiomatic Rust. When browser APIs, third-party bindings, or compiler behavior
are missing, fix the relevant rust-js owner with tests rather than hiding the gap
in application workarounds. Keep generated JavaScript readable and inspect it.

## Validation

- Known move fixtures, inverse moves, four quarter-turns, and invariants.
- Native Rust versus rust-js-generated JavaScript on identical move sequences.
- Independent expected fixtures to catch a shared error in both targets.
- Storage contract tests for IndexedDB and SQLite, including failed commits,
  duplicate commands, revision conflicts, reload, and recovery.
- Browser tests for controls, animations, offline caching, and WebMCP lifecycle.
- Multiplayer tests for start gating, authorization, concurrent commands,
  dropped acknowledgments, event gaps, reconnect, spectators, and verified wins.
- Deterministic replay must reproduce final cube states and accepted results.

## References

- rust-js guide: https://github.com/rust-js-lang/rust-js
- Chrome WebMCP: https://developer.chrome.com/docs/ai/webmcp
- Chrome imperative API: https://developer.chrome.com/docs/ai/webmcp/imperative-api

External browser APIs remain experimental and must be checked against the target
browser when implementation begins.
