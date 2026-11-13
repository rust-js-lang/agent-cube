// Plausible rule regressions must fail native tests, not merely fail to compile.
import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const workspace = Bun.TOML.parse(readFileSync(join(root, "Cargo.toml"), "utf8")) as { workspace: { package: Record<string, unknown> } };
const manifest = readFileSync(join(root, "crates/cube-core/Cargo.toml"), "utf8")
  .replace(/^(\w+)\.workspace = true$/gm, (_, name) => `${name} = ${JSON.stringify(workspace.workspace.package[name])}`)
  .replace(/\n\[lints\]\nworkspace = true\n?/, "\n");
const cases = [
  { file: "lib.rs", name: "reversed-clockwise", find: "axis, -normal[axis]", replacement: "axis, normal[axis]" },
  { file: "lib.rs", name: "stale-revision-accepted", find: "expected_revision != self.revision", replacement: "false" },
  { file: "lib.rs", name: "truncated-batch-accepted", find: "let moves = parse_moves(text)?;", replacement: "let moves = parse_moves(text.split_whitespace().next().unwrap_or(\"\")).unwrap_or_default();" },
  { file: "lib.rs", name: "impossible-permutation-accepted", find: "parity(&corners.permutation) != parity(&edges.permutation)", replacement: "false" },
  { file: "lib.rs", name: "impossible-corner-twist-accepted", find: "corners.orientation_sum % 3 != 0", replacement: "false" },
  { file: "lib.rs", name: "impossible-edge-flip-accepted", find: "edges.orientation_sum % 2 != 0", replacement: "false" },
  { file: "journal.rs", name: "retry-not-recognized", find: "entry.command.command_id == command.command_id", replacement: "false" },
  { file: "journal.rs", name: "replay-includes-prior-challenges", find: ".rposition(", replacement: ".position(" },
];

await Promise.all(cases.map(async mutation => {
  const source = readFileSync(join(root, "crates/cube-core/src", mutation.file), "utf8");
  if (!source.includes(mutation.find)) throw new Error(`Mutation no longer applies: ${mutation.name}`);
  const directory = mkdtempSync(join(tmpdir(), `agent-cube-${mutation.name}-`));
  try {
    cpSync(join(root, "crates/cube-core/src"), join(directory, "src"), { recursive: true });
    cpSync(join(root, "rust-toolchain.toml"), join(directory, "rust-toolchain.toml"));
    writeFileSync(join(directory, "Cargo.toml"), manifest);
    writeFileSync(join(directory, "src", mutation.file), source.replaceAll(mutation.find, mutation.replacement));
    const child = Bun.spawn(["cargo", "test", "--offline", "--quiet"], {
      cwd: directory,
      env: { ...process.env, RUSTC_WRAPPER: Bun.which("sccache") ?? process.env.RUSTC_WRAPPER },
      stdout: "pipe", stderr: "pipe",
    });
    const timer = setTimeout(() => child.kill(), 60_000);
    const [stdout, stderr, code] = await Promise.all([new Response(child.stdout).text(), new Response(child.stderr).text(), child.exited]);
    clearTimeout(timer);
    if (code === 0 || !stdout.includes("test result: FAILED")) {
      throw new Error(`${mutation.name} was not caught by a test failure:\n${stdout}\n${stderr}`);
    }
    console.log(`caught: ${mutation.name}`);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}));
