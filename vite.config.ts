import ultracite from "ultracite/oxfmt";
import antiSlop from "ultracite/oxlint/anti-slop";
import core from "ultracite/oxlint/core";
import { jsPluginSettings, selectJsPlugins } from "ultracite/oxlint/js-plugins";
import react from "ultracite/oxlint/react";
import tanstack from "ultracite/oxlint/tanstack";
import { defineConfig } from "vite-plus";

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
      "crates/avatars/upstream/**",
      "docs/**",
    ],
    proseWrap: "preserve",
  },
  lint: {
    extends: [core, react, reactDoctor, antiSlop, tanstack],
    ignorePatterns: [
      ...(core.ignorePatterns ?? []),
      ".agents/**",
      "crates/avatars/upstream/**",
      ...testFiles,
    ],
    jsPlugins: [
      ...(reactDoctor.jsPlugins ?? []),
      ...(antiSlop.jsPlugins ?? []),
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
  },
  staged: {
    "*.{css,js,jsx,json,jsonc,md,ts,tsx}": "vp check --fix",
  },
});
