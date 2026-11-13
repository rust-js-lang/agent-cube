# 0003. Commit local play before acknowledgment; share positions through URLs

Status: implemented. Supersedes ADR 0001's temporary rejection-only retry behavior.

The browser session boundary exposes read and submit. Its storage adapter either
uses IndexedDB or explicit temporary memory when opening storage is denied.
The rules stay in cube-core's Journal, independent of React, browsers and I/O.
The future SQLite referee can consume the same command/replay behavior.

Every mutation carries commandId (1–128 ASCII characters) and expectedRevision.
Identity is scoped to this browser's practice journal. Commands are compared by
typed content, including the expected revision and exact notation string.
An identical retry returns the original snapshot even after later commands;
a conflicting use of an ID fails. The UI displays the latest committed snapshot,
not an old retry receipt. Both human controls and agents use this path.

IndexedDB database agent-cube version 1 has one practice store. Its active record
is a JSON string with format version 1 and ordered accepted commands. Shuffle
entries retain their seed internally; other entries carry no seed. Replay
validates every command, revision, ID and seed shape, reconstructing both current
state and original receipts. No unchecked serialized cubie state is trusted.

A readwrite transaction reads the current record, proposes the transition,
writes the new journal, and waits for transaction.done. Only then does the
session publish, update its URL, notify other tabs and acknowledge success.
IndexedDB serializes readwrite transactions across tabs. BroadcastChannel and
focus refresh provide presentation updates; notification failure cannot undo
an accepted command. A stale command refreshes the UI and returns an error.
Cancellation before the write accepts nothing; once submitted, the bounded
transaction completes and its committed receipt wins over later cancellation.

Keep the initial implementation small: one bounded journal, no checkpoint cache
or second state authority. Retain at most 4096 accepted commands and 4 MiB of
encoded JSON. Exceeding a bound rejects the proposal without dropping older
receipts. Check the byte limit before writing as well as on load. Compaction,
archiving and session management need an explicit future format change; this is
not an unlimited history service. Browser eviction or clearing can remove saves.

Unreadable data, unsupported record versions, and a newer database schema block
recovery without overwriting data or silently switching to memory. Connections
close on version changes so an updated application can migrate the database.
A write failure during saved play keeps that mode; it never falls back to an
unsaved replacement. Camera view alone is a small localStorage preference.

An address without a position requests a fresh 20-move shuffle, even with saved
play. Initial entry and root/home navigation use the same decision. This command
commits and notifies other tabs just like other accepted changes. The resulting
URL includes the position, so a reload resumes rather than reshuffling.

The current position is #cube=<54 URFDLB stickers>, updated with replaceState
only after acceptance. The fragment carries neither history, seed nor receipts,
and is absent from HTTP requests. Copy challenge link copies the address. A new
browser loads just that position with a zero move count. Matching saved play
resumes its original count/receipts; a different valid position starts a new
challenge. Invalid positions leave saved records untouched. Temporary play can
recover the position from its URL after reload, but loses history and receipts.

Import validation enforces nine stickers per color, fixed centers, unique
physical pieces, cyclic corner handedness, corner twist sum modulo three, edge
flip sum modulo two, and equal edge/corner permutation parity. These are exact
integer rules owned by cube-core, tested on native Rust and generated JavaScript.
The app exposes no solver. A test-only independent solver verifies that a
recipient can solve a shared, legally imported position through native WebMCP.

Rust/JS decision: persistence acceptance, replay, parsing and imported positions
change observable results. Preserve Rust's rules exactly. IndexedDB transaction
completion is the browser adapter's explicit acceptance boundary.
