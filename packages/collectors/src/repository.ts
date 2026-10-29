import {
  accessSync,
  constants,
  lstatSync,
  readFileSync,
  readdirSync,
  realpathSync,
  statSync,
} from "node:fs";
import path from "node:path";

import type {
  CaptureFileCategory,
  CaptureSkipReason,
  CollectionScope,
  CollectedFile,
  GitState,
  RepositoryCollection,
  RepositoryCollectionRequest,
  RepositoryCollector,
} from "@coderune/core";

const DEPENDENCY_DIRECTORIES = new Set([
  ".git",
  ".hg",
  ".svn",
  ".venv",
  "venv",
  "env",
  "node_modules",
  "__pycache__",
  ".pytest_cache",
  ".mypy_cache",
  ".ruff_cache",
  ".turbo",
  ".next",
  ".nuxt",
  ".cache",
  "dist",
  "build",
  "out",
  "coverage",
  "vendor",
  "target",
]);

const GENERATED_DIRECTORIES = new Set([
  "dist",
  "build",
  "out",
  "coverage",
  ".next",
  ".turbo",
]);

const BINARY_EXTENSIONS = new Set([
  ".png",
  ".jpg",
  ".jpeg",
  ".gif",
  ".webp",
  ".ico",
  ".bmp",
  ".tiff",
  ".mp3",
  ".mp4",
  ".mov",
  ".avi",
  ".wav",
  ".pdf",
  ".docx",
  ".doc",
  ".xlsx",
  ".zip",
  ".gz",
  ".tar",
  ".rar",
  ".7z",
  ".exe",
  ".dll",
  ".so",
  ".dylib",
  ".bin",
  ".woff",
  ".woff2",
  ".ttf",
  ".otf",
  ".sqlite",
  ".db",
]);

const MANIFEST_NAMES = new Set([
  "package.json",
  "package-lock.json",
  "pnpm-lock.yaml",
  "yarn.lock",
  "bun.lock",
  "bun.lockb",
  "pyproject.toml",
  "poetry.lock",
  "pipfile",
  "pipfile.lock",
  "uv.lock",
  "setup.py",
  "setup.cfg",
  "requirements.txt",
]);

const CONFIGURATION_NAMES = new Set([
  "tsconfig.json",
  "tsconfig.base.json",
  "jsconfig.json",
  "turbo.json",
  "nx.json",
  "vite.config.ts",
  "vite.config.js",
  "vitest.config.ts",
  "vitest.config.js",
  "jest.config.ts",
  "jest.config.js",
  "eslint.config.js",
  "eslint.config.mjs",
  ".eslintrc",
  ".eslintrc.json",
  ".prettierrc",
  ".prettierrc.json",
  ".editorconfig",
  ".python-version",
  ".nvmrc",
  ".tool-versions",
  "pytest.ini",
  "tox.ini",
  "mypy.ini",
  "ruff.toml",
  "pylintrc",
]);

const SOURCE_EXTENSIONS = new Set([
  ".ts",
  ".tsx",
  ".mts",
  ".cts",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".py",
]);

const DOCUMENTATION_EXTENSIONS = new Set([".md", ".markdown", ".txt"]);

const isSensitiveName = (name: string): boolean => {
  const lower = name.toLowerCase();

  return (
    lower === ".env" ||
    lower.startsWith(".env.") ||
    lower === ".npmrc" ||
    lower === ".pypirc" ||
    lower === ".netrc" ||
    lower === "id_rsa" ||
    lower === "id_ed25519" ||
    lower === "id_ecdsa" ||
    lower.endsWith(".pem") ||
    lower.endsWith(".key") ||
    lower.endsWith(".p12") ||
    lower.endsWith(".pfx")
  );
};

const isDependencyDirectory = (name: string): boolean =>
  DEPENDENCY_DIRECTORIES.has(name.toLowerCase());

const isGeneratedDirectory = (name: string): boolean =>
  GENERATED_DIRECTORIES.has(name.toLowerCase());

const isBinaryExtension = (extension: string): boolean =>
  BINARY_EXTENSIONS.has(extension);

const isManifest = (name: string): boolean => {
  const lower = name.toLowerCase();

  return MANIFEST_NAMES.has(lower) || /^requirements.*\.txt$/iu.test(name);
};

const isConfiguration = (name: string): boolean => {
  const lower = name.toLowerCase();

  return (
    CONFIGURATION_NAMES.has(lower) ||
    /^(?<tool>vite|vitest|jest|eslint|prettier)\.config\.(?<extension>ts|js|mjs|cjs)$/iu.test(
      name
    ) ||
    /^tsconfig(?<suffix>\..+)?\.json$/iu.test(name)
  );
};

const isTestPath = (relativePath: string): boolean =>
  /(?<prefix>^|\/)(?<directory>__tests__|tests?)\//iu.test(relativePath) ||
  /\.(?<kind>test|spec)\.[^.]+$/iu.test(relativePath) ||
  /(?<prefix>^|\/)test_[^/]+\.py$/iu.test(relativePath) ||
  /_test\.py$/iu.test(relativePath);

type IgnoreMatcher = (relativePath: string, isDirectory: boolean) => boolean;

const normalizeRelative = function normalizeRelative(
  relativePath: string
): string {
  return relativePath
    .replaceAll("\\", "/")
    .replace(/^\.\//u, "")
    .replace(/^\/+/u, "");
};

const matchesPattern = function matchesPattern(
  pattern: string,
  relativePath: string,
  isDirectory: boolean
): boolean {
  if (!pattern) {
    return false;
  }

  const directoryPattern = pattern.endsWith("/");
  const cleanPattern = pattern.replace(/\/+$/u, "");
  const candidate = normalizeRelative(relativePath);

  if (directoryPattern) {
    return (
      candidate === cleanPattern || candidate.startsWith(`${cleanPattern}/`)
    );
  }

  if (!cleanPattern.includes("/") && !/[*?[\]{}]/u.test(cleanPattern)) {
    return candidate.split("/").includes(cleanPattern);
  }

  if (
    cleanPattern.includes("/") &&
    (candidate === cleanPattern || candidate.startsWith(`${cleanPattern}/`))
  ) {
    return true;
  }

  if (/[*?[\]{}]/u.test(cleanPattern)) {
    const glob = new Bun.Glob(cleanPattern);

    // Bun.Glob#match is a glob API, not RegExp#match.
    // oxlint-disable-next-line unicorn/prefer-regexp-test
    if (glob.match(candidate)) {
      return true;
    }

    const name = path.basename(candidate);

    // oxlint-disable-next-line unicorn/prefer-regexp-test
    if (!cleanPattern.includes("/") && glob.match(name)) {
      return true;
    }
  }

  return isDirectory && candidate.startsWith(`${cleanPattern}/`);
};

const isExcluded = function isExcluded(
  relativePath: string,
  isDirectory: boolean,
  scope: CollectionScope,
  ignoreMatcher: IgnoreMatcher,
  extraExclusions: string[]
): boolean {
  const normalized = normalizeRelative(relativePath);

  const excludedByScope = [...scope.excludePaths, ...extraExclusions].some(
    (pattern) =>
      matchesPattern(normalizeRelative(pattern), normalized, isDirectory)
  );

  if (excludedByScope) {
    return true;
  }

  if (scope.respectGitignore && ignoreMatcher(normalized, isDirectory)) {
    return true;
  }

  return false;
};

const ignoreReason = function ignoreReason(
  relativePath: string,
  scope: CollectionScope,
  ignoreMatcher: IgnoreMatcher,
  extraExclusions: string[]
): CaptureSkipReason {
  const normalized = normalizeRelative(relativePath);

  const excludedByScope = [...scope.excludePaths, ...extraExclusions].some(
    (pattern) => matchesPattern(normalizeRelative(pattern), normalized, false)
  );

  if (excludedByScope) {
    return "excluded";
  }

  if (scope.respectGitignore && ignoreMatcher(normalized, false)) {
    return "ignored";
  }

  return "excluded";
};

const loadIgnoreMatcher = function loadIgnoreMatcher(
  root: string
): IgnoreMatcher {
  const rules: { pattern: string; negated: boolean }[] = [];

  try {
    const text = readFileSync(path.resolve(root, ".gitignore"), "utf-8");

    for (const line of text.split(/\r?\n/u)) {
      const trimmed = line.trim();

      if (!trimmed || trimmed.startsWith("#")) {
        continue;
      }

      rules.push({
        pattern: normalizeRelative(trimmed.replace(/^!/u, "")),
        negated: trimmed.startsWith("!"),
      });
    }
  } catch {
    // A repository does not need a .gitignore.
  }

  return (relativePath, isDirectory) => {
    let ignored = false;

    for (const rule of rules) {
      if (matchesPattern(rule.pattern, relativePath, isDirectory)) {
        ignored = !rule.negated;
      }
    }

    return ignored;
  };
};

const toRelativePath = function toRelativePath(
  root: string,
  absolutePath: string
): string {
  const relativePath = path.relative(root, absolutePath);

  return relativePath.split(path.sep).join("/") || ".";
};

const looksBinary = function looksBinary(bytes: Uint8Array): boolean {
  const sample = bytes.subarray(0, Math.min(bytes.byteLength, 8000));

  return sample.includes(0);
};

const categorize = function categorize(
  relativePath: string,
  name: string
): CaptureFileCategory {
  if (name.toLowerCase() === "agents.md") {
    return "agent-instructions";
  }

  if (/^readme(?<suffix>\..+)?$/iu.test(name)) {
    return "readme";
  }

  if (isManifest(name)) {
    return "manifest";
  }

  if (isConfiguration(name)) {
    return "configuration";
  }

  if (isTestPath(relativePath)) {
    return "test";
  }

  if (SOURCE_EXTENSIONS.has(path.extname(name).toLowerCase())) {
    return "source";
  }

  if (DOCUMENTATION_EXTENSIONS.has(path.extname(name).toLowerCase())) {
    return "documentation";
  }

  return "other";
};

const classifySkip = function classifySkip(
  relativePath: string,
  name: string,
  absolutePath: string,
  limits: RepositoryCollectionRequest["limits"],
  scope: CollectionScope,
  ignoreMatcher: IgnoreMatcher,
  extraExclusions: string[]
): CaptureSkipReason | null {
  if (/\.(?<extension>pem|key|p12|pfx)$/iu.test(name)) {
    return "private-key";
  }

  if (isSensitiveName(name)) {
    return "environment-file";
  }

  if (isExcluded(relativePath, false, scope, ignoreMatcher, extraExclusions)) {
    return ignoreReason(relativePath, scope, ignoreMatcher, extraExclusions);
  }

  const extension = path.extname(name).toLowerCase();

  if (isBinaryExtension(extension)) {
    return "binary";
  }

  const allowed =
    isManifest(name) ||
    isConfiguration(name) ||
    name.toLowerCase() === "agents.md" ||
    /^readme(?<suffix>\..+)?$/iu.test(name) ||
    DOCUMENTATION_EXTENSIONS.has(extension) ||
    SOURCE_EXTENSIONS.has(extension);

  if (!allowed) {
    return "unsupported-type";
  }

  try {
    if (!lstatSync(absolutePath).isFile()) {
      return "not-readable";
    }
  } catch {
    return "not-readable";
  }

  if (limits.maxFileBytes <= 0) {
    return "file-too-large";
  }

  return null;
};

const addSingleFile = function addSingleFile(
  root: string,
  absolutePath: string,
  request: RepositoryCollectionRequest,
  ignoreMatcher: IgnoreMatcher,
  extraExclusions: string[],
  files: CollectedFile[],
  skipped: Map<string, CaptureSkipReason>,
  totalBytes: number
): number {
  const name = path.basename(absolutePath);
  const relativePath = toRelativePath(root, absolutePath);

  const skip = classifySkip(
    relativePath,
    name,
    absolutePath,
    request.limits,
    request.scope,
    ignoreMatcher,
    extraExclusions
  );

  if (skip) {
    skipped.set(relativePath, skip);

    return 0;
  }

  const { size } = statSync(absolutePath);

  if (size > request.limits.maxFileBytes) {
    skipped.set(relativePath, "file-too-large");

    return 0;
  }

  if (totalBytes + size > request.limits.maxCaptureBytes) {
    skipped.set(relativePath, "capture-limit");

    return 0;
  }

  const bytes = new Uint8Array(readFileSync(absolutePath));

  if (looksBinary(bytes)) {
    skipped.set(relativePath, "binary");

    return 0;
  }

  files.push({
    path: relativePath,
    size,
    hash: new Bun.CryptoHasher("sha256").update(bytes).digest("hex"),
    category: categorize(relativePath, name),
    content: new TextDecoder("utf-8", { fatal: false }).decode(bytes),
  });

  return size;
};

const resolveScopeRoots = function resolveScopeRoots(
  root: string,
  scope: CollectionScope
): (string | null)[] {
  if (scope.includePaths.length === 0) {
    return [root];
  }

  return scope.includePaths.map((includePath) => {
    const absolute = path.resolve(root, includePath);
    const rel = path.relative(root, absolute);

    if (rel.startsWith("..") || rel === "..") {
      return null;
    }

    try {
      const real = realpathSync(absolute);
      const realRel = path.relative(root, real);

      return realRel.startsWith("..") ? null : real;
    } catch {
      return absolute;
    }
  });
};

const readGitState = function readGitState(root: string): GitState | null {
  try {
    const inside = Bun.spawnSync(
      ["git", "-C", root, "rev-parse", "--is-inside-work-tree"],
      {
        stdout: "pipe",
        stderr: "pipe",
      }
    );

    if (inside.exitCode !== 0 || inside.stdout.toString().trim() !== "true") {
      return null;
    }

    const headResult = Bun.spawnSync(["git", "-C", root, "rev-parse", "HEAD"], {
      stdout: "pipe",
      stderr: "pipe",
    });

    const branchResult = Bun.spawnSync(
      ["git", "-C", root, "rev-parse", "--abbrev-ref", "HEAD"],
      { stdout: "pipe", stderr: "pipe" }
    );

    const statusResult = Bun.spawnSync(
      ["git", "-C", root, "status", "--porcelain=v1"],
      { stdout: "pipe", stderr: "pipe" }
    );

    const status = statusResult.stdout
      .toString()
      .split(/\r?\n/u)
      .filter(Boolean);

    return {
      head:
        headResult.exitCode === 0
          ? headResult.stdout.toString().trim() || null
          : null,
      branch:
        branchResult.exitCode === 0
          ? branchResult.stdout.toString().trim() || null
          : null,
      dirty: status.length > 0,
      status,
      capturedAt: new Date().toISOString(),
    };
  } catch {
    return null;
  }
};

const protectedPathReason = function protectedPathReason(
  relativePath: string,
  isDirectory: boolean
): CaptureSkipReason | null {
  const segments = normalizeRelative(relativePath).split("/").filter(Boolean);

  for (const segment of isDirectory ? segments : segments.slice(0, -1)) {
    if (isSensitiveName(segment)) {
      return "environment-file";
    }

    if (isGeneratedDirectory(segment)) {
      return "generated-output";
    }

    if (isDependencyDirectory(segment)) {
      return "dependency-directory";
    }
  }

  return null;
};

export class LocalRepositoryCollector implements RepositoryCollector {
  // The collector contract is instance-based but these methods hold no state.
  // oxlint-disable-next-line eslint/class-methods-use-this
  validateRoot(rootPath: string): void {
    const status = statSync(rootPath);

    if (!status.isDirectory()) {
      throw new Error("The project root must be a readable folder.");
    }

    accessSync(rootPath, constants.R_OK);
  }

  // oxlint-disable-next-line eslint/class-methods-use-this
  async collect(
    request: RepositoryCollectionRequest
  ): Promise<RepositoryCollection> {
    const root = realpathSync(request.rootPath);
    const git = await readGitState(root);
    const ignoreMatcher = loadIgnoreMatcher(root);
    const extraExclusions = request.extraExcludePaths ?? [];
    const files: CollectedFile[] = [];
    const skipped = new Map<string, CaptureSkipReason>();
    let totalBytes = 0;

    const scopeRoots = resolveScopeRoots(root, request.scope);
    const includeAll = request.scope.includePaths.length === 0;

    const walk = (absoluteDirectory: string): void => {
      let entries;

      try {
        entries = readdirSync(absoluteDirectory, { withFileTypes: true });
      } catch {
        skipped.set(toRelativePath(root, absoluteDirectory), "not-readable");

        return;
      }

      entries.sort((left, right) => left.name.localeCompare(right.name));

      for (const entry of entries) {
        const absolutePath = path.resolve(absoluteDirectory, entry.name);
        const relativePath = toRelativePath(root, absolutePath);

        if (relativePath.startsWith("..") || relativePath === "") {
          skipped.set(relativePath, "escapes-repository");
          continue;
        }

        if (entry.isSymbolicLink()) {
          skipped.set(relativePath, "symbolic-link");
          continue;
        }

        if (entry.isDirectory()) {
          if (isSensitiveName(entry.name)) {
            skipped.set(relativePath, "environment-file");
            continue;
          }

          if (isGeneratedDirectory(entry.name)) {
            skipped.set(relativePath, "generated-output");
            continue;
          }

          if (isDependencyDirectory(entry.name)) {
            skipped.set(relativePath, "dependency-directory");
            continue;
          }

          if (
            isExcluded(
              relativePath,
              true,
              request.scope,
              ignoreMatcher,
              extraExclusions
            )
          ) {
            skipped.set(
              relativePath,
              ignoreReason(
                relativePath,
                request.scope,
                ignoreMatcher,
                extraExclusions
              )
            );
            continue;
          }

          walk(absolutePath);
          continue;
        }

        if (!entry.isFile()) {
          continue;
        }

        const fileSkip = classifySkip(
          relativePath,
          entry.name,
          absolutePath,
          request.limits,
          request.scope,
          ignoreMatcher,
          extraExclusions
        );

        if (fileSkip) {
          skipped.set(relativePath, fileSkip);
          continue;
        }

        let size: number;

        try {
          ({ size } = statSync(absolutePath));
        } catch {
          skipped.set(relativePath, "not-readable");
          continue;
        }

        if (size > request.limits.maxFileBytes) {
          skipped.set(relativePath, "file-too-large");
          continue;
        }

        if (totalBytes + size > request.limits.maxCaptureBytes) {
          skipped.set(relativePath, "capture-limit");
          continue;
        }

        let bytes: Uint8Array;

        try {
          bytes = new Uint8Array(readFileSync(absolutePath));
        } catch {
          skipped.set(relativePath, "not-readable");
          continue;
        }

        if (looksBinary(bytes)) {
          skipped.set(relativePath, "binary");
          continue;
        }

        const hash = new Bun.CryptoHasher("sha256").update(bytes).digest("hex");

        const content = new TextDecoder("utf-8", { fatal: false }).decode(
          bytes
        );

        files.push({
          path: relativePath,
          size,
          hash,
          category: categorize(relativePath, entry.name),
          content,
        });
        totalBytes += size;
      }
    };

    if (includeAll) {
      walk(root);
    } else {
      for (const scopeRoot of scopeRoots) {
        if (scopeRoot === null) {
          skipped.set(
            request.scope.includePaths[0] ?? "?",
            "escapes-repository"
          );
          continue;
        }

        try {
          const status = statSync(scopeRoot);
          const relativeScope = toRelativePath(root, scopeRoot);

          const protectedReason = protectedPathReason(
            relativeScope,
            status.isDirectory()
          );

          if (protectedReason) {
            skipped.set(relativeScope, protectedReason);
          } else if (status.isDirectory()) {
            walk(scopeRoot);
          } else if (status.isFile()) {
            totalBytes += addSingleFile(
              root,
              scopeRoot,
              request,
              ignoreMatcher,
              extraExclusions,
              files,
              skipped,
              totalBytes
            );
          }
        } catch {
          skipped.set(toRelativePath(root, scopeRoot), "not-readable");
        }
      }
    }

    files.sort((left, right) => left.path.localeCompare(right.path));

    const skippedFiles = [...skipped.entries()]
      .map(([relativePath, reason]) => ({ path: relativePath, reason }))
      .toSorted((left, right) => left.path.localeCompare(right.path));

    const fingerprint = new Bun.CryptoHasher("sha256")
      .update(
        JSON.stringify({
          files: files.map((file) => [file.path, file.hash]),
          git: git ? [git.head, git.branch, git.status] : null,
          scope: request.scope,
          limits: request.limits,
        })
      )
      .digest("hex");

    return {
      files,
      skipped: skippedFiles,
      git,
      totalBytes,
      fingerprint,
    };
  }
}
