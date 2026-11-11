import { expect, test } from "bun:test";

import { connectStdio } from "@gpuix/react/automation";
import { hasNativeTestRenderer } from "@gpuix/react/testing";

import { createTestHarness } from "./helpers";

const nativeTest = hasNativeTestRenderer ? test : test.skip;

nativeTest(
  "desktop dev launch initializes the window before loading Quartz fonts",
  async () => {
    const harness = await createTestHarness();
    harness.close();

    const child = Bun.spawn(
      [
        process.execPath,
        "--hot",
        `${import.meta.dir}/fixtures/desktop-startup.ts`,
        harness.dataDir,
      ],
      {
        stdin: "pipe",
        stdout: "pipe",
        stderr: "pipe",
      }
    );

    const stderr = new Response(child.stderr).text();
    let timeout: ReturnType<typeof setTimeout> | undefined;

    try {
      const connection = connectStdio({
        write: (chunk) => {
          child.stdin.write(chunk);
          child.stdin.flush();
        },
        feed: (listener) => {
          const decoder = new TextDecoder();
          void (async () => {
            for await (const chunk of child.stdout) {
              listener(decoder.decode(chunk, { stream: true }));
            }
          })();
        },
      });

      const ui = await Promise.race([
        connection,
        child.exited.then(async (code) => {
          throw new Error(`Desktop exited (${code}): ${await stderr}`);
        }),
        new Promise<never>((_, reject) => {
          timeout = setTimeout(
            () => reject(new Error("Desktop automation did not initialize")),
            5000
          );
        }),
      ]);

      await ui
        .getByTestId("field-display-name-or-pseudonym")
        .waitFor({ timeoutMs: 5000 });
      await ui
        .getByTestId("field-display-name-or-pseudonym")
        .fill("Startup probe");
      await ui
        .getByTestId("field-learning-goals")
        .fill("Verify live native startup");
      await ui.getByTestId("save-profile").press("enter");

      const projects = await ui
        .getByTestId("nav-projects")
        .waitFor({ timeoutMs: 5000 });

      expect(projects.bounds?.width).toBeGreaterThan(0);
      expect((await ui.call("getPaintedText", {})).text).toContain(
        "Set up a project"
      );
      await ui.close();
    } finally {
      clearTimeout(timeout);
      child.kill();
      await child.exited;
      harness.cleanup();
    }
  },
  15_000
);
