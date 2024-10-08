import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import checker from "vite-plugin-checker";

export default defineConfig({
  plugins: [
    solidPlugin(),
    checker({
      typescript: true,
    }),
  ],
  build: {
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
    target: "es2015",
  },
});
