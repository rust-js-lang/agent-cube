import { expect, type Page } from "@playwright/test";
import { PNG } from "pngjs";

export const solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
export const initialUrl = `./#cube=${solved}`;
const palette = [0xf1efdf, 0xe45e40, 0x95c975, 0xe9d35f, 0xf1a84b, 0x649cbf];
export type State = { facelets: string; faceOrder: string; revision: number; moveCount: number; solved: boolean };

export async function call(page: Page, name: string, input: Record<string, unknown> = {}) {
  if (name === "apply_moves" || name === "shuffle_cube") {
    input = { commandId: crypto.randomUUID(), ...(name === "shuffle_cube" ? { expectedRevision: (await read(page)).revision } : {}), ...input };
  }
  return page.webmcp.callTool(name, input);
}

export async function read(page: Page): Promise<State> {
  const result = await call(page, "get_cube_state");
  expect(result.isError).toBe(false);
  return JSON.parse(result.content[0].text);
}

export async function visibleState(page: Page, state: State) {
  await expect(page.locator(".cube-net")).toHaveAttribute("data-revision", String(state.revision));
  expect(await page.locator(".cube-net [data-color]").evaluateAll(nodes => nodes.map(node => node.getAttribute("data-color")).join(""))).toBe(state.facelets);
}

export async function cubePixels(page: Page, minColors = 1) {
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


export async function controlFrames(page: Page) {
  await page.addInitScript(() => {
    let next = 0;
    const frames = new Map<number, FrameRequestCallback>();
    window.requestAnimationFrame = callback => { const id = ++next; frames.set(id, callback); return id; };
    window.cancelAnimationFrame = id => { frames.delete(id); };
    Object.assign(window, {
      __stepFrame(time: number) { const pending = [...frames.values()]; frames.clear(); for (const callback of pending) callback(time); },
      __pendingFrames() { return frames.size; },
    });
  });
}

export async function frame(page: Page, time: number) {
  await page.evaluate(time => Reflect.get(window, "__stepFrame")(time), time);
}


export async function savedText(page: Page) {
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

