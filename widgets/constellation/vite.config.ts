import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  base: "/constellation/",
  plugins: [tailwindcss(), solidPlugin()],
  server: {
    port: 3000,
  },
  build: {
    outDir: path.resolve(__dirname, "../../dist/constellation/"),
    emptyOutDir: true,
    target: "esnext",
  },
});
