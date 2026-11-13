import ReferenceCube from "./reference/cubejs/index.js";
import { Cube, Move, cubeDefault, scramble } from "../crates/cube-core/src/lib.js";

const native = Bun.spawnSync(["cargo", "run", "--locked", "--quiet", "--package", "cube-core", "--example", "conformance"], { timeout: 30_000, stdout: "pipe", stderr: "pipe" });
if (native.exitCode !== 0) throw new Error(native.stderr.toString());
const lines = native.stdout.toString().trim().split("\n");
if (lines.length !== 64) throw new Error("Native conformance probe did not run every seed");
for (const line of lines) {
  const [seed, nativeAlgorithm, expected] = line.split("\t");
  const moves = scramble(Number(seed), 100);
  if (moves.TAG !== "Ok") throw new Error(`Shuffle failed: ${seed}`);
  const algorithm = moves._0.map(Move.notation).join(" ");
  const cube = cubeDefault.default();
  for (const movement of moves._0) Cube.apply(cube, movement);
  if (algorithm !== nativeAlgorithm || Cube.facelets(cube) !== expected || new ReferenceCube().move(algorithm).asString() !== expected) {
    throw new Error(`Native / rust-js / independent reference disagreement at seed ${seed}`);
  }
}
console.log("64 seeded 100-move sequences agree: native Rust, rust-js JavaScript, and independent reference");
