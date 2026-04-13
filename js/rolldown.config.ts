import { defineConfig } from "rolldown";
export default defineConfig({
  input: "./src/index.ts",
  output: {
    minify: true,
  },
  resolve: {
    alias: {
      punycode: "punycode/punycode.js",
    },
  },
});
