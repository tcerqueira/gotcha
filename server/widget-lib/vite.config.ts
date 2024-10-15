import { defineConfig } from "vite";
import checker from "vite-plugin-checker";
import dts from "vite-plugin-dts";

export default defineConfig({
  plugins: [
    checker({ typescript: true }),
    dts({
      insertTypesEntry: true,
      outDir: "dist/types",
    }),
  ],
  build: {
    lib: {
      entry: "src/lib.ts",
      formats: ["es", "cjs"],
      fileName: (format) => `lib.${format}.js`,
    },
    target: "esnext",
  },
});
