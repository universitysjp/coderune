import { afterEach, describe, expect, test } from "bun:test";

import { ValidationError } from "@coderune/core";

import { createTestHarness } from "./helpers";
import type { TestHarness } from "./helpers";

let harness: TestHarness | undefined;

afterEach(() => {
  harness?.close();
  harness?.cleanup();
  harness = undefined;
});

const baseline = {
  displayName: "Alex Chen",
  experienceStage: "early-career" as const,
  newToCoding: false,
  skillsStatement: "Comfortable with JavaScript; learning SQL.",
  learningGoals: ["Learn SQL joins"],
  selfReportedSkills: [
    { topic: "TypeScript", rating: "intermediate" as const },
    { topic: "SQL", rating: null },
  ],
};

describe("developer profile", () => {
  test("incomplete required fields are rejected with specific reasons", async () => {
    harness = await createTestHarness();

    const error = await harness.app
      .confirmOnboarding({
        displayName: "  ",
        experienceStage: "early-career",
        newToCoding: false,
        skillsStatement: "",
        learningGoals: ["  "],
      })
      .catch((error: unknown) => error);

    expect(error).toBeInstanceOf(ValidationError);
    expect((error as ValidationError).details).toEqual([
      "display name",
      "initial skills statement",
      "learning goal",
    ]);
    expect(await harness.app.getProfile()).toBeNull();
  });

  test("a new-to-coding statement completes onboarding without invented skills", async () => {
    harness = await createTestHarness();

    const profile = await harness.app.confirmOnboarding({
      displayName: "Pseudonym",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn the terminal"],
    });

    expect(profile.skillsStatement).toBe(
      "New to coding; no skills listed yet."
    );
    expect(profile.selfReportedSkills).toEqual([]);
  });

  test("editing the profile version preserves the earlier version in export", async () => {
    harness = await createTestHarness();
    const first = await harness.app.confirmOnboarding(baseline);

    const edited = await harness.app.updateProfile({
      ...baseline,
      displayName: "Alex C.",
      learningGoals: ["Learn SQL joins", "Learn testing"],
    });

    expect(edited.version).toBe(first.version + 1);
    expect(edited.createdAt).toBe(first.createdAt);
    expect(edited.displayName).toBe("Alex C.");

    const exported = await harness.app.exportData();
    expect(exported.profile?.displayName).toBe("Alex C.");
    expect(exported.profileVersions.map((version) => version.version)).toEqual([
      1, 2,
    ]);
    expect(exported.profileVersions[0]?.displayName).toBe("Alex Chen");
    expect(exported.profileVersions[1]?.displayName).toBe("Alex C.");
  });

  test("unrated self-reported skills stay explicitly unrated", async () => {
    harness = await createTestHarness();
    const profile = await harness.app.confirmOnboarding(baseline);
    expect(profile.selfReportedSkills).toEqual([
      { topic: "TypeScript", rating: "intermediate" },
      { topic: "SQL", rating: null },
    ]);
  });
});
