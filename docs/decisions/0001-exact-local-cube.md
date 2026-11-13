# 0001. Exact local cube rules and an observable agent interface

Status: implemented.

The cube engine lives in a browser-independent Rust crate. Eight corners and
twelve edges carry integer positions and colored sticker normals. Centers are
fixed. A face turn rotates positions and normals using integer permutations;
there is no floating-point game state. The 54 facelets are derived from those
pieces in URFDLB order, row-major as each face is viewed from outside.

This encodes orientation directly rather than maintaining a second permutation
or facelet authority. Both renderers and the six-face view consume the same
derived stickers. Three.js never determines game state.

A move is U, R, F, D, L, or B, optionally followed by an inverse mark or 2.
Clockwise is viewed from outside that face. Count one token as one move (the
half-turn metric). Parse an entire batch and check its expected revision and
counter bounds before changing the cube. One accepted batch advances revision
once. Invalid and stale commands change nothing. Duplicate stale requests are
rejected for now; durable command IDs and receipts belong to the persistence
milestone, not this in-memory session.

shuffle_cube starts a new challenge from solved, applying 1–100 legal random
moves, default 20. This is a random walk, not uniform random-state sampling.
It advances revision and clears the played-move count. Its seed and scramble
history are not exposed through the agent state. get_cube_state returns all
facelets, revision, move count, solved status and face order. apply_moves accepts
the agent's solution moves. set_camera_view changes presentation only.

The ordinary Rust tests also run through rust-js with Bun. Independent move
fixtures and 64 native/JavaScript/reference comparisons guard notation and
orientation, which inverse-only tests would not catch. Three native mutations
prove the tests reject plausible rule and transaction regressions. A test-side
solver consumes only observed facelets to complete a shuffled cube through
WebMCP. Solver assistance is external to the game; no solve shortcut is exposed.

Rust/JS decision: results change if integer rotation, parsing, revision checks,
or command order change. Preserve the Rust behavior exactly. Rendering backends
may differ visually, but never change the game rules or returned state.
