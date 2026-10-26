import { afterEach, describe, expect, test } from "bun:test";

import { createHarnessIn, createTestHarness } from "./helpers";
import type { TestHarness } from "./helpers";

let harness: TestHarness | undefined;

afterEach(() => {
  harness?.close();
  harness?.cleanup();
  harness = undefined;
});

describe("developer onboarding", () => {
  test("project work stays gated until a baseline is confirmed and the profile survives reopen", async () => {
    harness = await createTestHarness();
    const { app } = harness;

    const start = await app.getOnboardingState();
    expect(start.requiredComplete).toBe(false);
    expect(start.canStartProject).toBe(false);
    expect(start.profile).toBeNull();
    expect(start.missing).toContain("display name");
    expect(start.missing).toContain("experience stage");
    expect(start.missing).toContain("initial skills statement");
    expect(start.missing).toContain("learning goal");

    await app.saveOnboardingDraft({
      displayName: "Sam Rivers",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn TypeScript"],
    });

    const { dataDir } = harness;
    await harness.close();
    harness = await createHarnessIn(dataDir);
    const resumed = await harness.app.getOnboardingState();
    expect(resumed.requiredComplete).toBe(false);
    expect(resumed.draft).toMatchObject({
      displayName: "Sam Rivers",
      newToCoding: true,
      learningGoals: ["Learn TypeScript"],
    });

    const profile = await harness.app.confirmOnboarding({
      displayName: "Sam Rivers",
      experienceStage: "new-to-coding",
      newToCoding: true,
      skillsStatement: "New to coding; no skills listed yet.",
      learningGoals: ["Learn TypeScript"],
      selfReportedSkills: [],
    });

    expect(profile.version).toBe(1);
    expect(profile.displayName).toBe("Sam Rivers");

    const complete = await harness.app.getOnboardingState();
    expect(complete.requiredComplete).toBe(true);
    expect(complete.canStartProject).toBe(true);
    expect(complete.missing).toEqual([]);
    expect(complete.profile).toEqual(profile);

    await harness.close();
    harness = await createHarnessIn(dataDir);
    const reopened = await harness.app.getOnboardingState();
    expect(reopened.requiredComplete).toBe(true);
    expect(reopened.profile?.displayName).toBe("Sam Rivers");
    expect(reopened.profile?.version).toBe(1);
  });
});
