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

