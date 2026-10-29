import nodePath from "node:path";

import type {
  DocumentExtractionRequest,
  DocumentExtractionResult,
  DocumentExtractor,
  DocumentImportErrorCode,
  ExtractedDocument,
  SupportedDocumentExtension,
} from "@coderune/core";
import {
  DEFAULT_DOCUMENT_LIMIT_BYTES,
  SUPPORTED_DOCUMENT_EXTENSIONS,
} from "@coderune/core";
import mammoth from "mammoth";
import { extractText, getDocumentProxy } from "unpdf";

const failure = (
  code: DocumentImportErrorCode,
  message: string,
  path: string
): DocumentExtractionResult => ({
  ok: false,
  error: { code, message, path, manualEntryAvailable: true },
});

const supportedExtensionNames = new Set<string>(SUPPORTED_DOCUMENT_EXTENSIONS);

const supportedExtension = (
  extension: string
): extension is SupportedDocumentExtension =>
  supportedExtensionNames.has(extension);

interface ExtractedText {
  text: string;
  pageCount: number | null;
}

const extractTextFor = async function extractTextFor(
  extension: SupportedDocumentExtension,
  bytes: Uint8Array,
  path: string
): Promise<ExtractedText | null> {
  if (
    extension === ".md" ||
    extension === ".markdown" ||
    extension === ".txt"
  ) {
    const text = new TextDecoder("utf-8", { fatal: false }).decode(bytes);

    if (text.includes("\u0000")) {
      throw new Error("Binary content in a text document.");
    }

    return text.trim() ? { text, pageCount: null } : null;
  }

  if (extension === ".docx") {
    const result = await mammoth.extractRawText({
      buffer: Buffer.from(bytes),
    });

    const text = result.value.trim();

    return text ? { text: result.value, pageCount: null } : null;
  }

  if (extension === ".pdf") {
    const document = await getDocumentProxy(bytes);
    const result = await extractText(document, { mergePages: false });
    const pages = Array.isArray(result.text) ? result.text : [result.text];
    const text = pages.join("\n\n");

    if (!text.trim()) {
      return null;
    }

    return { text, pageCount: result.totalPages };
  }

  throw new Error(`No extractor for ${path}`);
};

const formatBytes = function formatBytes(bytes: number): string {
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  }

  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KiB`;
  }

  return `${bytes} B`;
};

export class LocalDocumentExtractor implements DocumentExtractor {
  // The extractor contract is instance-based but this method holds no state.
  // oxlint-disable-next-line eslint/class-methods-use-this
  async extract(
    request: DocumentExtractionRequest
  ): Promise<DocumentExtractionResult> {
    const path = nodePath.resolve(request.path);
    const extension = nodePath.extname(path).toLowerCase();

    if (!supportedExtension(extension)) {
      return failure(
        "unsupported-document",
        `Unsupported document type "${extension || "unknown"}". Select a PDF, DOCX, Markdown, or plain-text file, or enter the information manually.`,
        path
      );
    }

    const file = Bun.file(path);

    if (!(await file.exists())) {
      return failure(
        "unreadable-document",
        "The selected document is missing or cannot be read. Enter the information manually instead.",
        path
      );
    }

    const maxBytes = request.maxBytes ?? DEFAULT_DOCUMENT_LIMIT_BYTES;

    if (file.size > maxBytes) {
      return failure(
        "document-too-large",
        `The selected document is ${formatBytes(file.size)}, above the ${formatBytes(maxBytes)} limit. Narrow the selection or enter the information manually.`,
        path
      );
    }

    const bytes = new Uint8Array(await file.arrayBuffer());
    const hash = new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
    const extractedAt = new Date().toISOString();
    const name = nodePath.basename(path);

    try {
      const extracted = await extractTextFor(extension, bytes, path);

      if (extracted === null) {
        return failure(
          "image-only-document",
          "No text could be extracted. The document may be scanned or image-only; enter the information manually or select a text-based document.",
          path
        );
      }

      const document: ExtractedDocument = {
        name,
        path,
        extension,
        kind: request.kind,
        hash,
        size: bytes.byteLength,
        text: extracted.text,
        extractedAt,
        pageCount: extracted.pageCount,
      };

      return { ok: true, document };
    } catch {
      return failure(
        "unreadable-document",
        "The selected document could not be read. It may be corrupt, encrypted, or image-only; enter the information manually instead.",
        path
      );
    }
  }
}
