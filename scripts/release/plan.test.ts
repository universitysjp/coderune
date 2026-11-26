import { afterEach, beforeEach, expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { plan } from "./plan";

let root = "";
let baseline = "";

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
  baseline = commit("0.2.0");
  git("tag", "v0.2.0");
});

afterEach(() => {
  rmSync(root, { recursive: true, force: true });
});

test("commits without a manual bump are not tagged", () => {
  commit("0.2.0");
  commit("0.2.0");
  expect(plan(root)).toEqual({
    tags: {},
    release: { version: "0.2.0", commit: baseline },
  });
});

test("a manual bump tags only its own commit and starts the release", () => {
  commit("0.2.0");
  const releaseCommit = commit("0.3.0");
  commit("0.3.0");
  expect(plan(root)).toEqual({
    tags: { "v0.3.0": releaseCommit },
    release: { version: "0.3.0", commit: releaseCommit },
  });
});

test("the newest manual bump is the release when several land together", () => {
  const first = commit("0.3.0");
  const second = commit("0.3.1");
  expect(plan(root)).toEqual({
    tags: { "v0.3.0": first, "v0.3.1": second },
    release: { version: "0.3.1", commit: second },
  });
});

test("an existing tag retries its unpublished release", () => {
  const releaseCommit = commit("0.3.0");
  git("tag", "v0.3.0", releaseCommit);
  commit("0.3.0");
  expect(plan(root)).toEqual({
    tags: {},
    release: { version: "0.3.0", commit: releaseCommit },
  });
});

test("a manual bump cannot reuse a tag from another commit", () => {
  const tagged = commit("0.2.0");
  git("tag", "v0.3.0", tagged);
  commit("0.3.0");
  expect(() => plan(root)).toThrow("v0.3.0 already belongs to another commit");
});

test("a workspace version decrease is rejected", () => {
  commit("0.3.0");
  commit("0.2.9");
  expect(() => plan(root)).toThrow("the workspace version decreased");
});
