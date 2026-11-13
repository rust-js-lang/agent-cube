import { expect, test } from "@playwright/test";

test("Rust UI renders a real cube, changes camera, and probes IndexedDB", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto("./");
  await expect(page.getByRole("heading", { level: 1 })).toContainText("A little twist.");
  await expect(page.getByTestId("storage-status")).toHaveText("Available");
  await expect(page.getByRole("img", { name: /Solved cube preview/ })).toBeVisible();
  const preview = page.locator("canvas");
  const before = await preview.screenshot();
  await page.getByRole("button", { name: "Front", exact: true }).click();
  await expect(page.getByRole("button", { name: "Front", exact: true })).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-view", "front");
  expect((await preview.screenshot()).equals(before)).toBe(false);
  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect(errors).toEqual([]);
});

test("Playwright WebMCP tools change visible state and reject invalid input", async ({ page }) => {
  await page.goto("./");
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 2 tools");
  const tools = await page.webmcp.tools();
  expect(tools.map(tool => tool.name)).toEqual(["get_preview_state", "set_camera_view"]);
  expect(tools.find(tool => tool.name === "get_preview_state")?.annotations?.readOnly).toBe(true);
  const changed = await page.webmcp.callTool("set_camera_view", { view: "front" });
  expect(changed).toEqual({ content: [{ type: "text", text: "Camera view: front." }], isError: false });
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-view", "front");
  const observed = await page.webmcp.callTool("get_preview_state");
  expect(JSON.stringify(observed)).toContain("Camera view: front.");
  const invalid = await page.webmcp.callTool("set_camera_view", { view: "upside-down" });
  expect(invalid.isError).toBe(true);
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-view", "front");
  const extra = await page.webmcp.callTool("set_camera_view", { view: "perspective", extra: true });
  expect(extra.isError).toBe(true);
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-view", "front");
  await page.getByRole("button", { name: "Perspective", exact: true }).click();
  expect(JSON.stringify(await page.webmcp.callTool("get_preview_state"))).toContain("Camera view: perspective.");
});

test("manual controls work without WebMCP", async ({ browser }) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.addInitScript(() => Object.defineProperty(document, "modelContext", { value: undefined }));
  await page.goto("http://127.0.0.1:4173/agent-cube/");
  await expect(page.getByTestId("agent-status")).toHaveText("Not available");
  await page.getByRole("button", { name: "Front", exact: true }).click();
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-view", "front");
  await context.close();
});

test("restricted storage and unavailable WebGL leave the controls usable", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.addInitScript(() => {
    Object.defineProperty(window, "indexedDB", { get() { throw new DOMException("Storage blocked", "SecurityError"); } });
    Object.defineProperty(HTMLCanvasElement.prototype, "getContext", { value() { return null; } });
  });
  await page.goto("./");
  await expect(page.getByTestId("storage-status")).toHaveText("Unavailable");
  await expect(page.getByRole("alert")).toContainText("needs WebGL");
  await page.getByRole("button", { name: "Front", exact: true }).click();
  await expect(page.locator(".cube-preview")).toHaveAttribute("data-view", "front");
  expect(errors).toEqual([]);
});
