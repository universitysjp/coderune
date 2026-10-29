import { expect, test } from "bun:test";

import { registerQuartzFonts } from "@coderune/quartz/native";
import { connectTest } from "@gpuix/react/automation";
import { createTestRoot, hasNativeTestRenderer } from "@gpuix/react/testing";

import { App } from "../app";
import { createTestHarness } from "./helpers";

const nativeTest = hasNativeTestRenderer ? test : test.skip;

const agents = {
  listAgents: () => [],
  getAgent: () => null,
  draft: async () => {
    throw new Error("No agent was requested");
  },
};

async function settle() {
  await Bun.sleep(120);
}

nativeTest(
  "developer can confirm a baseline through the native Quartz form",
  async () => {
    const harness = await createTestHarness({ agents });
    const root = createTestRoot({ width: 1280, height: 1600 });

    try {
      await registerQuartzFonts(root.renderer);
      root.render(<App runtime={harness.runtime} />);
      await settle();
      const ui = await connectTest(root.renderer);
      await ui.getByTestId("field-display-name-or-pseudonym").fill("Ada");
      await ui
        .getByTestId("field-learning-goals")
        .fill("Learn native UI testing");
      await ui.getByTestId("save-profile").press("enter");
      await settle();
      const saved = await harness.app.getProfile();
      expect(saved?.displayName).toBe("Ada");
      expect(saved?.learningGoals).toEqual(["Learn native UI testing"]);
      expect(root.renderer.getAllText()).toContain("Developer profile");
    } finally {
      root.unmount();
      harness.close();
      harness.cleanup();
    }
  },
  15_000
);

nativeTest(
  "all workspace sections remain named and reachable at a narrow width",
  async () => {
    const harness = await createTestHarness({ agents });
    await harness.app.confirmOnboarding({
      displayName: "Ada",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn testing"],
    });
    const root = createTestRoot({ width: 390, height: 900 });

    try {
      await registerQuartzFonts(root.renderer);
      root.render(<App runtime={harness.runtime} />);
      await settle();
      const ui = await connectTest(root.renderer);

      for (const section of [
        "profile",
        "projects",
        "documents",
        "agents",
        "capture",
        "data",
      ]) {
        await ui.getByTestId("toggle-navigation").press("enter");
        await settle();
        const nav = ui.getByTestId(`nav-${section}`);
        expect(await nav.count(), section).toBe(1);
        const target = await nav.element();
        root.renderer.nativeSimulateKeyDown(target.id, "space");
        root.renderer.nativeSimulateKeyUp(target.id, "space");
        await settle();
        expect(await nav.count(), `navigation closes after ${section}`).toBe(0);
        const tree = root.renderer.getA11yTree();

        const buttons = Object.values(tree.nodes ?? {}).filter(
          (node) => node.aria?.role === "Button"
        );

        expect(buttons.length).toBeGreaterThan(0);
        expect(buttons.every((node) => Boolean(node.aria?.label))).toBe(true);
        const focus = root.renderer.getFocusedElementId();
        root.renderer.simulateKeystrokes("tab");
        expect(root.renderer.getFocusedElementId()).not.toBe(focus);
      }

      expect(await harness.app.getProfile()).not.toBeNull();
    } finally {
      root.unmount();
      harness.close();
      harness.cleanup();
    }
  },
  15_000
);
