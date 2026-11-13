# AgentCube development

AgentCube is an ordinary Rust application compiled to JavaScript with the pinned
rust-js submodule. Read README.md and docs/architecture.md before changing scope.

- Keep application and infrastructure code in this repository for now.
- Keep cube and match rules independent of browsers, rendering, and persistence.
- Write idiomatic Rust. Fix compiler or shipped browser binding gaps in rust-js;
  do not hide them with application workarounds. npm libraries get accurate typed
  Rust declarations based on their pinned public API.
- Use Bun for JavaScript tooling. Every command must have a suitable timeout.
- Work on main. Do not create branches or pull requests.
- Keep generated .js/.jsx and .d.ts beside their Rust sources and commit them.
  Never edit generated output by hand. Source maps and build artifacts are ignored.
- Before pushing, run bun run build, bun run check, and bun run test. Read generated
  JavaScript. Run browser tests against the production bundle, including native
  Chromium WebMCP; a mocked registry alone is not sufficient proof.
- Do not claim the milestone's natural-language agent acceptance from a direct
  browser API test. Record which verification actually ran.
- Commit titles start with feat(agent-cube):, fix(agent-cube):,
  chore(agent-cube):, or perf(agent-cube):. Codex commits include
  Co-Authored-By: Codex <codex@openai.com> in their description.
- Do not change the submodule pointer without checking that its commit is
  published upstream. Changes inside rust-js follow its own AGENTS.md.
- GitHub Pages deploys on pushes to main and by manual dispatch. Keep one deploy
  workflow; no separate Check workflow. Pin the latest action releases to full
  commit hashes, with version comments. The static site does not host the future
  multiplayer server.

- Keep the tracked downstream compiler patches in patches/rust-js until their
  upstream issues are resolved. Apply them through the shared patch script; do
  not commit or push changes inside the submodule for this project.
