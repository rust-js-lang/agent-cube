import { expect, test, type Page } from "@playwright/test";
import ReferenceCube from "../reference/cubejs/index.js";
import { controlFrames, frame, initialUrl, read, savedText, solved, visibleState } from "./helpers";

const challenge = "LFLLULBFURFFLRDDUDRDFDFDFBLRRBUDRUUBFBUFLBBBDURDRBURLL";
const solution = "U R' F B2 R' F U2 R B D2 R2 L2 F2 U R2 L2 U2 F2 D' B2 U B2";

async function ready(page: Page, url = initialUrl) {
  await page.goto(url);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
}
async function play(page: Page, moves: string, revision: number, time: number) {
  const pending = page.webmcp.callTool("apply_moves", { commandId: crypto.randomUUID(), expectedRevision: revision, moves });
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, time);
  await frame(page, time + 2500);
  const result = await pending;
  expect(result.isError).toBe(false);
  return JSON.parse(result.content[0].text);
}
async function finishReplay(page: Page, time: number) {
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "replay");
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, time);
  await frame(page, time + 60_001);
  await expect(page.getByRole("button", { name: /^Replay \d+ moves?$/ })).toBeEnabled();
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "live");
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "idle");
}

test.beforeEach(async ({ page }) => { await controlFrames(page); });

test("replay is unavailable until the current challenge has played moves", async ({ page }) => {
  await ready(page);
  await expect(page.getByRole("button", { name: "Replay moves", exact: true })).toHaveCount(0);
});

test("a 22-move solution replays from its challenge start, repeats, and survives reload without changing game data", async ({ page }) => {
  await ready(page, `./#cube=${challenge}`);
  const snapshot = await play(page, solution, 1, 0);
  expect(snapshot.solved).toBe(true);
  expect(snapshot.moveCount).toBe(22);
  const url = page.url();
  const saved = await savedText(page);
  const replay = page.getByRole("button", { name: "Replay 22 moves", exact: true });
  await replay.click();
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "replay");
  await expect(page.getByRole("button", { name: "Replaying…", exact: true })).toBeDisabled();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", challenge);
  await visibleState(page, snapshot);
  await frame(page, 3000);
  await frame(page, 3120);
  expect(Math.abs(Number(await page.locator("canvas").getAttribute("data-turn-angle")))).toBeGreaterThan(0.1);
  await frame(page, 5500);
  // Replay stays readable beyond the live-command 2.4-second fast-play budget.
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  expect(await read(page)).toEqual(snapshot);
  expect(page.url()).toBe(url);
  expect(await savedText(page)).toBe(saved);
  await frame(page, 13_000);
  await expect(replay).toBeEnabled();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", solved);
  expect(await read(page)).toEqual(snapshot);
  expect(await savedText(page)).toBe(saved);
  // Presentation IDs must differ even though game revision and endpoint are unchanged.
  await replay.click();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", challenge);
  await finishReplay(page, 20_000);
  expect(await read(page)).toEqual(snapshot);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  await replay.click();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", challenge);
  await finishReplay(page, 0);
  expect(await savedText(page)).toBe(saved);
  expect(await read(page)).toEqual(snapshot);
  expect(page.url()).toBe(url);
});

test("replay joins separate command batches and retries appear only once", async ({ page }) => {
  await ready(page);
  const first = { commandId: "replay-once", expectedRevision: 0, moves: "R U" };
  const pending = page.webmcp.callTool("apply_moves", first);
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 0); await frame(page, 2500);
  expect((await pending).isError).toBe(false);
  expect((await page.webmcp.callTool("apply_moves", first)).isError).toBe(false);
  const snapshot = await play(page, "U' R'", 1, 3000);
  await page.getByRole("button", { name: "Replay 4 moves", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", solved);
  await finishReplay(page, 6000);
  expect(await read(page)).toEqual(snapshot);
  await expect(page.locator("canvas")).toHaveAttribute("data-completed-turns", "8");
});

test("new committed moves cancel replay and animate from live state", async ({ page }) => {
  await ready(page);
  const before = await play(page, "R R'", 0, 0);
  await page.getByRole("button", { name: "Replay 2 moves", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "replay");
  await frame(page, 3000); await frame(page, 3080);
  const pending = page.webmcp.callTool("apply_moves", { commandId: "interrupt-replay", expectedRevision: 1, moves: "F" });
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "2");
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "live");
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", before.facelets);
  await frame(page, 4000); await frame(page, 4500);
  expect((await pending).isError).toBe(false);
  const current = await read(page);
  expect(current.moveCount).toBe(3);
  expect(current.facelets).toBe(new ReferenceCube().move("F").asString());
  await expect(page.locator(".replay-button")).toHaveCount(0);
});

test("a new shuffle or shared challenge resets replay history", async ({ page }) => {
  await ready(page);
  await play(page, "R R'", 0, 0);
  await page.getByRole("button", { name: "Replay 2 moves", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "replay");
  const shuffled = page.webmcp.callTool("shuffle_cube", { commandId: "new-challenge", expectedRevision: 1, length: 20 });
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", solved);
  await frame(page, 4000); await frame(page, 6500);
  expect((await shuffled).isError).toBe(false);
  await expect(page.locator(".replay-button")).toHaveCount(0);
  await play(page, "R", 2, 3000);
  await page.goto(initialUrl);
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "4");
  await expect(page.locator(".replay-button")).toHaveCount(0);
  await play(page, "B2 B2", 4, 6000);
  await page.getByRole("button", { name: "Replay 2 moves", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", solved);
  await finishReplay(page, 9000);
});

test("reduced motion keeps replay read-only and never schedules frames", async ({ page }) => {
  await ready(page);
  const snapshot = await play(page, "R R'", 0, 0);
  const saved = await savedText(page);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.getByRole("button", { name: "Replay 2 moves", exact: true }).click();
  await expect(page.getByRole("button", { name: "Replay 2 moves", exact: true })).toBeEnabled();
  expect(await page.evaluate(() => Reflect.get(window, "__pendingFrames")())).toBe(0);
  expect(await read(page)).toEqual(snapshot);
  expect(await savedText(page)).toBe(saved);
});

test("graphics loss during replay restores the committed position and releases the replay button", async ({ page }) => {
  await ready(page);
  const snapshot = await play(page, "R R'", 0, 0);
  const saved = await savedText(page);
  await page.getByRole("button", { name: "Replay 2 moves", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "replay");
  await frame(page, 3000); await frame(page, 3080);
  await page.locator("canvas").evaluate(canvas => {
    const extension = canvas.getContext("webgl2")?.getExtension("WEBGL_lose_context");
    if (!extension) throw Error("Actual context loss required");
    extension.loseContext();
  });
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-renderer", "compatible");
  await expect(page.getByRole("button", { name: "Replay 2 moves", exact: true })).toBeEnabled();
  expect(await read(page)).toEqual(snapshot);
  expect(await savedText(page)).toBe(saved);
  expect(await page.locator(".compatible-cube [data-color]").evaluateAll(nodes => nodes.map(node => node.getAttribute("data-color")).join(""))).toBe(snapshot.facelets);
});

test("replay is available to human controls without WebMCP", async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(document, "modelContext", { value: undefined }));
  await page.goto(initialUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Not available");
  await page.getByRole("button", { name: "Turn R", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 0); await frame(page, 500);
  await expect(page.locator(".replay-button")).toHaveCount(0);
  await page.getByRole("button", { name: "Turn R'", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 1000); await frame(page, 1500);
  await page.getByRole("button", { name: "Replay 2 moves", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", solved);
  await finishReplay(page, 2000);
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "2");
  await expect(page.getByTestId("cube-status")).toContainText("2 moves");
});

test("temporary sessions can replay until reload discards their history", async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(window, "indexedDB", { get() { throw new DOMException("blocked", "SecurityError"); } }));
  await ready(page);
  await expect(page.getByTestId("storage-status")).toHaveText("Temporary · history not saved");
  const snapshot = await play(page, "R R'", 0, 0);
  await page.getByRole("button", { name: "Replay 2 moves", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", solved);
  await finishReplay(page, 3000);
  expect(await read(page)).toEqual(snapshot);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect((await read(page)).facelets).toBe(snapshot.facelets);
  expect((await read(page)).moveCount).toBe(0);
  await expect(page.locator(".replay-button")).toHaveCount(0);
});


test("Replay appears only for a solved challenge with recorded moves", async ({ page }) => {
  await ready(page);
  await expect(page.locator(".replay-button")).toHaveCount(0);
  await play(page, "R", 0, 0);
  await expect(page.locator(".replay-button")).toHaveCount(0);
  await play(page, "R'", 1, 1000);
  await expect(page.getByRole("button", { name: "Replay 2 moves", exact: true })).toBeEnabled();
});
