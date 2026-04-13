import fs from "node:fs";
import { defineConfig } from "rolldown";
export default defineConfig({
  input: "./src/index.ts",
  resolve: {
    alias: {
      punycode: "punycode/punycode.js",
    },
  },
});
