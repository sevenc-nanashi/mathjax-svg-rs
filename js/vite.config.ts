import { resolve } from "path";
import { defineConfig } from "vite";
import license from "rollup-plugin-license";
import { nodePolyfills } from "vite-plugin-node-polyfills";

export default defineConfig({
  build: {
    lib: {
      entry: resolve(import.meta.dirname, "src/index.ts"),
      name: "index",
      fileName: "index",
      formats: ["es"],
    },
  },
  define: {
    "import.meta": "__importMeta",
  },
  plugins: [
    license({
      thirdParty: {
        output: resolve(import.meta.dirname, "dist/NOTICE.txt"),
      },
    }),
    nodePolyfills({}),
  ],
});
