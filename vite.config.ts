import ultracite from "ultracite/oxfmt";
import antiSlop from "ultracite/oxlint/anti-slop";
import core from "ultracite/oxlint/core";
import { jsPluginSettings, selectJsPlugins } from "ultracite/oxlint/js-plugins";
import react from "ultracite/oxlint/react";
import tanstack from "ultracite/oxlint/tanstack";
import { defineConfig } from "vite-plus";

import {
  appRules as quartzAppRules,
  sourceRules as quartzSourceRules,
} from "./lint/quartz/plugin.mjs";

const reactDoctor = selectJsPlugins(["react-doctor"]);

const generalRuleNames = new Set([
  "vite-plus/prefer-vite-plus-imports",
  ...[core, react, reactDoctor, antiSlop, tanstack].flatMap((config) => [
    ...Object.keys(config.rules ?? {}),
    ...(config.overrides ?? []).flatMap((override) =>
      Object.keys(override.rules ?? {})
    ),
  ]),
]);

const nonQuartzRules = Object.fromEntries(
  [...generalRuleNames].map((rule) => [rule, "off"])
);

const testAndQuartzFiles = [
  "**/*.{test,spec}.{js,jsx,ts,tsx}",
  "**/test/**/*.{js,jsx,ts,tsx}",
  "**/tests/**/*.{js,jsx,ts,tsx}",
  "**/__tests__/**/*.{js,jsx,ts,tsx}",
  "apps/web/src/components/quartz/**/*.{js,jsx,ts,tsx}",
  "apps/web/src/lib/quartz/**/*.{js,jsx,ts,tsx}",
  "apps/desktop/src/quartz.tsx",
  "packages/quartz/**/*.{js,jsx,ts,tsx}",
  "packages/tokens/src/quartz.ts",
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
      "lint/**",
    ],
    proseWrap: "preserve",
  },
  lint: {
    extends: [core, react, reactDoctor, antiSlop, tanstack],
    ignorePatterns: [...(core.ignorePatterns ?? []), ".agents/**", "lint/**"],
    jsPlugins: [
      ...(reactDoctor.jsPlugins ?? []),
      ...(antiSlop.jsPlugins ?? []),
      { name: "quartz", specifier: "./lint/quartz/plugin.mjs" },
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
        // Quartz app rules: desktop screens, Quartz blocks, and the web bootstrap.
        // The marketing site's existing cream design is outside the desktop migration.
        files: [
          "apps/desktop/**/*.{js,jsx,ts,tsx}",
          "apps/web/src/components/quartz/**/*.{js,jsx,ts,tsx}",
          "apps/web/src/lib/quartz/**/*.{js,jsx,ts,tsx}",
          "apps/web/src/main.tsx",
          "apps/web/src/routes/__root.tsx",
        ],
        rules: quartzAppRules,
      },
      {
        // Quartz's own components and helpers build the controls, so app-only
        // rules do not apply to them.
        files: [
          "apps/web/src/components/quartz/{ui,ai}/**",
          "apps/web/src/components/quartz/icon.tsx",
          "apps/web/src/lib/quartz/**",
          "apps/desktop/src/quartz.tsx",
        ],
        rules: quartzSourceRules,
      },
      {
        // GPUIX takes StyleDesc objects and cannot execute CSS classes.
        files: ["apps/desktop/**/*.tsx"],
        jsPlugins: [
          { name: "quartz-native", specifier: "./lint/quartz-native.mjs" },
        ],
        rules: {
          "quartz/no-inline-style": "off",
          "quartz-native/tokens-only": "error",
          "quartz-native/components-only": "error",
          "quartz-native/controls-named": "error",
        },
      },
      {
        // Tests and Quartz sources are reference and generated code. Keep the
        // Quartz rules above, but do not enforce the repository's general lint
        // set on them.
        files: testAndQuartzFiles,
        rules: nonQuartzRules,
      },
    ],
  },
  staged: {
    "*.{css,js,jsx,json,jsonc,md,ts,tsx}": "vp check --fix",
  },
});
