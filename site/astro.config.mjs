import { defineConfig } from "astro/config";

// On GitHub Pages the site lives under /kinetic-engine/; locally it is served from the root.
const onPages = process.env.GITHUB_PAGES === "true";

export default defineConfig({
  site: "https://hashimhashmi02.github.io",
  base: onPages ? "/kinetic-engine/" : "/",
  server: { port: 4321 },
  vite: {
    // The Kin rig lives in ../design/kin and is shared with the character sheet.
    server: { fs: { allow: [".."] } },
  },
});
