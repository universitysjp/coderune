import { expect, test } from "bun:test";
import { chmodSync } from "node:fs";
import path from "node:path";

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
      expect(root.renderer.getAllText()).toContain("Set up a project");
    } finally {
      root.unmount();
      harness.close();
      harness.cleanup();
    }
  },
  15_000
);

nativeTest(
  "a directory passed to the desktop appears in the workspace and project form",
  async () => {
    const harness = await createTestHarness({ agents });
    await harness.app.confirmOnboarding({
      displayName: "Ada",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn testing"],
    });
    const root = createTestRoot({ width: 1280, height: 900 });

    try {
      await registerQuartzFonts(root.renderer);
      root.render(
        <App runtime={harness.runtime} initialDirectory={harness.dataDir} />
      );
      await settle();
      expect(root.renderer.getAllText()).toContain(harness.dataDir);
      const ui = await connectTest(root.renderer);
      await ui.getByTestId("nav-projects").press("enter");
      await settle();
      await ui.getByTestId("field-project-name").fill("Directory project");
      await ui.getByTestId("field-goals").fill("Test directory launch");
      await ui.getByTestId("create-project").press("enter");
      await settle();
      expect((await harness.app.listProjects())[0]?.rootPath).toBe(
        harness.dataDir
      );
    } finally {
      root.unmount();
      harness.close();
      harness.cleanup();
    }
  },
  15_000
);

nativeTest(
  "workspace tabs remain open, switch sections, and close back to the previous tab",
  async () => {
    const harness = await createTestHarness({ agents });
    await harness.app.confirmOnboarding({
      displayName: "Ada",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn testing"],
    });
    const root = createTestRoot({ width: 1280, height: 900 });
    try {
      await registerQuartzFonts(root.renderer);
      root.render(
        <App runtime={harness.runtime} initialDirectory={harness.dataDir} />
      );
      await settle();
      const ui = await connectTest(root.renderer);
      await ui.getByTestId("nav-projects").press("enter");
      await settle();
      await ui.getByTestId("field-project-name").fill("Unsaved project");
      await ui.getByTestId("nav-documents").press("enter");
      await settle();
      expect(await ui.getByTestId("tab-projects").count()).toBe(1);
      expect(await ui.getByTestId("tab-documents").count()).toBe(1);
      await ui.getByTestId("tab-documents").press("right");
      await settle();
      expect(root.renderer.getAllText()).toContain("Set up a project");
      await ui.getByTestId("tab-projects").press("enter");
      await settle();
      expect(root.renderer.getAllText()).toContain("Create project");
      await ui.getByTestId("close-tab-projects").press("enter");
      await settle();
      expect(await ui.getByTestId("tab-projects").count()).toBe(0);
      await ui.getByTestId("nav-projects").press("enter");
      await settle();
      await ui.getByTestId("field-goals").fill("Keep the draft");
      await ui.getByTestId("create-project").press("enter");
      await settle();
      expect((await harness.app.listProjects())[0]?.name).toBe(
        "Unsaved project"
      );
      await ui.getByTestId("close-tab-projects").press("enter");
      await settle();
      expect(await ui.getByTestId("tab-projects").count()).toBe(0);
      expect(await ui.getByTestId("tab-home").count()).toBe(1);
      await ui.getByTestId("tab-home").press("enter");
      await settle();
      expect(root.renderer.getAllText()).toContain("Open project");
    } finally {
      root.unmount();
      harness.close();
      harness.cleanup();
    }
  },
  15_000
);

nativeTest(
  "editor dropdown opens without moving the workspace and closes with Escape",
  async () => {
    const harness = await createTestHarness({ agents });
    await harness.app.confirmOnboarding({
      displayName: "Ada",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn testing"],
    });
    const root = createTestRoot({ width: 1280, height: 900 });
    try {
      await registerQuartzFonts(root.renderer);
      root.render(<App runtime={harness.runtime} />);
      await settle();
      const ui = await connectTest(root.renderer);
      const before = await ui.getByTestId("workspace-surface").element();
      await ui.getByTestId("open-editor-menu").click();
      await settle();
      expect(await ui.getByTestId("open-editor-menu-content").count()).toBe(1);
      await ui.getByTestId("open-editor-menu").click();
      await settle();
      expect(await ui.getByTestId("open-editor-menu-content").count()).toBe(0);
      await ui.getByTestId("open-editor-menu").press("enter");
      await settle();
      expect(await ui.getByTestId("open-editor-menu-content").count()).toBe(1);
      const after = await ui.getByTestId("workspace-surface").element();
      expect(after.bounds).toEqual(before.bounds);
      await ui.getByTestId("open-editor-menu-content").press("tab");
      await settle();
      expect(await ui.getByTestId("open-editor-menu-content").count()).toBe(0);
      expect(root.renderer.getFocusedElementId()).toBe(
        (await ui.getByTestId("nav-projects").element()).id
      );
      await ui.getByTestId("open-editor-menu").press("enter");
      await settle();
      await ui.getByTestId("open-editor-menu-content").press("shift-tab");
      await settle();
      expect(root.renderer.getFocusedElementId()).toBe(
        (await ui.getByTestId("tab-home").element()).id
      );
      await ui.getByTestId("open-editor-menu").press("enter");
      await settle();
      await ui.getByTestId("open-editor-menu-content").press("escape");
      await settle();
      expect(await ui.getByTestId("open-editor-menu-content").count()).toBe(0);
    } finally {
      root.unmount();
      harness.close();
      harness.cleanup();
    }
  },
  15_000
);

nativeTest(
  "choosing an editor opens the current directory each time",
  async () => {
    const harness = await createTestHarness({ agents });
    const previousPath = process.env.PATH;
    const command = path.join(harness.dataDir, "code");
    const output = path.join(harness.dataDir, "opened.txt");
    await Bun.write(command, `#!/bin/sh\nprintf '%s\\n' "$1" >> '${output}'\n`);
    chmodSync(command, 0o755);
    const cursor = path.join(harness.dataDir, "cursor");
    await Bun.write(cursor, "#!/bin/sh\nexit 0\n");
    chmodSync(cursor, 0o755);
    process.env.PATH = harness.dataDir;
    await harness.app.confirmOnboarding({
      displayName: "Ada",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn testing"],
    });
    const root = createTestRoot({ width: 1280, height: 900 });
    try {
      await registerQuartzFonts(root.renderer);
      root.render(
        <App runtime={harness.runtime} initialDirectory={harness.dataDir} />
      );
      await settle();
      const ui = await connectTest(root.renderer);
      for (let attempt = 0; attempt < 2; attempt += 1) {
        await ui.getByTestId("open-editor-menu").press("enter");
        await settle();
        const codeItem = await ui
          .getByTestId("open-editor-menu-option-code")
          .element();
        const cursorItem = await ui
          .getByTestId("open-editor-menu-option-cursor")
          .element();
        const tree = root.renderer.getA11yTree();
        expect(
          Object.values(tree.nodes ?? {}).some(
            (node) =>
              node.aria?.role === "MenuItem" &&
              node.aria?.label === "Visual Studio Code"
          )
        ).toBe(true);
        expect(root.renderer.getFocusedElementId()).toBe(codeItem.id);
        await ui.getByTestId("open-editor-menu-option-code").press("down");
        expect(root.renderer.getFocusedElementId()).toBe(cursorItem.id);
        if (attempt === 0) {
          await ui.getByTestId("open-editor-menu-option-cursor").press("up");
          await ui.getByTestId("open-editor-menu-option-code").press("enter");
        } else {
          await ui.getByTestId("open-editor-menu-option-code").click();
        }
        let lines: string[] = [];
        for (let retries = 0; retries < 40; retries += 1) {
          if (await Bun.file(output).exists()) {
            lines = (await Bun.file(output).text()).trim().split("\n");
            if (lines.length > attempt) break;
          }
          await Bun.sleep(50);
        }
        if (lines.length <= attempt) {
          throw new Error(
            `Editor did not launch: ${root.renderer.getAllText().join(" | ")}`
          );
        }
      }
      expect((await Bun.file(output).text()).trim().split("\n")).toEqual([
        harness.dataDir,
        harness.dataDir,
      ]);
    } finally {
      if (previousPath === undefined) delete process.env.PATH;
      else process.env.PATH = previousPath;
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

      await ui.getByTestId("nav-projects").hover();
      await settle();
      expect(await ui.getByTestId("tooltip-nav-projects").count()).toBe(1);

      for (const section of [
        "profile",
        "projects",
        "documents",
        "agents",
        "capture",
        "data",
      ]) {
        const nav = ui.getByTestId(`nav-${section}`);
        expect(await nav.count(), section).toBe(1);
        const target = await nav.element();
        root.renderer.nativeSimulateKeyDown(target.id, "space");
        root.renderer.nativeSimulateKeyUp(target.id, "space");
        await settle();
        expect(
          await nav.count(),
          `navigation remains visible after ${section}`
        ).toBe(1);
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
