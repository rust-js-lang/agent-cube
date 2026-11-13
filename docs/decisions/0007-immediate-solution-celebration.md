# 0007. Celebrate an accepted solution before its animation finishes

Status: implemented.

The referee computes every accepted batch's exact final position before visual
playback starts. LocalSession commits it, publishes the accepted snapshot and
presentation event, then signals celebration immediately if fresh played moves
changed an unsolved position to solved. Tool completion continues waiting for
normal cube presentation; celebration does not change that contract.

Rejected requests, duplicate receipts, shuffles, imports, recovery and replay do
not create celebration events. The existing snapshot.solved result remains the
authority; no solver or second move simulation is introduced. Both human and
WebMCP commands use this acceptance path.

The transient event carries its revision. The overlay appears only while that
revision is current and solved, so new moves or challenges dismiss it. Forty-eight
CSS particles use the cube palette, ignore pointer input, and clean up after
2.8 seconds. Reduced motion hides the decoration. No game data, URL, receipts or
replay history change. Existing Solved status provides the accessible result.

Tests stop cube RAF and require actual confetti pixels before the solution is
presented; retries, replay, reload, imports, human moves and reduced motion are
covered. Missing and delayed celebration mutations must fail. Fallback completion
uses presentation identity rather than game revision, including after reload
when those numbers differ.

Rust/JS decision: preserve acceptance, final solved state and event ordering
exactly. Decorative timing never changes cube state or command acceptance.
