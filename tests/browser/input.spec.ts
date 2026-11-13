import { expect, test } from "@playwright/test";
import ReferenceCube from "../reference/cubejs/index.js";
import { controlFrames, frame, initialUrl, read, solved } from "./helpers";

test.beforeEach(async ({ page }) => {
  await controlFrames(page);
  await page.goto(initialUrl);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
});

test("shuffle rotates its full scramble and finishes at the committed zero-move challenge", async ({ page }) => {
  const first = page.webmcp.callTool("apply_moves", { commandId: "before-shuffle", expectedRevision: 0, moves: "R" });
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await frame(page, 0); await frame(page, 500);
  expect((await first).isError).toBe(false);
  let returned = false;
  const shuffled = page.webmcp.callTool("shuffle_cube", { commandId: "animated-shuffle", expectedRevision: 1, length: 20 }).then(result => { returned = true; return result; });
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "2");
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", solved);
  await frame(page, 1000); await frame(page, 1040);
  expect(Math.abs(Number(await page.locator("canvas").getAttribute("data-turn-angle")))).toBeGreaterThan(0.01);
  expect(returned).toBe(false);
  await frame(page, 3500);
  const result = await shuffled;
  expect(result.isError).toBe(false);
  const snapshot = JSON.parse(result.content[0].text);
  expect(snapshot.moveCount).toBe(0);
  expect(snapshot.solved).toBe(false);
  await expect(page.locator("canvas")).toHaveAttribute("data-completed-turns", "21");
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", snapshot.facelets);
  await expect(page.locator(".replay-button")).toHaveCount(0);
});

test("rapid human clicks are all accepted in order and speed up the active and queued turns", async ({ page }) => {
  const moves = ["R", "U", "F", "R'", "U'", "F'", "R2", "D", "L", "B2", "D'", "L'"];
  await page.locator(".move-controls").evaluate((root, moves) => {
    for (const move of moves) {
      const button = [...root.querySelectorAll("button")].find(button => button.getAttribute("aria-label") === `Turn ${move}`);
      if (!button) throw Error(`Missing move control ${move}`);
      button.click();
    }
  }, moves);
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "12");
  await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
  await expect(page.getByRole("alert")).toHaveCount(0);
  await frame(page, 0); await frame(page, 30);
  expect(Math.abs(Number(await page.locator("canvas").getAttribute("data-turn-angle"))), "A fast click burst must rotate faster than the normal 240 ms turn").toBeGreaterThan(0.8);
  await frame(page, 60);
  await expect(page.locator("canvas")).toHaveAttribute("data-visual-revision", "1");
  for (let revision = 2; revision <= 12; revision++) {
    await expect(page.locator("canvas")).toHaveAttribute("data-motion", "turning");
    await frame(page, revision * 1000); await frame(page, revision * 1000 + 60);
    await expect(page.locator("canvas")).toHaveAttribute("data-visual-revision", String(revision));
  }
  const snapshot = await read(page);
  expect(snapshot.moveCount).toBe(12);
  expect(snapshot.facelets).toBe(new ReferenceCube().move(moves.join(" ")).asString());
  await expect(page.locator("canvas")).toHaveAttribute("data-presented-facelets", snapshot.facelets);
});
