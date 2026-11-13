import { expect, test, type Page } from "@playwright/test";
import ReferenceCube from "../reference/cubejs/index.js";
import { call, cubePixels, initialUrl, read, solved, visibleState } from "./helpers";

async function ready(page: Page, url = initialUrl) {
  await page.goto(url);
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
}

async function savedText(page: Page) {
  return page.evaluate(async () => {
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      const request = indexedDB.open("agent-cube", 1);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    try {
      return await new Promise<string | undefined>((resolve, reject) => {
        const request = db.transaction("practice").objectStore("practice").get("active");
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });
    } finally { db.close(); }
  });
}

async function replaceSavedText(page: Page, text: string) {
  await page.evaluate(async text => {
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      const request = indexedDB.open("agent-cube", 1);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    try {
      await new Promise<void>((resolve, reject) => {
        const tx = db.transaction("practice", "readwrite");
        tx.objectStore("practice").put(text, "active");
        tx.oncomplete = () => resolve();
        tx.onabort = () => reject(tx.error);
      });
    } finally { db.close(); }
  }, text);
}

test("reload restores committed cube, move count, receipts, camera preference and URL", async ({ page }) => {
  await ready(page);
  await expect(page.getByTestId("storage-status")).toHaveText("Saved on this browser");
  const input = { commandId: "survives-reload", moves: "R U2 F'", expectedRevision: 0 };
  expect((await page.webmcp.callTool("apply_moves", input)).isError).toBe(false);
  const before = await read(page);
  const url = page.url();
  expect(new URL(url).hash).toBe(`#cube=${before.facelets}`);
  await page.getByRole("button", { name: "Front", exact: true }).click();
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect(await read(page)).toEqual(before);
  await expect(page.getByRole("button", { name: "Front", exact: true })).toHaveAttribute("aria-pressed", "true");
  expect(page.url()).toBe(url);
  const retry = await page.webmcp.callTool("apply_moves", input);
  expect(retry.isError).toBe(false);
  expect(JSON.parse(retry.content[0].text)).toEqual(before);
  expect(await read(page)).toEqual(before);
  await visibleState(page, before);
  await cubePixels(page);
});

test("lost-response retries return original receipts after later commands without rewinding the UI", async ({ page }) => {
  await ready(page);
  const input = { commandId: "lost-response", expectedRevision: 0, moves: "R" };
  const first = await page.webmcp.callTool("apply_moves", input);
  await call(page, "apply_moves", { expectedRevision: 1, moves: "U" });
  const latest = await read(page);
  const text = await savedText(page);
  const retry = await page.webmcp.callTool("apply_moves", input);
  expect(retry).toEqual(first);
  await visibleState(page, latest);
  expect(await savedText(page)).toBe(text);
  for (const changed of [{ ...input, moves: "U" }, { ...input, expectedRevision: 1 }]) {
    expect((await page.webmcp.callTool("apply_moves", changed)).isError).toBe(true);
  }
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect(await read(page)).toEqual(latest);
});

test("shuffle retries keep the original position even after reload", async ({ page }) => {
  await ready(page);
  const input = { commandId: "same-shuffle", expectedRevision: 0, length: 20 };
  const first = await page.webmcp.callTool("shuffle_cube", input);
  expect(first.isError).toBe(false);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect(await page.webmcp.callTool("shuffle_cube", input)).toEqual(first);
  expect((await read(page)).revision).toBe(1);
});

test("two tabs serialize conflicting revisions and refresh through committed notifications", async ({ page, context }) => {
  await ready(page);
  const peer = await context.newPage();
  await ready(peer, page.url());
  const results = await Promise.all([page, peer].map((target, index) => target.webmcp.callTool("apply_moves", {
    commandId: `tab-${index}`, moves: index ? "U" : "R", expectedRevision: 0,
  })));
  expect(results.filter(result => !result.isError)).toHaveLength(1);
  expect(results.filter(result => result.isError)).toHaveLength(1);
  const accepted = JSON.parse(results.find(result => !result.isError)!.content[0].text);
  // Observe without a read tool call, so only notification-driven updates can satisfy this.
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "1");
  await expect(peer.locator(".cube-net")).toHaveAttribute("data-revision", "1");
  await visibleState(page, accepted);
  await visibleState(peer, accepted);
  expect(accepted.moveCount).toBe(1);
});

test("simultaneous retries from different tabs apply a command once", async ({ page, context }) => {
  await ready(page);
  const peer = await context.newPage();
  await ready(peer, page.url());
  const input = { commandId: "shared-request", expectedRevision: 0, moves: "R U" };
  const results = await Promise.all([page, peer].map(target => target.webmcp.callTool("apply_moves", input)));
  expect(results[0].isError).toBe(false);
  expect(results[1]).toEqual(results[0]);
  const state = await read(peer);
  expect(state.revision).toBe(1);
  expect(state.moveCount).toBe(2);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect(await read(page)).toEqual(state);
});

test("an aborted transaction after a successful put never publishes or acknowledges a move", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.addInitScript(() => {
    let abortOnce = true;
    const put = IDBObjectStore.prototype.put;
    IDBObjectStore.prototype.put = function(value, key) {
      const request = Reflect.apply(put, this, [value, key]);
      if (this.name === "practice" && abortOnce) {
        abortOnce = false;
        request.addEventListener("success", () => this.transaction.abort());
      }
      return request;
    };
  });
  await ready(page);
  const before = await read(page);
  const beforeUrl = page.url();
  const input = { commandId: "retry-after-abort", expectedRevision: 0, moves: "R" };
  expect((await page.webmcp.callTool("apply_moves", input)).isError).toBe(true);
  expect(await read(page)).toEqual(before);
  await visibleState(page, before);
  expect(page.url()).toBe(beforeUrl);
  expect(await savedText(page)).toBeUndefined();
  expect((await page.webmcp.callTool("apply_moves", input)).isError).toBe(false);
  expect((await read(page)).moveCount).toBe(1);
  expect(errors).toEqual([]);
});

for (const text of ["invalid JSON", '{"version":2,"entries":[]}']) {
  test(`unreadable or future records are preserved and block moves: ${text}`, async ({ page }) => {
    await ready(page);
    await call(page, "apply_moves", { moves: "R", expectedRevision: 0 });
    await replaceSavedText(page, text);
    await page.reload();
    await expect(page.getByTestId("storage-status")).toHaveText("Recovery blocked");
    await expect(page.getByRole("button", { name: "Turn R", exact: true })).toBeDisabled();
    expect((await page.webmcp.callTool("get_cube_state")).isError).toBe(true);
    expect(await savedText(page)).toBe(text);
  });
}

test("a URL transfers the exact position to another browser with zero moves and can be solved", async ({ page, browser }) => {
  await ready(page);
  await call(page, "shuffle_cube", { length: 20 });
  await call(page, "apply_moves", { expectedRevision: 1, moves: "R U" });
  const original = await read(page);
  const link = page.url();
  expect(new URL(link).hash).toBe(`#cube=${original.facelets}`);
  const recipient = await browser.newContext();
  try {
    const other = await recipient.newPage();
    await ready(other, link);
    const imported = await read(other);
    expect(imported.facelets).toBe(original.facelets);
    expect(imported.moveCount).toBe(0);
    await visibleState(other, imported);
    ReferenceCube.initSolver();
    const solution = ReferenceCube.fromString(imported.facelets).solve();
    expect((await call(other, "apply_moves", { expectedRevision: imported.revision, moves: solution })).isError).toBe(false);
    expect((await read(other)).facelets).toBe(solved);
    expect(await read(page)).toEqual(original);
    await cubePixels(other, 3);
  } finally { await recipient.close(); }
});

test("invalid URL positions cannot overwrite saved play", async ({ page }) => {
  await ready(page);
  await call(page, "apply_moves", { expectedRevision: 0, moves: "R" });
  const before = await read(page);
  const saved = await savedText(page);
  const flipped = [...solved];
  [flipped[5], flipped[10]] = [flipped[10], flipped[5]];
  await page.goto(`./#cube=${flipped.join("")}`);
  await page.reload();
  await expect(page.getByTestId("storage-status")).toHaveText("Recovery blocked");
  await expect(page.getByRole("alert")).toContainText("impossible");
  expect(await savedText(page)).toBe(saved);
  await page.goto(`./#cube=${before.facelets}`);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect(await read(page)).toEqual(before);
});

test("copy challenge link puts only the exact position in the clipboard", async ({ page, context }) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await ready(page);
  await call(page, "apply_moves", { expectedRevision: 0, moves: "R U2" });
  await page.getByRole("button", { name: "Copy challenge link", exact: true }).click();
  await expect(page.getByRole("button", { name: "Link copied", exact: true })).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(page.url());
  const copied = page.url();
  await page.getByRole("button", { name: "Turn R", exact: true }).click();
  await expect(page.getByRole("button", { name: "Copy challenge link", exact: true })).toBeVisible();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(copied);
  expect(page.url()).not.toBe(copied);
  expect(new URL(page.url()).hash).toBe(`#cube=${(await read(page)).facelets}`);
});

test("denied storage still shares a position while reload clearly loses history", async ({ page, browser }) => {
  await page.addInitScript(() => Object.defineProperty(window, "indexedDB", { get() { throw new DOMException("blocked", "SecurityError"); } }));
  await ready(page);
  await expect(page.getByTestId("storage-status")).toHaveText("Temporary · history not saved");
  await call(page, "apply_moves", { expectedRevision: 0, moves: "R U F2" });
  const before = await read(page);
  const link = page.url();
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  const resumed = await read(page);
  expect(resumed.facelets).toBe(before.facelets);
  expect(resumed.moveCount).toBe(0);
  const context = await browser.newContext();
  try {
    const other = await context.newPage();
    await ready(other, link);
    expect((await read(other)).facelets).toBe(before.facelets);
    expect((await read(other)).moveCount).toBe(0);
  } finally { await context.close(); }
});

test("a newer database schema is preserved and never silently becomes temporary play", async ({ page }) => {
  await ready(page);
  await page.evaluate(async () => {
    await new Promise<void>((resolve, reject) => {
      const request = indexedDB.open("agent-cube", 2);
      request.onsuccess = () => { request.result.close(); resolve(); };
      request.onerror = () => reject(request.error);
    });
  });
  await page.reload();
  await expect(page.getByTestId("storage-status")).toHaveText("Recovery blocked");
  await expect(page.getByRole("alert")).toContainText("newer app");
  await expect(page.getByRole("button", { name: "Shuffle cube ↗", exact: true })).toBeDisabled();
  expect((await page.webmcp.callTool("get_cube_state")).isError).toBe(true);
});

test("native tool schemas require stable identities and invalid identities change nothing", async ({ page }) => {
  await ready(page);
  const tools = await page.webmcp.tools();
  for (const name of ["apply_moves", "shuffle_cube"]) {
    expect(tools.find(tool => tool.name === name)?.inputSchema.required).toContain("commandId");
    expect(tools.find(tool => tool.name === name)?.inputSchema.required).toContain("expectedRevision");
  }
  for (const commandId of ["", "x".repeat(129), "🦀", null, 42]) {
    const result = await page.webmcp.callTool("apply_moves", { commandId, expectedRevision: 0, moves: "R" });
    expect(result.isError).toBe(true);
    expect((await read(page)).revision).toBe(0);
  }
});

for (const temporary of [false, true]) {
  test(`a bare visit starts a saved, shareable random shuffle (temporary=${temporary})`, async ({ page }) => {
    if (temporary) await page.addInitScript(() => Object.defineProperty(window, "indexedDB", { get() { throw new DOMException("blocked", "SecurityError"); } }));
    await ready(page, "./");
    const shuffled = await read(page);
    expect(shuffled.solved).toBe(false);
    expect(shuffled.moveCount).toBe(0);
    expect(shuffled.revision).toBe(1);
    expect(new URL(page.url()).hash).toBe(`#cube=${shuffled.facelets}`);
    await visibleState(page, shuffled);
    await page.reload();
    await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
    expect(await read(page)).toEqual(shuffled);
    await cubePixels(page, 3);
  });
}

test("entering without a position replaces previous play with a fresh shuffle, including the home link", async ({ page, context }) => {
  await ready(page);
  await call(page, "apply_moves", { expectedRevision: 0, moves: "R U" });
  const previous = await read(page);
  await page.goto("./");
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "2");
  const first = await read(page);
  expect(first.moveCount).toBe(0);
  expect(first.solved).toBe(false);
  expect(first.facelets).not.toBe(previous.facelets);
  await page.getByRole("link", { name: "AgentCube home", exact: true }).click();
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "3");
  const second = await read(page);
  expect(second.moveCount).toBe(0);
  expect(second.solved).toBe(false);
  expect(second.facelets).not.toBe(first.facelets);
  await page.reload();
  await expect(page.getByTestId("agent-status")).toHaveText("Ready · 4 tools");
  expect(await read(page)).toEqual(second);
  const peer = await context.newPage();
  await ready(peer, new URL("./", page.url()).href);
  const third = await read(peer);
  expect(third.revision).toBe(4);
  expect(third.moveCount).toBe(0);
  expect(third.facelets).not.toBe(second.facelets);
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", "4");
  await visibleState(page, third);
});
