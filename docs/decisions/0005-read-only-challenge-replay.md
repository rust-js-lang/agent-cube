# 0005. Replay a challenge's accepted moves without playing them again

Status: implemented. Extends ADR 0004's presentation queue.

Replay N moves appears only when the current challenge is solved and has accepted played moves.
It starts at that challenge's initial position and plays all accepted moves in
order, including multiple command batches. Rejected requests and retries add
nothing. A new shuffle or shared-position load resets the replay boundary.
The button stays hidden while unsolved or with no played moves, and is disabled while replay is being prepared/played.

The rule owner, Journal::playback, first validates/replays the durable record,
then selects the last shuffle/load receipt as the origin (or solved for a record
with no reset). It gathers only subsequent played moves. The scramble itself and
older challenges are excluded. This derives replay from the one accepted-command
record rather than introducing a second editable history or a schema migration.
Storage reads snapshots and playback through the same readonly transaction path;
only a changed accepted command can write the record.

Replay submits no commands, writes no receipt, and changes neither revision nor
move count. The game net, status, URL and WebMCP state remain authoritative while
the 3D view watches history. Replay ends at the current position. It is available
after reload when IndexedDB retained the record; temporary play loses history on
reload. Position-only shared URLs do not transfer replay history.

Presentation IDs are separate from game revisions. Every live/replay presentation
has a fresh ID so a repeat replay at the same revision still runs, and a late
completion from an interrupted replay cannot finish a newer live presentation.
Game-revision waiters remain independent of replay: observation/agent calls can
read committed state while replay is active.

Use normal 240 ms quarter / 360 ms half turns, with a 60-second cap for very long
replays. Keep the existing 2.4-second cap for live agent batches. Reduced motion,
graphics loss and hidden pages finish at the authoritative endpoint. Replaying
can be repeated; new committed moves, shuffles, linked challenges or external
refreshes interrupt it and resume live presentation from exact state. A stale
replay request fails explicitly instead of overwriting a newer position.

Browser tests replay the known 22-move solution, repeat it, reload it, compare raw
saved bytes and all game counters, and verify live interruption and accessibility.
The native and generated-JavaScript tests guard history boundaries and deduplication.
Mutating the reset selector or using game revision as the replay effect key is
caught. Replay shares the tested physical renderer instead of implementing a
second cube animator.

Rust/JS decision: replay origin, ordered moves, identities and saved state are
observable. Preserve Rust behavior exactly. Browser presentation can be skipped
or interrupted without accepting any additional game action.
