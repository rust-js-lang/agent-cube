import { defineConfig, devices } from "@playwright/test";
import { availableParallelism } from "node:os";

export default defineConfig({
  testDir: "tests/browser",
  fullyParallel: true,
  workers: availableParallelism(),
  forbidOnly: !!process.env.CI,
  retries: 0,
  timeout: 30_000,
  expect: { timeout: 5_000 },
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL: "http://127.0.0.1:4173/agent-cube/",
    trace: "retain-on-failure",
    actionTimeout: 5_000,
    launchOptions: {
      args: ["--enable-features=WebMCP", "--enable-unsafe-swiftshader"],
    },
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "bun run preview --host 127.0.0.1 --port 4173 --strictPort --base /agent-cube/",
    url: "http://127.0.0.1:4173/agent-cube/",
    reuseExistingServer: false,
    timeout: 20_000,
  },
});
