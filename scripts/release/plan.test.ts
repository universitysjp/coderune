import { afterEach, beforeEach, expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { plan } from "./plan";

let root = "";

const git = (...args: string[]): string => {
  const result = Bun.spawnSync({
    cmd: ["git", ...args],
    cwd: root,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(result.stderr.toString().trim());
  }
  return result.stdout.toString().trim();
};

const commit = (version: string): string => {
  writeFileSync(
    join(root, "Cargo.toml"),
    `[workspace.package]\nversion = "${version}"\n`
  );
  git("add", "Cargo.toml");
  git("commit", "-qm", `Version ${version}`, "--allow-empty");
  return git("rev-parse", "HEAD");
};

beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), "release-plan-"));
  git("init", "-q", "-b", "main");
  git("config", "user.email", "test@example.com");
  git("config", "user.name", "Release Test");
  commit("0.2.0");
  git("tag", "v0.2.0");
});

afterEach(() => {
  rmSync(root, { recursive: true, force: true });
});

test("patch push tags each commit without a build", () => {
  const first = commit("0.2.0");
  const second = commit("0.2.0");
  expect(plan(root)).toEqual({
    tags: { "v0.2.1": first, "v0.2.2": second },
    release: null,
  });
});

test("manual bump builds the milestone even if patches follow", () => {
  const milestone = commit("0.3.0");
  const patch = commit("0.3.0");
  const release = { version: "0.3.0", commit: milestone };
  expect(plan(root)).toEqual({
    tags: { "v0.3.0": milestone, "v0.3.1": patch },
    release,
  });

  git("tag", "v0.3.0", milestone);
  git("tag", "v0.3.1", patch);
  expect(plan(root)).toEqual({ tags: {}, release });
});

test("explicit patch bump reserves its tag and retries its build", () => {
  git("tag", "v0.3.0", commit("0.3.0"));
  commit("0.3.0");
  const releaseCommit = commit("0.3.1");
  const later = commit("0.3.1");
  const release = { version: "0.3.1", commit: releaseCommit };

  expect(plan(root)).toEqual({
    tags: { "v0.3.1": releaseCommit, "v0.3.2": later },
    release,
  });

  git("tag", "v0.3.1", releaseCommit);
  git("tag", "v0.3.2", later);
  expect(plan(root)).toEqual({ tags: {}, release });
});

test("explicit patch bump cannot reuse an automatic tag", () => {
  git("tag", "v0.3.0", commit("0.3.0"));
  const automaticTag = commit("0.3.0");
  git("tag", "v0.3.1", automaticTag);
  commit("0.3.1");

  expect(() => plan(root)).toThrow("v0.3.1 already belongs to another commit");
  expect(git("rev-parse", "v0.3.1")).toBe(automaticTag);
});

test("merge preserves existing tags and tags the merge", () => {
  git("switch", "-qc", "setup");
  commit("0.2.0");
  git("tag", "v0.2.1");
  git("switch", "-q", "main");
  git("merge", "-q", "--no-ff", "--no-edit", "setup");
  expect(plan(root)).toEqual({
    tags: { "v0.2.2": git("rev-parse", "HEAD") },
    release: null,
  });
});
