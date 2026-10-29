import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { ApplicationError, draftJsonSchema } from "@coderune/core";
import type { AgentDraftRequest, JsonObject, JsonValue } from "@coderune/core";

import { AGENT_DEFINITIONS } from "../definitions";
import { invalidOutput, parseJsonObject, requireSuccess } from "../parse";
import { runProcess } from "../process";

interface AntigravityEnvelope {
  event?: string;
  status?: string;
  error?: string;
  response?: string;
  structured_output?: JsonValue;
  result?: {
    status?: string;
    error?: string;
    response?: string;
    structured_output?: JsonValue;
  };
}

export const draftWithAntigravity = async function draftWithAntigravity(
  request: AgentDraftRequest,
  options: { signal?: AbortSignal; timeoutMs?: number } = {}
): Promise<JsonObject> {
  const definition = AGENT_DEFINITIONS.antigravity;
  const cwd = mkdtempSync(path.join(tmpdir(), "coderune-antigravity-"));

  const input = `${JSON.stringify({
    event: "user",
    message: { content: request.text },
  })}\n`;

  try {
    const result = await runProcess(
      [
        definition.executable,
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--json-schema",
        JSON.stringify(draftJsonSchema(request.kind)),
        "--sandbox",
        "--print-timeout",
        "2m",
      ],
      {
        cwd,
        input,
        signal: options.signal,
        timeoutMs: options.timeoutMs,
      }
    );

    requireSuccess(result.exitCode, result.stderr, definition.name);

    let final: AntigravityEnvelope | null = null;

    for (const line of result.stdout.split("\n")) {
      if (!line.trim()) {
        continue;
      }

      try {
        // SAFETY: Each stdout line is a JSON envelope emitted by the Antigravity CLI; its event field is checked below.
        const message = JSON.parse(line) as AntigravityEnvelope;

        if (message.event === "result") {
          final = message.result ?? message;
        }
      } catch {
        continue;
      }
    }

    if (!final) {
      throw invalidOutput();
    }

    if (final.status !== "SUCCESS") {
      const detail =
        final.error ??
        final.response ??
        "Antigravity did not complete the draft.";

      let code: "agent-cancelled" | "agent-unavailable" | "agent-failed";

      if (final.status === "CANCELED" || final.status === "INTERRUPTED") {
        code = "agent-cancelled";
      } else if (detail.toLowerCase().includes("auth")) {
        code = "agent-unavailable";
      } else {
        code = "agent-failed";
      }

      throw new ApplicationError(code, detail);
    }

    if (final.structured_output !== undefined) {
      return parseJsonObject(final.structured_output);
    }

    if (final.response !== undefined) {
      return parseJsonObject(final.response);
    }

    throw invalidOutput();
  } finally {
    rmSync(cwd, { force: true, recursive: true });
  }
};
