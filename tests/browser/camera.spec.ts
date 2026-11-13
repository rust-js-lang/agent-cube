import { PNG } from "pngjs";
import { expect, test, type Page } from "@playwright/test";
import { call, controlFrames, frame, initialUrl, read, savedText } from "./helpers";

async function angles(page: Page) {
  return page.getByRole("group", { name: "Rotate cube view", exact: true }).evaluate(node => ({ yaw: Number(node.getAttribute("data-yaw")), pitch: Number(node.getAttribute("data-pitch")) }));
}
async function center(page: Page) {
  await page.getByRole("group", { name: "Rotate cube view", exact: true }).scrollIntoViewIfNeeded();
  const box = await page.getByRole("group", { name: "Rotate cube view", exact: true }).boundingBox();
  if (!box) throw Error("Cube view must be visible");
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
}
async function drag(page: Page, dx: number, dy: number) {
  const point = await center(page);
  await page.mouse.move(point.x, point.y);
  await page.mouse.down();
  await page.mouse.move(point.x + dx, point.y + dy, { steps: 8 });
  await page.mouse.up();
}

for (const fallback of [false, true]) {
  test("free dragging changes horizontal and vertical viewpoint without changing the game (fallback=" + fallback + ")", async ({ page }) => {
    if (fallback) await page.addInitScript(() => Object.defineProperty(HTMLCanvasElement.prototype, "getContext", { value() { return null; } }));
    await page.goto(initialUrl);
    await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
    await page.getByRole("button", { name: "Front", exact: true }).click();
    const state = await read(page), saved = await savedText(page), url = page.url();
    const target = page.locator(fallback ? ".compatible-stage" : "canvas");
    const before = await target.screenshot();
    await drag(page, 90, 60);
    const rotated = await angles(page);
    expect(rotated.yaw).toBeCloseTo(2 * Math.PI - 0.72);
    expect(rotated.pitch).toBeCloseTo(0.48);
    const a = PNG.sync.read(before), b = PNG.sync.read(await target.screenshot());
    let changed = 0;
    for (let i = 0; i < a.data.length; i += 4) {
      if (Math.abs(a.data[i] - b.data[i]) + Math.abs(a.data[i + 1] - b.data[i + 1]) + Math.abs(a.data[i + 2] - b.data[i + 2]) > 60) changed++;
    }
    expect(changed, "Dragging must change actual cube pixels").toBeGreaterThan(4000);
    await drag(page, -90, -60);
    expect((await angles(page)).yaw % (2 * Math.PI)).toBeCloseTo(0);
    expect((await angles(page)).pitch).toBeCloseTo(0);
    await drag(page, 90, -60);
    // Clicking the already-selected preset must reset a custom orbit too.
    await page.getByRole("button", { name: "Front", exact: true }).click();
    await expect.poll(() => angles(page)).toEqual({ yaw: 0, pitch: 0 });
    expect(await read(page)).toEqual(state);
    expect(await savedText(page)).toBe(saved);
    expect(page.url()).toBe(url);
  });
}

test("Snap drags freely then settles to the nearest fixed angle; keyboard and agent presets remain usable", async ({ page }) => {
  await page.goto(initialUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  await page.getByRole("button", { name: "Front", exact: true }).click();
  await page.getByRole("button", { name: "Snap", exact: true }).click();
  const point = await center(page);
  await page.mouse.move(point.x, point.y); await page.mouse.down();
  await page.mouse.move(point.x - 70, point.y + 65, { steps: 6 });
  expect((await angles(page)).yaw).toBeCloseTo(0.56);
  await page.mouse.up();
  expect((await angles(page)).yaw).toBeCloseTo(Math.PI / 4);
  expect((await angles(page)).pitch).toBeCloseTo(Math.PI / 4);
  const view = page.getByRole("group", { name: "Rotate cube view", exact: true });
  await view.press("ArrowLeft");
  expect((await angles(page)).yaw).toBeCloseTo(Math.PI / 2);
  await view.press("ArrowDown");
  expect((await angles(page)).pitch).toBeCloseTo(Math.PI / 2);
  await expect(page.locator("canvas")).toBeVisible();
  expect((await call(page, "set_camera_view", { view: "front" })).isError).toBe(false);
  await expect.poll(() => angles(page)).toEqual({ yaw: 0, pitch: 0 });
  await page.getByRole("button", { name: "Free", exact: true }).click();
  await view.press("ArrowRight");
  expect((await angles(page)).yaw).toBeCloseTo(2 * Math.PI - Math.PI / 18);
  await view.press("Home");
  await expect.poll(() => angles(page)).toEqual({ yaw: 0, pitch: 0 });
  expect((await read(page)).moveCount).toBe(0);
});

test("touch drag, cancellation and release outside the cube do not leave a stuck drag", async ({ browser }) => {
  const context = await browser.newContext({ hasTouch: true });
  const page = await context.newPage();
  try {
    await page.goto(initialUrl);
    await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
    await page.getByRole("button", { name: "Front", exact: true }).click();
    const point = await center(page);
    const cdp = await context.newCDPSession(page);
    await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x: point.x, y: point.y }] });
    await cdp.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [{ x: point.x + 90, y: point.y + 50 }] });
    await expect.poll(async () => (await angles(page)).pitch).toBeCloseTo(0.4);
    await cdp.send("Input.dispatchTouchEvent", { type: "touchCancel", touchPoints: [] });
    const cancelled = await angles(page);
    await page.mouse.move(point.x - 50, point.y - 50);
    expect(await angles(page)).toEqual(cancelled);
    await drag(page, 30, 20);
    expect(await angles(page)).not.toEqual(cancelled);
    await page.mouse.move(point.x, point.y); await page.mouse.down();
    await page.mouse.move(1, 1); await page.mouse.up();
    const released = await angles(page);
    await page.mouse.move(point.x, point.y);
    expect(await angles(page)).toEqual(released);
    expect((await read(page)).moveCount).toBe(0);
  } finally { await context.close(); }
});

test("orbiting during a face turn preserves animation, notation and the exact endpoint", async ({ page }) => {
  await controlFrames(page);
  await page.goto(initialUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  const result = call(page, "apply_moves", { expectedRevision: 0, moves: "R R'" });
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 0); await frame(page, 100);
  await drag(page, -80, 40);
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 800);
  expect((await result).isError).toBe(false);
  expect((await read(page)).solved).toBe(true);
  expect((await read(page)).moveCount).toBe(2);
  await expect(page.getByRole("button", { name: "Replay 2 moves", exact: true })).toBeVisible();
});
