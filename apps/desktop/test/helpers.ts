import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { createApplication } from "../src/application";
import type {
  ApplicationRuntime,
  CreateApplicationOptions,
} from "../src/application";

export interface TestHarness {
  app: ApplicationRuntime["operations"];
  runtime: ApplicationRuntime;
  dataDir: string;
  close: () => void;
  cleanup: () => void;
}

export const captureError = async (action: () => unknown): Promise<unknown> => {
  try {
    await action();
  } catch (error) {
    return error;
  }

  throw new Error("Expected the operation to fail.");
};

export async function createTestHarness(
  options: Partial<Omit<CreateApplicationOptions, "dataDir">> = {}
): Promise<TestHarness> {
  const dataDir = mkdtempSync(join(tmpdir(), "coderune-test-"));

  return createHarnessIn(dataDir, options);
}

export async function createHarnessIn(
  dataDir: string,
  options: Partial<Omit<CreateApplicationOptions, "dataDir">> = {}
): Promise<TestHarness> {
  const runtime = await createApplication({ dataDir, ...options });

  return {
    app: runtime.operations,
    runtime,
    dataDir,
    close: () => runtime.close(),
    cleanup: () => rmSync(dataDir, { force: true, recursive: true }),
  };
}
