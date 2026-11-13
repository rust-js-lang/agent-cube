// Mutate one renderer/storage boundary at a time; restore and rebuild in all cases.
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
const root = resolve(import.meta.dir, "..");
const cases = [
  { file: "session.rs", find: "if solved {", replacement: "if false && solved {", test: "a submitted solution celebrates immediately", assertion: /data-revision/ },
  { file: "session.rs", find: "self.celebrate.set(Some(revision));", replacement: "self.presentations.wait(revision).await; self.celebrate.set(Some(revision));", test: "a submitted solution celebrates immediately", assertion: /turning|Expected: false/ },
  { file: "preview.rs", find: "orbit.distance * pitch.cos() * orbit.yaw.sin(),\n            orbit.distance * pitch.sin(),\n            orbit.distance * pitch.cos() * orbit.yaw.cos(),", replacement: "0.0, 0.0, orbit.distance,", test: "free dragging.*fallback=false", assertion: /Dragging must change actual cube pixels/ },
  { file: "preview.rs", find: "let step = PI / 4.0;", replacement: "let step = PI / 2.0;", test: "Snap drags freely", assertion: /toBeCloseTo/ },
  { file: "preview.rs", find: "delta * self.pace.get()", replacement: "delta", test: "rapid human clicks", assertion: /A fast click burst must rotate faster/ },
  { file: "App.rs", find: "snapshot.solved && snapshot.move_count > 0", replacement: "snapshot.move_count > 0", test: "Replay appears only", assertion: /Expected: 0/ },
  { file: "preview.rs", find: "Some(&()),", replacement: "Some(&(view,)),", test: "without recreating WebGL", assertion: /toBe\(calls\)|The cube must contain real colored sticker pixels/ },
  { file: "storage.rs", find: "completion.await?;", replacement: "let _ = completion.await;", test: "an aborted transaction", assertion: /Expected: true/ },
  { file: "preview.rs", find: "f64::from(normal[0]) * angle,\n            f64::from(normal[1]) * angle,\n            f64::from(normal[2]) * angle,", replacement: "0.0, 0.0, 0.0,", test: "the middle of a face turn", assertion: /A halfway turn must move actual cubie pixels/ },
  { file: "preview.rs", find: "Some(&(step.id, *problem))", replacement: "Some(&(step.snapshot.revision, *problem))", test: "a 22-move solution replays", assertion: /Expected: \"replay\"/ },
];
async function run(command: string[], timeout: number) {
  const child = Bun.spawn(command, { cwd: root, stdout: "pipe", stderr: "pipe" });
  const timer = setTimeout(() => child.kill(), timeout);
  const [out, err, code] = await Promise.all([new Response(child.stdout).text(), new Response(child.stderr).text(), child.exited]);
  clearTimeout(timer);
  return { code, text: out + err };
}
try {
  for (const mutation of cases) {
    const path = resolve(root, "crates/web/src", mutation.file);
    const original = readFileSync(path, "utf8");
    if (original.split(mutation.find).length !== 2) throw Error(`Mutation site changed: ${mutation.file}`);
    try {
      writeFileSync(path, original.replace(mutation.find, mutation.replacement));
      const build = await run(["bun", "run", "build"], 120_000);
      if (build.code !== 0) throw Error(build.text);
      const result = await run(["bunx", "--bun", "playwright", "test", "-g", mutation.test], 30_000);
      if (result.code === 0 || !mutation.assertion.test(result.text)) throw Error(`Mutation was not caught by the expected assertion:\n${result.text}`);
      console.log(`caught: ${mutation.file} — ${mutation.test}`);
    } finally { writeFileSync(path, original); }
  }
} finally {
  const build = await run(["bun", "run", "build"], 120_000);
  if (build.code !== 0) throw Error(`Restoring the production build failed:\n${build.text}`);
}
