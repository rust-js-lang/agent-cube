# Tests

Updated 2026-11-13. The bootstrap suite has four tests in one Playwright file.

| Test | Evidence |
|---|---|
| Rust UI and renderer | A real WebGL canvas renders, camera changes alter pixels, IndexedDB enumeration succeeds, and mobile layout fits the viewport |
| WebMCP | Playwright discovers native Chromium tools, calls a camera action, reads it back, rejects invalid and extra input, and sees human changes |
| WebMCP unavailable | Manual camera controls still work when the browser has no registry |
| Restricted browser | Denied storage and unavailable WebGL produce explicit statuses while controls remain usable |

Run bun run build, bun run check, then bun run test. Tests use the production
bundle at /agent-cube/, matching the future GitHub Pages mount point. There are
no snapshots to bless and no mocked WebMCP registry in the native integration
test. Chromium is installed by the pinned Playwright dependency.

Additional acceptance on 2026-11-13: Codex used Playwright CLI 0.1.24 with native
WebMCP enabled in Chrome 157 to discover set_camera_view, call it with front,
read get_preview_state, and inspect the selected Front button. The automated
suite uses Chrome for Testing 158.0.8127.18 (Playwright 1.65.0).

This is a coding-agent tool invocation check, not a benchmark of an autonomous
solver or proof of natural-language tool selection across different agent clients.
Cube-engine, transactional-storage, and multiplayer tests arrive with those
implementations.
