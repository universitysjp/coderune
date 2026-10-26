import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { ApplicationError } from "@coderune/core";

import { createTestHarness } from "./helpers";
import type { TestHarness } from "./helpers";

let harness: TestHarness | undefined;
let workspace: string;

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "coderune-project-"));
  mkdirSync(join(workspace, "readable"));
  writeFileSync(join(workspace, "not-a-directory.txt"), "plain file");
});

afterEach(() => {
  harness?.close();
  harness?.cleanup();
  harness = undefined;
  rmSync(workspace, { force: true, recursive: true });
});

async function completeOnboarding(): Promise<TestHarness> {
  const created = await createTestHarness();
  await created.app.confirmOnboarding({
    displayName: "Sam",
    experienceStage: "early-career",
    newToCoding: false,
    skillsStatement: "Some JavaScript",
    learningGoals: ["Learn testing"],
  });

  return created;
}

describe("project context", () => {
  test("project creation is blocked before the developer baseline is confirmed", async () => {
    harness = await createTestHarness();

    const error = await harness.app
      .createProject({
        name: "Blocked",
        rootPath: join(workspace, "readable"),
        goals: ["Ship it"],
      })
      .catch((error: unknown) => error);

    expect(error).toBeInstanceOf(ApplicationError);
    expect((error as ApplicationError).code).toBe("onboarding-required");
  });

  test("a project needs a name, a readable root folder, and at least one goal", async () => {
    harness = await completeOnboarding();

    const noName = await harness.app
      .createProject({
        name: " ",
        rootPath: join(workspace, "readable"),
        goals: ["Ship it"],
      })
      .catch((error: unknown) => error);

    expect(noName).toBeInstanceOf(ApplicationError);
    expect((noName as ApplicationError).code).toBe("validation");

    const noGoal = await harness.app
      .createProject({
        name: "No goal",
        rootPath: join(workspace, "readable"),
        goals: ["  "],
      })
      .catch((error: unknown) => error);

    expect(noGoal).toBeInstanceOf(ApplicationError);
    expect((noGoal as ApplicationError).code).toBe("validation");

    const unreadable = await harness.app
      .createProject({
        name: "Bad root",
        rootPath: join(workspace, "missing"),
        goals: ["Ship it"],
      })
      .catch((error: unknown) => error);

    expect(unreadable).toBeInstanceOf(ApplicationError);
    expect((unreadable as ApplicationError).code).toBe("unreadable-path");

    const notDirectory = await harness.app
      .createProject({
        name: "File root",
        rootPath: join(workspace, "not-a-directory.txt"),
        goals: ["Ship it"],
      })
      .catch((error: unknown) => error);

    expect(notDirectory).toBeInstanceOf(ApplicationError);
    expect((notDirectory as ApplicationError).code).toBe("unreadable-path");

    expect(await harness.app.listProjects()).toEqual([]);
  });

  test("project context keeps developer statements, supports SRS provenance, reuse, and editing", async () => {
    harness = await completeOnboarding();

    const project = await harness.app.createProject({
      name: "Bridge",
      rootPath: join(workspace, "readable"),
      goals: ["Capture local context"],
      purpose: "Help coding agents understand the project",
      businessLogic: "The developer approves each capture",
      rules: ["Do not collect secrets"],
      constraints: ["Local only"],
      decisions: ["SQLite is the source of truth"],
      acceptanceCriteria: ["A changed refresh creates a new version"],
      knowledge: ["The team reviews captures on Fridays"],
    });

    expect(project.version).toBe(1);
    expect(project.goals[0]?.text).toBe("Capture local context");
    expect(project.goals[0]?.provenance.kind).toBe("developer");
    expect(project.rules).toHaveLength(1);

    const reopened = await harness.app.getProject(project.id);
    expect(reopened?.name).toBe("Bridge");

    const edited = await harness.app.updateProject(project.id, {
      name: "Bridge 2",
      rootPath: project.rootPath,
      goals: ["Capture local context", "Export it"],
      purpose: project.purpose?.text,
      rules: project.rules.map((rule) => rule.text),
    });

    expect(edited.version).toBe(2);
    expect(edited.goals).toHaveLength(2);
    expect(edited.rules[0]?.provenance.kind).toBe("developer");

    expect((await harness.app.listProjects()).map((item) => item.name)).toEqual(
      ["Bridge 2"]
    );
  });
});
