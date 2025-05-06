import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import checker from "vite-plugin-checker";
import path from "path";
import fs from "fs";

export default defineConfig({
  plugins: [
    solidPlugin(),
    checker({
      typescript: true,
    }),
    {
      name: "copy-files",
      closeBundle() {
        fs.copyFileSync(
          path.resolve(
            __dirname,
            "node_modules/@gotcha-widget/lib/dist/lib.umd.js",
          ),
          path.resolve(__dirname, "../dist/lib.js"),
        );
      },
    },
  ],
  build: {
    outDir: path.resolve(__dirname, "../dist"),
    lib: {
      entry: "src/api.tsx",
      name: "Gotcha",
      fileName: () => "api.js",
      formats: ["iife"],
    },
    minify: "terser",
    terserOptions: {
      compress: {
        drop_console: false, // TODO: prod true
        drop_debugger: false, // TODO: prod true
      },
    },
    target: "esnext",
  },
  css: {
    postcss: "./postcss.config.js",
  },
});
