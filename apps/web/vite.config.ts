import { fileURLToPath, URL } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, lazyPlugins } from "vite-plus";

import { navItems } from "./src/lib/site";

export default defineConfig({
  plugins: lazyPlugins(() => [
    tanstackRouter({ autoCodeSplitting: true }),
    tailwindcss(),
    react({ compiler: true }),
    {
      apply: "build",
      generateBundle: {
        handler(_options, bundle) {
          const index = bundle["index.html"];

          if (index?.type !== "asset") {
            this.error("The web build did not emit index.html");
          }

          // Static hosts serve each known route's entry point on a direct visit.
          for (const { to } of navItems) {
            if (to !== "/") {
              this.emitFile({
                fileName: `${to.slice(1)}/index.html`,
                source: index.source,
                type: "asset",
              });
            }
          }
        },
        order: "post",
      },
      name: "coderune:static-pages",
    },
  ]),
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("src", import.meta.url)),
    },
  },
});
