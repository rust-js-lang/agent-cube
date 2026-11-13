# 0004. Animate committed turns with temporary transforms, then snap exactly

Status: implemented. Extends ADR 0002's stable renderer lifetime.

Represent the visible shell as 27 rounded cubie groups with attached, slightly
raised colored tiles. Lighting and a glossy Phong plastic material provide depth;
tile colors remain readable. One shared body geometry and tile geometry serve all
cubies. Keep the renderer, camera, geometries and materials across moves and view
changes. Dispose them and cancel frame/timer callbacks only on cleanup.

The game referee still owns integer cube state. A fresh accepted move command
provides a transient Turns event containing its before-position and parsed moves.
Retries provide no event. The record format, receipts and WebMCP state observation
remain unchanged. A fresh local shuffle supplies its solved origin and full scramble
as a transient reset event; it animates without counting scramble turns as played
moves. Load/recovery snapshots still snap rather than inventing missed turns.

Publish committed state to the net, URL and status immediately. Queue visual
presentations in revision order. Animate only when the event starts at the
previous queued endpoint, or declares a shuffle reset; a gap or recovered snapshot snaps. Human controls and
WebMCP both use this queue. A move call waits until its committed revision has
been presented, while its durable acceptance remains independent of rendering.
An old retry receipt can precede the current revision, without rewinding the UI
or repeating its animation. Animation never resubmits or persists a move.

For a turn, reparent the nine affected cubies into an origin-centered group and
rotate it about the outward face normal with smoothstep easing. Move::rotation
owns the normal and signed quarter-turn count: inverse turns take the short path;
half turns rotate 180 degrees. Nominal duration is 240 ms for a quarter turn and
360 ms for a half turn. Scale long batches to a 2400 ms timeline, using elapsed
time rather than a frame count. Delayed frames can skip intermediate poses.

After each turn, apply the same parsed move to an exact temporary Cube, reset
the scene groups to their fixed slots, and repaint from its derived facelets.
At batch completion, present the authoritative target snapshot. Never read
floating-point transforms back into game state. Repeated moves therefore cannot
accumulate state drift.

Reduced motion, hidden pages and graphics-context loss complete committed
presentation immediately. A 2600 ms timer also fast-forwards stalled frames when
the event loop runs. Context loss keeps the compatible CSS cube playable.
Unmounting releases waiting callers; it does not roll back accepted commands.
An animation interrupted after commit returns the committed result rather than
suggesting a failed move. New commands remain guarded by revisions and receipts.

Deterministic Playwright checks control RAF, require changed pixels halfway
through a turn, and compare the final rendered endpoint with a new scene loaded
from the authoritative URL. All 18 turns also match an independent cube model.
The suite exercises queue order, retries, maximum batches, reduced motion,
visibility, stalled frames and actual graphics loss. Zeroing layer rotations
while keeping status metadata makes the halfway-pixel test fail.

Rust/JS decision: accepted events, turn direction, ordering and endpoint state
change observable results, so preserve Rust exactly. Rendering timing is browser
presentation; frame scheduling cannot change game acceptance or solved status.

Human clicks enter one FIFO acceptance queue. Each uses the latest durable revision
when processed, so rapid clicks do not compete with each other using a stale
revision. Agent requests retain explicit revision checks. Click intervals and
queue backlog raise visual pace up to six times normal, integrating elapsed
time so speed changes cannot reverse a turn. Replay keeps normal pace.
