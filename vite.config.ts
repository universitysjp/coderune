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
} from "./scripts/quartz-lint/plugin.mjs";

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

const quartzReferenceFiles = [
  "apps/web/src/components/project-home.tsx",
  "packages/quartz/src/native/**/*.{js,jsx,ts,tsx}",
  "packages/quartz/src/tokens.ts",
];

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
      "scripts/quartz-lint/**",
    ],
    proseWrap: "preserve",
  },
  lint: {
    extends: [core, react, reactDoctor, antiSlop, tanstack],
    ignorePatterns: [
      ...(core.ignorePatterns ?? []),
      ".agents/**",
      "packages/quartz/src/web/**",
      "scripts/quartz-lint/**",
      ...testFiles,
    ],
    jsPlugins: [
      ...(reactDoctor.jsPlugins ?? []),
      ...(antiSlop.jsPlugins ?? []),
      { name: "quartz", specifier: "./scripts/quartz-lint/plugin.mjs" },
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
        // Quartz app rules: desktop screens, the Quartz block kept in the web
        // app, and the web bootstrap. The marketing site's existing cream
        // design is outside the desktop migration.
        files: [
          "apps/desktop/**/*.{js,jsx,ts,tsx}",
          "apps/web/src/components/project-home.tsx",
          "apps/web/src/main.tsx",
          "apps/web/src/routes/__root.tsx",
        ],
        rules: quartzAppRules,
      },
      {
        // Quartz's own sources build the controls, so app-only rules do not
        // apply to them. The DOM components are excluded from linting; the
        // GPUIX adapters stay in scope.
        files: [
          "packages/quartz/src/native/**/*.{js,jsx,ts,tsx}",
          "packages/quartz/src/tokens.ts",
        ],
        rules: quartzSourceRules,
      },
      {
        // GPUIX takes StyleDesc objects and cannot execute CSS classes.
        files: ["apps/desktop/**/*.tsx"],
        jsPlugins: [
          {
            name: "quartz-native",
            specifier: "./scripts/quartz-lint/native.mjs",
          },
        ],
        rules: {
          "quartz/no-inline-style": "off",
          "quartz-native/tokens-only": "error",
          "quartz-native/components-only": "error",
          "quartz-native/controls-named": "error",
        },
      },
      {
        // Quartz reference sources keep the Quartz rules but not the
        // repository's general lint set. Tests and the DOM components are
        // excluded from linting entirely.
        files: quartzReferenceFiles,
        rules: nonQuartzRules,
      },
    ],
  },
  staged: {
    "*.{css,js,jsx,json,jsonc,md,ts,tsx}": "vp check --fix",
  },
});
