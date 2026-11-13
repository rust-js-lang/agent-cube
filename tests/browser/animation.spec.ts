import { expect, test, type Page } from "@playwright/test";
import { PNG } from "pngjs";
import ReferenceCube from "../reference/cubejs/index.js";
import { controlFrames, frame, initialUrl, visibleState } from "./helpers";

test.beforeEach(async ({ page }) => {
  // Control only RAF, leaving IndexedDB and React scheduling real. No arbitrary sleeps.
  await controlFrames(page);
  await page.goto(initialUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "idle");
});

function pixelChanges(first: Buffer, second: Buffer) {
  const a = PNG.sync.read(first), b = PNG.sync.read(second);
  expect([a.width, a.height]).toEqual([b.width, b.height]);
  let changed = 0;
  for (let i = 0; i < a.data.length; i += 4) {
    if (Math.abs(a.data[i] - b.data[i]) + Math.abs(a.data[i + 1] - b.data[i + 1]) + Math.abs(a.data[i + 2] - b.data[i + 2]) > 30) changed++;
  }
  return changed;
}

async function exactFinal(page: Page, facelets: string, revision: number) {
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "idle");
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", facelets);
  await expect(page.locator("canvas")).toHaveAttribute("data-visual-revision", String(revision));
}

test("the middle of a face turn visibly rotates geometry and completion snaps to the exact position", async ({ page, context }) => {
  const canvas = page.locator("canvas");
  const before = await canvas.screenshot();
  let returned = false;
  const pending = page.webmcp.callTool("apply_moves", { commandId: "mid-turn", expectedRevision: 0, moves: "R" }).then(result => { returned = true; return result; });
  await expect(canvas).toHaveAttribute("data-motion", "turning");
  // The game is already committed, but tool completion waits for actual presentation.
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "1");
  await frame(page, 0);
  await frame(page, 120);
  const angle = Number(await canvas.getAttribute("data-turn-angle"));
  expect(angle).toBeCloseTo(-Math.PI / 4, 4);
  const middle = await canvas.screenshot();
  expect(pixelChanges(before, middle), "A halfway turn must move actual cubie pixels").toBeGreaterThan(500);
  expect(returned).toBe(false);
  await frame(page, 300);
  const result = await pending;
  expect(result.isError).toBe(false);
  const state = JSON.parse(result.content[0].text);
  expect(state.facelets).toBe(new ReferenceCube().move("R").asString());
  await exactFinal(page, state.facelets, 1);
  await visibleState(page, state);
  const after = await canvas.screenshot();
  expect(pixelChanges(middle, after)).toBeGreaterThan(500);
  const reference = await context.newPage();
  await reference.goto(page.url());
  await expect(reference.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  await exactFinal(reference, state.facelets, 1);
  expect(pixelChanges(after, await reference.locator("canvas").screenshot()), "Animated endpoint must match a freshly rendered authoritative position").toBeLessThan(50);
});

test("all 18 animated turns have correct direction and land on independent reference states", async ({ page }) => {
  const reference = new ReferenceCube();
  let revision = 0;
  for (const face of "URFDLB") for (const suffix of ["", "'", "2"]) {
    const moves = face + suffix;
    const pending = page.webmcp.callTool("apply_moves", { commandId: `animated-${moves}`, expectedRevision: revision, moves });
    await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
    await frame(page, revision * 1000);
    await frame(page, revision * 1000 + 120);
    const angle = Number(await page.locator("canvas").getAttribute("data-turn-angle"));
    expect(Math.sign(angle)).toBe(suffix === "'" ? 1 : -1);
    await frame(page, revision * 1000 + 500);
    const result = await pending;
    expect(result.isError).toBe(false);
    reference.move(moves);
    revision++;
    await exactFinal(page, reference.asString(), revision);
  }
  await expect(page.locator("canvas")).toHaveAttribute("data-completed-turns", "18");
});

test("queued commands animate in commit order and retries do not repeat a turn", async ({ page }) => {
  const input = { commandId: "queued-first", expectedRevision: 0, moves: "R" };
  const first = page.webmcp.callTool("apply_moves", input);
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  const second = page.webmcp.callTool("apply_moves", { commandId: "queued-second", expectedRevision: 1, moves: "U" });
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "2");
  const retry = page.webmcp.callTool("apply_moves", input);
  await frame(page, 0);
  await frame(page, 300);
  expect((await first).isError).toBe(false);
  await expect(page.locator("canvas")).toHaveAttribute("data-visual-revision", "1");
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 1000);
  await expect(page.locator("canvas")).toHaveAttribute("data-turn", "U");
  await frame(page, 1300);
  expect((await second).isError).toBe(false);
  expect(await retry).toEqual(await first);
  await exactFinal(page, new ReferenceCube().move("R U").asString(), 2);
  await expect(page.locator("canvas")).toHaveAttribute("data-completed-turns", "2");
});

test("reduced motion presents the exact committed result without scheduling RAF", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  const result = await page.webmcp.callTool("apply_moves", { commandId: "reduced", expectedRevision: 0, moves: "R U F'" });
  expect(result.isError).toBe(false);
  await exactFinal(page, new ReferenceCube().move("R U F'").asString(), 1);
  expect(await page.evaluate(() => Reflect.get(window, "__pendingFrames")())).toBe(0);
});

test("a maximum batch uses a bounded timeline and still presents every exact turn", async ({ page }) => {
  const moves = Array.from({ length: 128 }, (_, i) => "RUF"[i % 3]).join(" ");
  const pending = page.webmcp.callTool("apply_moves", { commandId: "bounded-batch", expectedRevision: 0, moves });
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 0);
  await frame(page, 2500);
  expect((await pending).isError).toBe(false);
  await exactFinal(page, new ReferenceCube().move(moves).asString(), 1);
  await expect(page.locator("canvas")).toHaveAttribute("data-completed-turns", "128");
});

test("an animation with stalled frames completes through its presentation deadline", async ({ page }) => {
  const pending = page.webmcp.callTool("apply_moves", { commandId: "stalled-frames", expectedRevision: 0, moves: "R U" });
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "idle", { timeout: 4000 });
  expect((await pending).isError).toBe(false);
  await exactFinal(page, new ReferenceCube().move("R U").asString(), 1);
  expect(await page.evaluate(() => Reflect.get(window, "__pendingFrames")())).toBe(0);
});

for (const interruption of ["context loss", "reduced motion", "hidden page"]) {
  test(`${interruption} during a turn completes committed presentation without hanging`, async ({ page }) => {
    const pending = page.webmcp.callTool("apply_moves", { commandId: "interrupted", expectedRevision: 0, moves: "R" });
    await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
    await frame(page, 0);
    await frame(page, 80);
    if (interruption === "context loss") {
      await page.locator("canvas").evaluate(canvas => {
        const extension = canvas.getContext("webgl2")?.getExtension("WEBGL_lose_context");
        if (!extension) throw new Error("Actual context loss is required");
        extension.loseContext();
      });
      await expect(page.locator(".cube-preview")).toHaveAttribute("data-renderer", "compatible");
      await expect(page.locator("canvas")).toBeHidden();
    } else if (interruption === "reduced motion") await page.emulateMedia({ reducedMotion: "reduce" });
    else await page.evaluate(() => { Object.defineProperty(document, "hidden", { get: () => true }); document.dispatchEvent(new Event("visibilitychange")); });
    expect((await pending).isError).toBe(false);
    await exactFinal(page, new ReferenceCube().move("R").asString(), 1);
    expect(await page.evaluate(() => Reflect.get(window, "__pendingFrames")())).toBe(0);
  });
}
