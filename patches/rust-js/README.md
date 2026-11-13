# Downstream rust-js patches

AgentCube pins a published upstream commit through its rust-js Git submodule.
The following fixes stay in this repository until the upstream issues are resolved:

| Patch | Upstream issue | Purpose |
|---|---|---|
| 0001-preserve-unit-callback-results.patch | [#1685](https://github.com/rust-js-lang/rust-js/issues/1685) | Keep spawning unit callbacks synchronous so React sees undefined, not a Promise cleanup |
| 0002-find-submodule-worktrees.patch | [#1686](https://github.com/rust-js-lang/rust-js/issues/1686) | Find the actual submodule checkout in process-check tooling |

Setup, compilation, and local checks call scripts/patch-rust-js.ts. It verifies
that the checked-out commit matches the repository's gitlink, applies patches in
order, accepts an already-applied patch, and fails clearly on a conflict. It does
not reset the checkout or silently discard local edits. Git may show rust-js as
modified after setup: those tracked modifications are the expected applied patches.
No commit inside the submodule is needed.

Both patches contain regression coverage. The callback regression failed before
the fix and passed afterward; a mutation restoring the old rewrite was caught.
The submodule process-discovery regression covers the checkout and linked worktrees.

Verification on 2026-11-13: compiler lint, formatting, cargo tests, corpus shards,
browser tests, and the new mutation passed during the broad check. That whole
run reached its external 600-second limit under competing load. The three
initial infrastructure failures were corrected and their 74-test group passed;
the remaining metamorphic sample passed in 14 parallel parts, and the remaining
235 separate-crate tests passed separately. This is not a claim that one
uninterrupted full-check command completed.

When updating the submodule, review the upstream issues first. Remove a patch
only after the pinned upstream commit includes its fix and the relevant app and
regression checks pass. Rebase remaining patch files deliberately when upstream
changes their context. Do not replace the published pin with an unpublished
local commit.
