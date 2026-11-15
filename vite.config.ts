import ultracite from "ultracite/oxfmt";
import antiSlop from "ultracite/oxlint/anti-slop";
import core from "ultracite/oxlint/core";
import { jsPluginSettings, selectJsPlugins } from "ultracite/oxlint/js-plugins";
import react from "ultracite/oxlint/react";
import tanstack from "ultracite/oxlint/tanstack";
import { defineConfig } from "vite-plus";

import { appRules as uiAppRules } from "./scripts/ui-lint/plugin.mts";

const reactDoctor = selectJsPlugins(["react-doctor"]);

const testFiles = [
  "**/*.{test,spec}.{js,jsx,ts,tsx}",
  "**/test/**/*.{js,jsx,ts,tsx}",
  "**/tests/**/*.{js,jsx,ts,tsx}",
  "**/__tests__/**/*.{js,jsx,ts,tsx}",
];

export default defineConfig({
  fmt: {
    ...ultracite,
    ignorePatterns: [
      ...(ultracite.ignorePatterns ?? []),
      ".agents/**",
      "AGENTS.md",
      "GLOSSARY.md",
      "README.md",
      "docs/**",
      "scripts/ui-lint/**",
    ],
    proseWrap: "preserve",
  },
  lint: {
    extends: [core, react, reactDoctor, antiSlop, tanstack],
    ignorePatterns: [
      ...(core.ignorePatterns ?? []),
      ".agents/**",
      "packages/ui/src/web/**",
      "scripts/ui-lint/**",
      ...testFiles,
    ],
    jsPlugins: [
      ...(reactDoctor.jsPlugins ?? []),
      ...(antiSlop.jsPlugins ?? []),
      { name: "ui", specifier: "./scripts/ui-lint/plugin.mts" },
      { name: "vite-plus", specifier: "vite-plus/oxlint-plugin" },
    ],
    plugins: [
      "eslint",
      "typescript",
      "unicorn",
      "oxc",
      "react",
      "react-perf",
      "jsx-a11y",
    ],
    rules: {
      "vite-plus/prefer-vite-plus-imports": "error",
    },
    settings: {
      ...jsPluginSettings,
      react: {
        version: "19",
      },
      "react-doctor": {
        ...jsPluginSettings["react-doctor"],
        capabilities: ["react-compiler"],
      },
    },
    overrides: [
      {
        // UI app rules: desktop screens, the UI block kept in the web
        // app, and the web bootstrap. The marketing site's existing cream
        // design is outside the desktop migration.
        files: ["apps/web/src/main.tsx", "apps/web/src/routes/__root.tsx"],
        rules: uiAppRules,
      },
    ],
  },
  staged: {
    "*.{css,js,jsx,json,jsonc,md,ts,tsx}": "vp check --fix",
  },
});
