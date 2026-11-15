#!/usr/bin/env bun
/* eslint-disable no-await-in-loop -- sequential filesystem work is intentional. */
/**
 * Convert JavaScript files inside installed agent skills to TypeScript after
 * `skills add` or a skill update. Bun runs the converted files directly; the
 * script is idempotent and only touches `.agents/skills`.
 *
 * bun run skills:ts [--dry-run] [--root <repo>]
 */
import fs from "node:fs/promises";
import nodePath from "node:path";

const extensions = {
  ".js": ".ts",
  ".mjs": ".mts",
  ".cjs": ".cts",
  ".jsx": ".tsx",
} as const satisfies Record<string, string>;

const textExtensions = new Set([
  ".md",
  ".json",
  ".ts",
  ".mts",
  ".cts",
  ".tsx",
  ".js",
  ".mjs",
  ".cjs",
  ".jsx",
  ".sh",
  ".yml",
  ".yaml",
]);

const skipDirectories = new Set(["node_modules", ".git", "dist", "build"]);

const args = process.argv.slice(2);
const dryRun = args.includes("--dry-run");
const rootFlag = args.indexOf("--root");

const root =
  rootFlag !== -1 && args[rootFlag + 1]
    ? args[rootFlag + 1]
    : nodePath.join(import.meta.dir, "..");

const skills = nodePath.join(root, ".agents", "skills");

const walk = async (directory: string): Promise<string[]> => {
  const entries = await fs.readdir(directory, { withFileTypes: true });
  const files: string[] = [];

  for (const entry of entries) {
    if (skipDirectories.has(entry.name)) {
      continue;
    }

    const path = nodePath.join(directory, entry.name);

    if (entry.isDirectory()) {
      const nested = await walk(path);
      files.push(...nested);
    } else if (entry.isFile()) {
      files.push(path);
    }
  }

  return files;
};

const escape = (value: string): string =>
  value.replaceAll(/[.*+?^${}()|[\]\\]/gu, "\\$&");

if (
  !(await fs
    .stat(skills)
    .then(() => true)
    .catch(() => false))
) {
  console.error(`No skills directory at ${skills}`);
  process.exit(1);
}

const files = await walk(skills);
const renames = new Map<string, string>();

for (const file of files) {
  const replacement = extensions[nodePath.extname(file).toLowerCase()];

  if (!replacement) {
    continue;
  }

  renames.set(
    file,
    file.slice(0, -nodePath.extname(file).length) + replacement
  );
}

const normalizeCommands = async (): Promise<void> => {
  const allFiles = await walk(skills);

  const textFiles = allFiles.filter((file) =>
    textExtensions.has(nodePath.extname(file))
  );

  for (const file of textFiles) {
    const source = await fs.readFile(file, "utf-8");

    const next = source.replaceAll(
      /^(?<space>\s*)node (?<command>\S+\.mts)/gmu,
      "$<space>bun $<command>"
    );

    if (next !== source) {
      await fs.writeFile(file, next);
    }
  }
};

if (renames.size === 0) {
  if (!dryRun) {
    await normalizeCommands();
  }

  console.log("Skills already use TypeScript.");
  process.exit(0);
}

for (const [from, to] of renames) {
  console.log(
    `${dryRun ? "would rename" : "rename"} ${nodePath.relative(root, from)} -> ${nodePath.relative(root, to)}`
  );

  if (!dryRun) {
    await fs.rename(from, to);
  }
}

if (!dryRun) {
  // Longest names first so `foo.mjs` is replaced before `.js` fragments.
  const replacements = new Map<string, string>();

  for (const [from, to] of renames) {
    const oldPath = nodePath.relative(skills, from);
    const newPath = nodePath.relative(skills, to);
    replacements.set(oldPath, newPath);
    replacements.set(nodePath.basename(from), nodePath.basename(to));
  }

  const ordered = [...replacements].toSorted(([a], [b]) => b.length - a.length);

  const allFiles = await walk(skills);

  const textFiles = allFiles.filter((file) =>
    textExtensions.has(nodePath.extname(file))
  );

  for (const file of textFiles) {
    let source = await fs.readFile(file, "utf-8");
    const before = source;

    for (const [oldName, newName] of ordered) {
      source = source.replaceAll(new RegExp(escape(oldName), "gu"), newName);
    }

    if (source.startsWith("#!/usr/bin/env node")) {
      source = source.replace("#!/usr/bin/env node", "#!/usr/bin/env bun");
    }

    if (source !== before) {
      await fs.writeFile(file, source);
    }
  }

  await normalizeCommands();
}

console.log(
  `${dryRun ? "Would convert" : "Converted"} ${renames.size} skill file(s).`
);
