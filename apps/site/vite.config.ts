import { fileURLToPath, URL } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite-plus";

import { navItems } from "./src/lib/site";

const pagesBase = process.env.PAGES_BASE_PATH?.replace(/\/+$/u, "") ?? "";
const base = pagesBase === "" ? "/" : `${pagesBase}/`;

export default defineConfig({
  base,
  plugins: [
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

          this.emitFile({
            fileName: "404.html",
            source: index.source,
            type: "asset",
          });
        },
        order: "post",
      },
      name: "coderune:static-pages",
    },
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("src", import.meta.url)),
    },
  },
});
