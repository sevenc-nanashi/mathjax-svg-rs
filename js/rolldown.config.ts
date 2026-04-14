import { defineConfig } from "rolldown";
import license from "rollup-plugin-license";

export default defineConfig({
  input: "./src/index.ts",
  // output: {
  //   minify: true,
  // },
  resolve: {
    alias: {
      punycode: "punycode/punycode.js",
    },
  },
  plugins: [
    license({
      thirdParty: {
        output: "dist/NOTICE.txt",
      },
    }),
  ],
});
