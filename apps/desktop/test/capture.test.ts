import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { ApplicationError, summarizePreview } from "@coderune/core";

import { createTestHarness } from "./helpers";
import type { TestHarness } from "./helpers";

let harness: TestHarness | undefined;
let workspace: string;
let repository: string;

const write = (path: string, content: string) => {
  const absolute = join(repository, path);
  mkdirSync(join(absolute, ".."), { recursive: true });
  writeFileSync(absolute, content);
};

const git = (...args: string[]) =>
  Bun.spawnSync(["git", "-C", repository, ...args], {
    stdout: "pipe",
    stderr: "pipe",
  });

beforeEach(async () => {
  workspace = mkdtempSync(join(tmpdir(), "coderune-repo-"));
  repository = join(workspace, "repo");
  mkdirSync(repository);

  write(
    "package.json",
    JSON.stringify({ name: "fixture", scripts: { test: "true" } })
  );
  write("tsconfig.json", "{}");
  write("README.md", "# Fixture");
  write("AGENTS.md", "Use Bun.");
  write("docs/guide.md", "# Guide");
  write("src/index.ts", "export const value = 1;\n");
  write("src/app.py", "value = 1\n");
  write("tests/test_app.py", "def test_value():\n    assert True\n");
  write(".env", "SECRET=one\n");
  write(".gitignore", "coverage/\n");
  write("node_modules/dep/index.js", "module.exports = 1;\n");
  write("dist/generated.js", "generated\n");
  write("secrets/private.pem", "private\n");
  write("coverage/report.txt", "ignored\n");
  write("assets/logo.png", "\u0000binary");

  git("init", "-q");
  git("config", "user.email", "test@example.com");
  git("config", "user.name", "Test");
  git("add", ".");
  git("commit", "-qm", "fixture");

  harness = await createTestHarness();
  await harness.app.confirmOnboarding({
    displayName: "Sam",
    experienceStage: "early-career",
    newToCoding: false,
    skillsStatement: "TypeScript",
    learningGoals: ["Learn capture"],
  });
});

afterEach(() => {
  harness?.close();
  harness?.cleanup();
  harness = undefined;
  rmSync(workspace, { force: true, recursive: true });
});

async function createProject(overrides: Record<string, unknown> = {}) {
  return harness!.app.createProject({
    name: "Fixture",
    rootPath: repository,
    goals: ["Capture"],
    ...overrides,
  });
}

describe("repository capture", () => {
  test("preview reports permitted files, exclusions, skip reasons, limits, and Git state", async () => {
    const project = await createProject();
    const preview = await harness!.app.previewCapture(project.id);
    const paths = preview.files.map((file) => file.path);

    expect(paths).toContain("package.json");
    expect(paths).toContain("tsconfig.json");
    expect(paths).toContain("README.md");
    expect(paths).toContain("AGENTS.md");
    expect(paths).toContain("docs/guide.md");
    expect(paths).toContain("src/index.ts");
    expect(paths).toContain("src/app.py");
    expect(paths).toContain("tests/test_app.py");

    const skipped = new Map(
      preview.skipped.map((item) => [item.path, item.reason])
    );

    expect(skipped.get(".env")).toBe("environment-file");
    expect(skipped.get("node_modules")).toBe("dependency-directory");
    expect(skipped.get("dist")).toBe("generated-output");
    expect(skipped.get("secrets/private.pem")).toBe("private-key");
    expect(skipped.get("assets/logo.png")).toBe("binary");

    const categories = new Map(
      preview.files.map((file) => [file.path, file.category])
    );

    expect(categories.get("package.json")).toBe("manifest");
    expect(categories.get("tsconfig.json")).toBe("configuration");
    expect(categories.get("AGENTS.md")).toBe("agent-instructions");
    expect(categories.get("README.md")).toBe("readme");
    expect(categories.get("tests/test_app.py")).toBe("test");

    expect(preview.git?.dirty).toBe(false);
    expect(preview.git?.head).toBeTruthy();
    expect(preview.totalBytes).toBeGreaterThan(0);

    const summary = summarizePreview(preview);
    expect(summary.fileCount).toBe(preview.files.length);
    expect(summary.gitAvailable).toBe(true);
  });

  test("scope includes only approved paths and respects preview-time exclusions", async () => {
    const project = await createProject({
      scope: { includePaths: ["src", "docs"], excludePaths: ["src/app.py"] },
    });

    const preview = await harness!.app.previewCapture(project.id, {
      excludePaths: ["docs/guide.md"],
    });

    const paths = preview.files.map((file) => file.path);

    expect(paths).toContain("src/index.ts");
    expect(paths).not.toContain("src/app.py");
    expect(paths).not.toContain("docs/guide.md");
    expect(paths).not.toContain("README.md");

    const skipped = new Map(
      preview.skipped.map((item) => [item.path, item.reason])
    );

    expect(skipped.get("src/app.py")).toBe("excluded");
    expect(skipped.get("docs/guide.md")).toBe("excluded");
  });

  test("per-file and per-capture limits expose reasons instead of silently dropping content", async () => {
    write("big/one.txt", "a".repeat(2048));
    write("big/two.txt", "b".repeat(800));
    write("big/three.txt", "c".repeat(800));

    const project = await createProject({
      limits: { maxFileBytes: 1024, maxCaptureBytes: 1000 },
    });

    const preview = await harness!.app.previewCapture(project.id);

    const skipped = new Map(
      preview.skipped.map((item) => [item.path, item.reason])
    );

    expect(skipped.get("big/one.txt")).toBe("file-too-large");
    expect(
      [...skipped.values()].filter((reason) => reason === "capture-limit")
    ).toHaveLength(1);
    expect(
      preview.files.filter((file) => file.path.startsWith("big/"))
    ).toHaveLength(1);
    expect(preview.totalBytes).toBeLessThanOrEqual(1000);
  });

  test("explicit includes cannot bypass protected directories or key exclusions", async () => {
    const project = await createProject({
      scope: {
        includePaths: [
          "node_modules",
          "dist",
          "secrets/private.pem",
          "node_modules/dep/index.js",
        ],
      },
    });

    const preview = await harness!.app.previewCapture(project.id);

    const skipped = new Map(
      preview.skipped.map((item) => [item.path, item.reason])
    );

    expect(preview.files).toHaveLength(0);
    expect(skipped.get("node_modules")).toBe("dependency-directory");
    expect(skipped.get("dist")).toBe("generated-output");
    expect(skipped.get("secrets/private.pem")).toBe("private-key");
    expect(skipped.get("node_modules/dep/index.js")).toBe(
      "dependency-directory"
    );
  });

  test("a readable non-Git folder still captures", async () => {
    const plain = join(workspace, "plain");
    mkdirSync(plain);
    writeFileSync(join(plain, "README.md"), "# Plain folder\n");

    const project = await createProject({ rootPath: plain });
    const preview = await harness!.app.previewCapture(project.id);
    expect(preview.git).toBeNull();
    expect(preview.files.map((file) => file.path)).toContain("README.md");
  });

  test("save requires confirmation, binds to the reviewed capture, and rejects stale source content", async () => {
    const project = await createProject();
    const preview = await harness!.app.previewCapture(project.id);

    const unconfirmed = await harness!.app
      .saveCapture(preview.id, { confirmed: false })
      .catch((error: unknown) => error);

    expect(unconfirmed).toBeInstanceOf(ApplicationError);
    expect((unconfirmed as ApplicationError).code).toBe(
      "confirmation-required"
    );

    write("src/index.ts", "export const value = 2;\n");

    const stale = await harness!.app
      .saveCapture(preview.id, { confirmed: true })
      .catch((error: unknown) => error);

    expect(stale).toBeInstanceOf(ApplicationError);
    expect((stale as ApplicationError).code).toBe("capture-stale");

    const fresh = await harness!.app.previewCapture(project.id);

    const capture = await harness!.app.saveCapture(fresh.id, {
      confirmed: true,
    });

    expect(capture.version).toBe(1);
    expect(
      capture.files.find((file) => file.path === "src/index.ts")?.content
    ).toContain("value = 2");
    expect(await harness!.app.listCaptures(project.id)).toHaveLength(1);

    const reopened = await harness!.app.getCapture(capture.id);
    expect(reopened?.version).toBe(1);
    expect(reopened?.git?.head).toBe(capture.git?.head);
  });

  test("unchanged refresh creates no duplicate; changed refresh versions history; edits affect only future captures", async () => {
    const project = await createProject();
    const firstPreview = await harness!.app.previewCapture(project.id);

    const first = await harness!.app.saveCapture(firstPreview.id, {
      confirmed: true,
    });

    const unchanged = await harness!.app.refreshProjectContext(project.id);
    expect(unchanged.status).toBe("unchanged");
    expect(unchanged.capture?.id).toBe(first.id);
    expect(await harness!.app.listCaptures(project.id)).toHaveLength(1);

    write("src/index.ts", "export const value = 3;\n");
    const changed = await harness!.app.refreshProjectContext(project.id);
    expect(changed.status).toBe("changed");

    const second = await harness!.app.saveCapture(changed.preview.id, {
      confirmed: true,
    });

    expect(second.version).toBe(2);
    expect(
      second.files.find((file) => file.path === "src/index.ts")?.content
    ).toContain("value = 3");

    const profile = await harness!.app.getProfile();

    const editedProfile = await harness!.app.updateProfile({
      ...profile!,
      displayName: "Sam Edited",
    });

    const afterProfileEdit = await harness!.app.previewCapture(project.id);

    const third = await harness!.app.saveCapture(afterProfileEdit.id, {
      confirmed: true,
    });

    expect(third.version).toBe(3);
    expect(third.profileSnapshot.displayName).toBe("Sam Edited");

    const history = await harness!.app.listCaptures(project.id);
    expect(history.map((capture) => capture.version)).toEqual([3, 2, 1]);
    expect(
      history.find((capture) => capture.version === 1)?.profileSnapshot.version
    ).toBe(editedProfile.version - 1);
    expect(
      history
        .find((capture) => capture.version === 1)
        ?.files.find((file) => file.path === "src/index.ts")?.content
    ).toContain("value = 1");
  });

  test("project deletion removes its captures and documents but preserves the profile and other projects", async () => {
    const firstProject = await createProject();
    const secondProject = await createProject({ name: "Other" });
    const preview = await harness!.app.previewCapture(firstProject.id);
    await harness!.app.saveCapture(preview.id, { confirmed: true });

    const document = await harness!.app.extractDocument({
      path: join(repository, "README.md"),
      kind: "srs",
    });

    expect(document.ok).toBe(true);

    if (document.ok) {
      await harness!.app.confirmDocumentImport(document.document, {
        confirmed: true,
        projectId: firstProject.id,
      });
    }

    await harness!.app.deleteProject(firstProject.id);

    expect(await harness!.app.getProject(firstProject.id)).toBeNull();
    expect(await harness!.app.listCaptures(firstProject.id)).toEqual([]);
    expect((await harness!.app.exportData()).documents).toEqual([]);
    expect(await harness!.app.getProfile()).not.toBeNull();
    expect((await harness!.app.listProjects()).map((item) => item.id)).toEqual([
      secondProject.id,
    ]);
  });

  test("export is a complete JSON bundle and all-data deletion removes the local record", async () => {
    const project = await createProject();
    const preview = await harness!.app.previewCapture(project.id);

    const capture = await harness!.app.saveCapture(preview.id, {
      confirmed: true,
    });

    const exported = await harness!.app.exportData();
    expect(exported.application).toBe("Coderune");
    expect(exported.profile?.displayName).toBe("Sam");
    expect(exported.projects).toHaveLength(1);
    expect(exported.captures).toHaveLength(1);
    expect(
      (exported.captures[0] as { files: { content: string }[] }).files[0]
        ?.content
    ).toBeString();

    JSON.stringify(exported);
    expect((exported.captures[0] as { id: string }).id).toBe(capture.id);

    await harness!.app.deleteAllData();
    const empty = await harness!.app.exportData();
    expect(empty.profile).toBeNull();
    expect(empty.projects).toEqual([]);
    expect(empty.captures).toEqual([]);
    expect((await harness!.app.getOnboardingState()).requiredComplete).toBe(
      false
    );
  });
});
