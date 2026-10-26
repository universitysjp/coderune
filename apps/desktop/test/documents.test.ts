import { afterEach, describe, expect, test } from "bun:test";
import { join } from "node:path";

import { createTestHarness } from "./helpers";
import type { TestHarness } from "./helpers";

const fixtures = join(import.meta.dir, "..", "fixtures");

let harness: TestHarness | undefined;

afterEach(() => {
  harness?.close();
  harness?.cleanup();
  harness = undefined;
});

describe("local document extraction", () => {
  test("text-based PDF, DOCX, Markdown, and TXT files extract locally", async () => {
    harness = await createTestHarness();

    const pdf = await harness.app.extractDocument({
      path: join(fixtures, "simple.pdf"),
      kind: "cv",
    });

    expect(pdf.ok).toBe(true);

    if (pdf.ok) {
      expect(pdf.document.text).toContain("learn distributed systems");
      expect(pdf.document.pageCount).toBe(1);
      expect(pdf.document.extension).toBe(".pdf");
      expect(pdf.document.hash).toMatch(/^[a-f0-9]{64}$/);
    }

    const docx = await harness.app.extractDocument({
      path: join(fixtures, "simple.docx"),
      kind: "cv",
    });

    expect(docx.ok).toBe(true);

    if (docx.ok) {
      expect(docx.document.text).toContain("Jordan Lee");
      expect(docx.document.text).toContain("learn Python testing");
    }

    for (const name of ["notes.md", "notes.txt"]) {
      const text = await harness.app.extractDocument({
        path: join(fixtures, name),
        kind: "cv",
      });

      expect(text.ok).toBe(true);

      if (text.ok) {
        expect(text.document.text.length).toBeGreaterThan(10);
      }
    }
  });

  test("unsupported, image-only, unreadable, and oversized documents explain the problem and keep manual entry available", async () => {
    harness = await createTestHarness();

    const unsupported = await harness.app.extractDocument({
      path: join(fixtures, "unsupported.bin"),
      kind: "cv",
    });

    expect(unsupported.ok).toBe(false);

    if (!unsupported.ok) {
      expect(unsupported.error.code).toBe("unsupported-document");
      expect(unsupported.error.manualEntryAvailable).toBe(true);
    }

    const imageOnly = await harness.app.extractDocument({
      path: join(fixtures, "image-only.pdf"),
      kind: "srs",
    });

    expect(imageOnly.ok).toBe(false);

    if (!imageOnly.ok) {
      expect(imageOnly.error.code).toBe("image-only-document");
      expect(imageOnly.error.manualEntryAvailable).toBe(true);
    }

    const missing = await harness.app.extractDocument({
      path: join(fixtures, "missing.pdf"),
      kind: "cv",
    });

    expect(missing.ok).toBe(false);

    if (!missing.ok) {
      expect(missing.error.code).toBe("unreadable-document");
    }

    const oversized = await harness.app.extractDocument({
      path: join(fixtures, "notes.md"),
      kind: "cv",
      maxBytes: 10,
    });

    expect(oversized.ok).toBe(false);

    if (!oversized.ok) {
      expect(oversized.error.code).toBe("document-too-large");
      expect(oversized.error.message).toContain("Narrow the selection");
    }
  });
});
