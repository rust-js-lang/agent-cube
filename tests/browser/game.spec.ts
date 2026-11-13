import { expect, test, type Page } from "@playwright/test";
import { PNG } from "pngjs";
import ReferenceCube from "../reference/cubejs/index.js";

const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const palette = [0xf1efdf, 0xe45e40, 0x95c975, 0xe9d35f, 0xf1a84b, 0x649cbf];
type State = { facelets: string; faceOrder: string; revision: number; moveCount: number; solved: boolean };

async function read(page: Page): Promise<State> {
  const result = await page.webmcp.callTool("get_cube_state");
  expect(result.isError).toBe(false);
  return JSON.parse(result.content[0].text);
}

async function visibleState(page: Page, state: State) {
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", String(state.revision));
  expect(await page.locator(".cube-net [data-color]").evaluateAll(nodes => nodes.map(node => node.getAttribute("data-color")).join(""))).toBe(state.facelets);
}

async function cubePixels(page: Page, minColors = 1) {
  const buffer = await page.locator(".cube-preview").screenshot();
  const png = PNG.sync.read(buffer);
  const counts = new Map<number, number>();
  for (let i = 0; i < png.data.length; i += 4) {
    const rgb = (png.data[i] << 16) | (png.data[i + 1] << 8) | png.data[i + 2];
    if (palette.includes(rgb)) counts.set(rgb, (counts.get(rgb) ?? 0) + 1);
  }
  expect([...counts.values()].reduce((sum, count) => sum + count, 0), "The cube must contain real colored sticker pixels, not a blank/crashed canvas or error text").toBeGreaterThan(2000);
  expect([...counts.values()].filter(count => count > 100).length).toBeGreaterThanOrEqual(minColors);
}

test("real cube pixels survive repeated camera changes without recreating WebGL", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.addInitScript(() => {
    let calls = 0;
    const original = HTMLCanvasElement.prototype.getContext;
    Object.defineProperty(window, "__webglCalls", { get: () => calls });
    HTMLCanvasElement.prototype.getContext = function(type: string, options?: unknown) {
      if (type === "webgl2") calls++;
      return Reflect.apply(original, this, [type, options]);
    } as typeof original;
  });
  await page.goto("./");
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-renderer", "webgl");
  await cubePixels(page, 3);
  const calls = await page.evaluate(() => Reflect.get(window, "__webglCalls"));
  expect(calls).toBeGreaterThan(0);
  for (let i = 0; i < 10; i++) {
    await page.getByRole("button", { name: "Front", exact: true }).click();
    await cubePixels(page);
    await page.getByRole("button", { name: "Perspective", exact: true }).click();
  }
  await cubePixels(page, 3);
  await page.getByRole("button", { name: "Turn R", exact: true }).click();
  await page.getByRole("button", { name: "Turn R'", exact: true }).click();
  await cubePixels(page, 3);
  expect(await page.evaluate(() => Reflect.get(window, "__webglCalls"))).toBe(calls);
  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect(errors).toEqual([]);
});

test("WebMCP discovers game tools and all 18 turns agree with the independent oracle", async ({ page }) => {
  await page.goto("./");
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  const tools = await page.webmcp.tools();
  expect(tools.map(tool => tool.name)).toEqual(["apply_moves", "get_cube_state", "set_camera_view", "shuffle_cube"]);
  expect(tools.find(tool => tool.name === "get_cube_state")?.annotations?.readOnly).toBe(true);
  const reference = new ReferenceCube();
  let state = await read(page);
  expect(state.facelets).toBe(solved);
  for (const face of "URFDLB") {
    for (const suffix of ["", "'", "2"]) {
      const move = face + suffix;
      const result = await page.webmcp.callTool("apply_moves", { moves: move, expectedRevision: state.revision });
      expect(result.isError).toBe(false);
      state = JSON.parse(result.content[0].text);
      reference.move(move);
      expect(state.facelets).toBe(reference.asString());
      await visibleState(page, state);
    }
  }
  expect(state.moveCount).toBe(18);
});

test("an agent can shuffle 20 moves, inspect state, and solve only through WebMCP", async ({ page }) => {
  await page.goto("./");
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  const shuffled = await page.webmcp.callTool("shuffle_cube", { length: 20 });
  expect(shuffled.isError).toBe(false);
  const state = await read(page);
  expect(state.solved).toBe(false);
  expect(Object.keys(state).sort()).toEqual(["faceOrder", "facelets", "moveCount", "revision", "solved"]);
  await visibleState(page, state);
  // Test-side solver sees only the observed facelets; no scramble, seed, or app internals.
  ReferenceCube.initSolver();
  const solution = ReferenceCube.fromString(state.facelets).solve();
  const result = await page.webmcp.callTool("apply_moves", { moves: solution, expectedRevision: state.revision });
  expect(result.isError).toBe(false);
  const final = await read(page);
  expect(final.solved).toBe(true);
  expect(final.facelets).toBe(solved);
  expect(final.revision).toBe(state.revision + 1);
  await visibleState(page, final);
  await expect(page.getByTestId("cube-status")).toContainText("Solved");
  await cubePixels(page, 3);
});

test("invalid batches, extra input, and stale or concurrent commands do not corrupt state", async ({ page }) => {
  await page.goto("./");
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  for (const input of [
    { moves: "R invalid U", expectedRevision: 0 },
    { moves: "", expectedRevision: 0 },
    { moves: "R", expectedRevision: 0.5 },
    { moves: "R", expectedRevision: 0, extra: true },
    { moves: "R ".repeat(129), expectedRevision: 0 },
    { moves: "R", expectedRevision: 1 },
  ]) {
    const result = await page.webmcp.callTool("apply_moves", input);
    expect(result.isError).toBe(true);
    expect((await read(page)).facelets).toBe(solved);
    expect((await read(page)).revision).toBe(0);
  }
  for (const length of [0, 101, 1.5, null, "20"]) {
    expect((await page.webmcp.callTool("shuffle_cube", { length })).isError).toBe(true);
    expect((await read(page)).revision).toBe(0);
  }
  const results = await Promise.all(["R", "U"].map(moves => page.webmcp.callTool("apply_moves", { moves, expectedRevision: 0 })));
  expect(results.filter(result => !result.isError)).toHaveLength(1);
  expect(results.filter(result => result.isError)).toHaveLength(1);
  const state = await read(page);
  expect(state.revision).toBe(1);
  expect(state.moveCount).toBe(1);
  await visibleState(page, state);
});

test("WebGL creation failure hides the broken canvas and renders playable stickers", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.addInitScript(() => {
    Object.defineProperty(window, "indexedDB", { get() { throw new DOMException("Storage blocked", "SecurityError"); } });
    Object.defineProperty(HTMLCanvasElement.prototype, "getContext", { value() { return null; } });
  });
  await page.goto("./");
  await expect(page.getByTestId("storage-status")).toHaveText("Unavailable");
  await expect(page.locator("canvas")).toBeHidden();
  await expect(page.locator(".compatible-cube [data-color]")).toHaveCount(54);
  await cubePixels(page, 3);
  await page.getByRole("button", { name: "Turn R", exact: true }).click();
  const state = await read(page);
  expect(state.facelets).toBe(new ReferenceCube().move("R").asString());
  await visibleState(page, state);
  expect(await page.locator(".compatible-cube [data-color]").evaluateAll(nodes => nodes.map(node => node.getAttribute("data-color")).join(""))).toBe(state.facelets);
  await cubePixels(page, 3);
  expect(errors).toEqual([]);
});

test("actual graphics context loss switches to a working compatible cube", async ({ page }) => {
  await page.goto("./");
  await cubePixels(page, 3);
  await page.locator("canvas").evaluate(canvas => {
    const gl = canvas.getContext("webgl2");
    const extension = gl?.getExtension("WEBGL_lose_context");
    if (!extension) throw new Error("Test requires real context-loss support");
    extension.loseContext();
  });
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-renderer", "compatible");
  await expect(page.locator("canvas")).toBeHidden();
  await cubePixels(page, 3);
  await page.getByRole("button", { name: "Shuffle cube ↗", exact: true }).click();
  expect((await read(page)).solved).toBe(false);
  await cubePixels(page, 3);
});

test("manual moves work without WebMCP", async ({ page }) => {
  await page.addInitScript(() => Object.defineProperty(document, "modelContext", { value: undefined }));
  await page.goto("./");
  await expect(page.getByTestId("agent-status")).toHaveText("Not available");
  await page.getByRole("button", { name: "Turn R", exact: true }).click();
  await expect(page.getByTestId("cube-status")).toContainText("In progress");
  await page.getByRole("button", { name: "Turn R'", exact: true }).click();
  await expect(page.getByTestId("cube-status")).toContainText("Solved");
  await cubePixels(page, 3);
});
