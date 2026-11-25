#!/usr/bin/env bun
// Plan the first-parent tags and the current desktop release from Git history.
import path from "node:path";

const BASELINE = "v0.2.0";

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

const hasVersion = (versions: Version[], version: Version) =>
  versions.some((seen) => compare(seen, version) === 0);

const nextBump = (bumps: Version[], version: Version) =>
  bumps.findIndex((bump) => compare(bump, version) > 0);

const isAncestor = (cwd: string, parent: string, commit: string) =>
  Bun.spawnSync({
    cmd: ["git", "merge-base", "--is-ancestor", parent, commit],
    cwd,
    stdout: "ignore",
    stderr: "ignore",
  }).exitCode === 0;

interface TagState {
  latest: Version;
  declared: Version;
  tags: Tags;
}

const applyChanges = (
  root: string,
  known: Set<string>,
  changes: readonly (readonly [string, Version])[],
  start: { latest: Version; declared: Version },
  orderedBumps: Version[]
): TagState => {
  let { latest, declared } = start;

  let bumpIndex = nextBump(orderedBumps, declared);

  const tags: Tags = {};

  for (const [commit, version] of changes) {
    if (compare(version, declared) < 0) {
      // Main can have commits predating the merge of the initial version.
      if (
        declared[0] === 0 &&
        declared[1] === 2 &&
        !isAncestor(root, BASELINE, commit)
      ) {
        continue;
      }

      throw new Error(`${commit}: the workspace version decreased`);
    }

    if (compare(version, declared) > 0) {
      const tag = tagName(version);

      if (known.has(tag)) {
        throw new Error(`${tag} already belongs to another commit`);
      }

      if (compare(version, latest) <= 0) {
        throw new Error(
          `${commit}: the workspace version must exceed ${tagName(latest)}`
        );
      }

      latest = version;
      declared = version;
      bumpIndex = nextBump(orderedBumps, declared);
    } else {
      const candidate: Version = [latest[0], latest[1], latest[2] + 1];

      while (known.has(tagName(candidate))) {
        candidate[2] += 1;
      }

      // Leave the next declared version available for the commit that declares it.
      const next = orderedBumps[bumpIndex];

      if (next && compare(candidate, next) >= 0) {
        continue;
      }

      latest = candidate;
    }

    tags[tagName(latest)] = commit;
  }

  return { latest, declared, tags };
};

export const plan = (cwd = process.cwd()): ReleasePlan => {
  const root = path.resolve(cwd);
  const known = new Set(git(root, "tag", "--list").split("\n").filter(Boolean));

  const merged = git(root, "tag", "--merged", "HEAD")
    .split("\n")
    .filter(Boolean);

  if (!merged.includes(BASELINE)) {
    throw new Error(
      `Merge the commit tagged ${BASELINE} into main before tagging releases`
    );
  }

  let latest: Version | null = null;

  for (const tag of merged) {
    if (!tag.startsWith("v")) {
      continue;
    }

    const version = parseVersion(tag.slice(1));

    if (version && (!latest || compare(version, latest) > 0)) {
      latest = version;
    }
  }

  if (!latest) {
    throw new Error("No release tag is merged into HEAD");
  }

  const previous = tagName(latest);
  const declared = declares(root, previous);

  const commits = git(
    root,
    "rev-list",
    "--reverse",
    "--first-parent",
    "HEAD",
    `^${previous}`
  )
    .split("\n")
    .filter(Boolean);

  const changes = commits.map(
    (commit) => [commit, declares(root, commit)] as const
  );

  const bumps: Version[] = [];

  for (const [, version] of changes) {
    if (compare(version, declared) <= 0) {
      continue;
    }

    if (!hasVersion(bumps, version)) {
      bumps.push(version);
    }
  }

  const orderedBumps = bumps.toSorted(compare);

  const { declared: last, tags } = applyChanges(
    root,
    known,
    changes,
    { latest, declared },
    orderedBumps
  );

  const releaseTag = tagName(last);
  let release: ReleasePlan["release"] = null;

  if (releaseTag !== BASELINE) {
    const commit =
      tags[releaseTag] ?? git(root, "rev-parse", `${releaseTag}^{commit}`);

    release = { version: releaseTag.slice(1), commit };
  }

  return { tags, release };
};

if (import.meta.main) {
  try {
    console.log(JSON.stringify(plan()));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
