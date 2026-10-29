import { afterEach, describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { join } from "node:path";

import { ApplicationError } from "@coderune/core";

import { captureError, createTestHarness } from "./helpers";
import type { TestHarness } from "./helpers";

const fixtures = join(import.meta.dir, "..", "fixtures");
let harness: TestHarness | undefined;

afterEach(() => {
  harness?.close();
  harness?.cleanup();
  harness = undefined;
});

describe("approved document retention", () => {
  test("extracted text is saved only after review, without copying the original file", async () => {
    harness = await createTestHarness();

    const extracted = await harness.app.extractDocument({
      path: join(fixtures, "simple.docx"),
      kind: "cv",
    });

    expect(extracted.ok).toBe(true);

    if (!extracted.ok) {
      return;
    }

    const unconfirmed = await captureError(() =>
      harness!.app.confirmDocumentImport(extracted.document, {
        confirmed: false,
      })
    );

    expect(unconfirmed).toBeInstanceOf(ApplicationError);
    expect((unconfirmed as ApplicationError).code).toBe(
      "confirmation-required"
    );
    expect((await harness.app.exportData()).documents).toEqual([]);

    const saved = await harness.app.confirmDocumentImport(extracted.document, {
      confirmed: true,
    });

    expect(saved.text).toContain("Jordan Lee");
    expect(saved.hash).toBe(extracted.document.hash);
    expect(saved.path).toBe(extracted.document.path);
    expect(saved.projectId).toBeNull();
    expect(existsSync(saved.path)).toBe(true);

    const exported = await harness.app.exportData();
    expect(exported.documents).toHaveLength(1);
    expect((exported.documents[0] as { text: string }).text).toContain(
      "Jordan Lee"
    );
  });
});
