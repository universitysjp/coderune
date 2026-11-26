#!/usr/bin/env bun
// Plan release tags from manually bumped workspace versions in Git history.
import path from "node:path";

const VERSION =
  /^(?<major>0|[1-9][0-9]*)\.(?<minor>0|[1-9][0-9]*)\.(?<patch>0|[1-9][0-9]*)$/u;

type Version = [number, number, number];

type Tags = Record<string, string>;

export interface ReleasePlan {
  tags: Tags;
  release: { version: string; commit: string } | null;
}

const parseVersion = (value: string): Version | null => {
  const groups = VERSION.exec(value)?.groups;

  if (!groups) {
    return null;
  }

  return [Number(groups.major), Number(groups.minor), Number(groups.patch)];
};

const tagName = (version: Version) => `v${version.join(".")}`;

const compare = (a: Version, b: Version) =>
  a[0] - b[0] || a[1] - b[1] || a[2] - b[2];

const git = (cwd: string, ...args: string[]): string => {
  const result = Bun.spawnSync({
    cmd: ["git", ...args],
    cwd,
    stdout: "pipe",
    stderr: "pipe",
  });

  if (result.exitCode !== 0) {
    const message = result.stderr.toString().trim();
    throw new Error(
      message || `git ${args.join(" ")} exited ${result.exitCode}`
    );
  }

  return result.stdout.toString().trim();
};

const declares = (cwd: string, commit: string): Version => {
  const cargo = git(cwd, "show", `${commit}:Cargo.toml`);

  const workspace =
    cargo.match(/\[workspace\.package\](?<section>[\s\S]*?)(?:\n\[|$)/u)?.groups
      ?.section ?? "";

  const value = workspace.match(/^\s*version\s*=\s*"(?<version>[^"]+)"/mu)
    ?.groups?.version;

  const version = value ? parseVersion(value) : null;

  if (!version) {
    throw new Error(
      `${commit}: set the workspace version to MAJOR.MINOR.PATCH`
    );
  }

  return version;
};

export const plan = (cwd = process.cwd()): ReleasePlan => {
  const root = path.resolve(cwd);
  const known = new Set(git(root, "tag", "--list").split("\n").filter(Boolean));

  const merged = git(root, "tag", "--merged", "HEAD")
    .split("\n")
    .filter(Boolean);

  let latest: Version | null = null;
  let latestTag: string | null = null;

  for (const tag of merged) {
    if (!tag.startsWith("v")) {
      continue;
    }

    const version = parseVersion(tag.slice(1));

    if (version && (!latest || compare(version, latest) > 0)) {
      latest = version;
      latestTag = tag;
    }
  }

  if (!(latest && latestTag)) {
    throw new Error("No release tag is merged into HEAD");
  }

  let declared = declares(root, latestTag);

  const commits = git(
    root,
    "rev-list",
    "--reverse",
    "--first-parent",
    "HEAD",
    `^${latestTag}`
  )
    .split("\n")
    .filter(Boolean);

  const tags: Tags = {};

  for (const commit of commits) {
    const version = declares(root, commit);
    const order = compare(version, declared);

    if (order < 0) {
      throw new Error(`${commit}: the workspace version decreased`);
    }

    // Commits keep the current release version until a person bumps it.
    if (order === 0) {
      continue;
    }

    const tag = tagName(version);

    if (known.has(tag)) {
      throw new Error(`${tag} already belongs to another commit`);
    }

    tags[tag] = commit;
    declared = version;
  }

  const releaseTag = tagName(declared);

  const commit =
    tags[releaseTag] ?? git(root, "rev-parse", `${releaseTag}^{commit}`);

  return { tags, release: { version: releaseTag.slice(1), commit } };
};

if (import.meta.main) {
  try {
    console.log(JSON.stringify(plan()));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
