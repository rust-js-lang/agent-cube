# 0002. A renderer belongs to its canvas; graphics failure remains playable

Status: implemented.

A user reproduced a broken white canvas and an unavailable-preview message in
the built-in browser. Its log reported a Three.js initialization error reading
shader precision from a null graphics object. The previous camera-dependent
effect disposed the renderer, forced context loss, and immediately constructed
another renderer on the same canvas. That was an incorrect resource lifetime.

Create the renderer once on mount. Camera changes and accepted game moves update
the existing camera and sticker materials. On cleanup, remove listeners and
dispose resources; do not force context loss on a canvas React may reuse.

WebGL creation can still fail because of browser or graphics configuration.
Handle creation failure and context loss by hiding the failed canvas and showing
a CSS cube containing the same 54 stickers. Keep the six-face view and all human
and WebMCP commands usable. Never leave a browser failure surface underneath an
error overlay, and never claim that the game itself needs WebGL.

The old test asserted only that a canvas was visible and changed pixels. A blank
or crashed canvas could satisfy both. Tests now require substantial exact
sticker-colored regions in a screenshot, count graphics-context creation across
repeated camera and cube changes, and inject both creation failure and actual
WEBGL_lose_context loss. The old failure-path build failed the new regression;
a mutation making the effect camera-dependent fails the lifetime assertion.

Compatible rendering is an alternative presentation of one state, not a second
cube engine. Graphics support does not decide move legality or solved status.
