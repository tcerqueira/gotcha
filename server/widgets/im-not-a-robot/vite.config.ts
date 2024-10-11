import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
// import devtools from 'solid-devtools/vite';
import path from "path";

export default defineConfig({
  base: "/im-not-a-robot/",
  plugins: [
    /*
    Uncomment the following line to enable solid-devtools.
    For more info see https://github.com/thetarnav/solid-devtools/tree/main/packages/extension#readme
    */
    // devtools(),
    solidPlugin(),
  ],
  server: {
    port: 3000,
  },
  build: {
    outDir: path.resolve(__dirname, "../../dist/im-not-a-robot"),
    target: "esnext",
  },
});
