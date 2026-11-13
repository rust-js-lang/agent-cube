import { defineConfig } from "vite";

export default defineConfig({
  // The same artifact works at / and at GitHub Pages' /agent-cube/.
  base: "./",
  oxc: { jsx: { runtime: "automatic" } },
  build: { outDir: "../dist", emptyOutDir: true },
});
