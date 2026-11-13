import { expect, test } from "@playwright/test";
import { PNG } from "pngjs";
import ReferenceCube from "../reference/cubejs/index.js";
import { controlFrames, frame, initialUrl, read } from "./helpers";
const challenge = new ReferenceCube().move("R").asString();
const challengeUrl = "./#cube=" + challenge;

test.beforeEach(async ({ page }) => { await controlFrames(page); });

test("a submitted solution celebrates immediately while cube animation still runs", async ({ page }, testInfo) => {
  await page.goto(challengeUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  let returned = false;
  const pending = page.webmcp.callTool("apply_moves", { commandId: "celebrate-solution", expectedRevision: 1, moves: "R'" }).then(result => { returned = true; return result; });
  await expect(page.locator(".confetti")).toHaveAttribute("data-revision", "2");
  await expect(page.locator(".confetti span")).toHaveCount(48);
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", challenge);
  expect(returned).toBe(false);
  // Freeze only decorative CSS animations at a visible point for an actual pixel check.
  await page.locator(".confetti span").evaluateAll(nodes => { for (const node of nodes) for (const animation of node.getAnimations()) { animation.pause(); animation.currentTime = 800; } });
  const a = PNG.sync.read(await page.screenshot({ path: testInfo.outputPath("confetti.png") }));
  const b = PNG.sync.read(await page.screenshot({ style: ".confetti{visibility:hidden!important}" }));
  let changed = 0;
  for (let i = 0; i < a.data.length; i += 4) if (Math.abs(a.data[i] - b.data[i]) + Math.abs(a.data[i+1] - b.data[i+1]) + Math.abs(a.data[i+2] - b.data[i+2]) > 60) changed++;
  expect(changed, "Confetti must paint visible celebratory pixels before the cube finishes").toBeGreaterThan(300);
  expect(await page.locator(".confetti").evaluate(node => getComputedStyle(node).pointerEvents)).toBe("none");
  await frame(page, 0); await frame(page, 500);
  expect((await pending).isError).toBe(false);
  expect((await read(page)).solved).toBe(true);
  await expect(page.locator(".confetti")).toHaveCount(0, { timeout: 4000 });
  expect((await page.webmcp.callTool("apply_moves", { commandId: "celebrate-solution", expectedRevision: 1, moves: "R'" })).isError).toBe(false);
  await expect(page.locator(".confetti")).toHaveCount(0);
  await page.getByRole("button", { name: "Replay 1 move", exact: true }).click();
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "replay");
  await frame(page, 1000); await frame(page, 1500);
  await expect(page.locator("canvas")).toHaveAttribute("data-playback", "live");
  await expect(page.locator(".confetti")).toHaveCount(0);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  await expect(page.locator(".confetti")).toHaveCount(0);
});

test("invalid, unsolved and only-intermediately-solved batches never celebrate", async ({ page }) => {
  await page.goto(challengeUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  for (const moves of ["R' invalid", ""]) {
    expect((await page.webmcp.callTool("apply_moves", { commandId: crypto.randomUUID(), expectedRevision: 1, moves })).isError).toBe(true);
    await expect(page.locator(".confetti")).toHaveCount(0);
  }
  const pending = page.webmcp.callTool("apply_moves", { commandId: "intermediate-solved", expectedRevision: 1, moves: "R' U" });
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await expect(page.locator(".confetti")).toHaveCount(0);
  await frame(page, 0); await frame(page, 500);
  expect((await pending).isError).toBe(false);
  expect((await read(page)).solved).toBe(false);
  await expect(page.locator(".confetti")).toHaveCount(0);
});

test("human solutions celebrate immediately and new moves dismiss the burst", async ({ page }) => {
  await page.goto(challengeUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  await page.getByRole("button", { name: "Turn R'", exact: true }).click();
  await expect(page.locator(".confetti")).toHaveAttribute("data-revision", "2");
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await page.getByRole("button", { name: "Turn U", exact: true }).click();
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "3");
  await expect(page.locator(".confetti")).toHaveCount(0);
});

test("reduced motion suppresses confetti while accepting the solved result", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(challengeUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  const result = await page.webmcp.callTool("apply_moves", { commandId: "quiet-solve", expectedRevision: 1, moves: "R'" });
  expect(result.isError).toBe(false);
  await expect(page.locator(".confetti")).toBeHidden();
  expect((await read(page)).solved).toBe(true);
  expect(await page.evaluate(() => Reflect.get(window, "__pendingFrames")())).toBe(0);
});

test("solved imports do not celebrate and fallback completion survives revision offsets", async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(HTMLCanvasElement.prototype, "getContext", { value() { return null; } }));
  await page.goto(challengeUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  await page.goto(initialUrl);
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "2");
  await expect(page.locator(".confetti")).toHaveCount(0);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect((await read(page)).solved).toBe(true);
  expect((await page.webmcp.callTool("apply_moves", { commandId: "fallback-unsolve", expectedRevision: 2, moves: "R" })).isError).toBe(false);
  expect((await page.webmcp.callTool("apply_moves", { commandId: "fallback-solve", expectedRevision: 3, moves: "R'" })).isError).toBe(false);
  await expect(page.locator(".confetti")).toHaveAttribute("data-revision", "4");
});
