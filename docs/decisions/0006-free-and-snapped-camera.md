# 0006. Orbit the camera without rotating the puzzle state

Status: implemented.

Free dragging changes camera yaw and pitch around the cube center. Snap uses the
same continuous drag, then rounds both angles to the nearest 45 degrees on release.
Pitch reaches either pole; the Three.js camera uses a tiny offset there to keep
its up vector well-defined. Distance remains fixed. Front/Perspective commands
reset orientation even when the selected preset has not changed.

Preview owns camera angles and active-pointer capture. Only the primary pointer
and main mouse button start dragging. Capture keeps releases outside the view
working; cancellation/lost capture clears the drag. Touch scrolling is disabled
only on the preview. Arrow keys rotate the focused view; Home restores its preset.
Switching to Snap rounds the current view immediately. Free/Snap and custom
angles are presentation state for this visit; existing preset preference remains
saved separately from the game.

Two angles suffice for this fixed-distance orbit: no zoom, pan or inertial loop
is needed. Camera changes redraw the existing renderer and never restart an
active face turn. CSS fallback uses the same angles to rotate its six-face view.
Game facelets, notation, revisions, counts, URL and storage are unchanged.

Playwright checks actual pixels, both rendering paths, snapping, keyboard and
real touch events, cancellation, outside release, resets and orbit during play.
Mutations freeze camera position and change snap spacing to prove coverage.

Rust/JS decision: preserve angle arithmetic and input ordering exactly. Camera
orientation is browser presentation and never supplies authoritative cube state.
