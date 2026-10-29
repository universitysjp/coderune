import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { ApplicationError, draftJsonSchema } from "@coderune/core";
import type { AgentDraftRequest, JsonObject } from "@coderune/core";

import { AGENT_DEFINITIONS } from "../definitions";
import {
  invalidOutput,
  parseJsonObject,
  parseJsonString,
  requireSuccess,
} from "../parse";
import { runProcess } from "../process";

export const draftWithClaude = async function draftWithClaude(
  request: AgentDraftRequest,
  options: { signal?: AbortSignal; timeoutMs?: number } = {}
): Promise<JsonObject> {
  const definition = AGENT_DEFINITIONS["claude-code"];
  const cwd = mkdtempSync(path.join(tmpdir(), "coderune-claude-"));

  try {
    const result = await runProcess(
      [
        definition.executable,
        "-p",
        "Read the drafting request on stdin. Return only the requested JSON object.",
        "--output-format",
        "json",
        "--json-schema",
        JSON.stringify(draftJsonSchema(request.kind)),
        "--permission-mode",
        "dontAsk",
        "--permission-prompts",
        "none",
        "--disallowedTools",
        "Bash,Edit,Write,NotebookEdit,WebFetch,WebSearch",
      ],
      {
        cwd,
        input: request.text,
        signal: options.signal,
        timeoutMs: options.timeoutMs,
      }
    );

    requireSuccess(result.exitCode, result.stderr, definition.name);

    const envelope = parseJsonObject(result.stdout);

    if (envelope.is_error === true) {
      const detail =
        parseJsonString(envelope.result) ??
        "Claude Code could not complete the draft.";

      throw new ApplicationError(
        detail.toLowerCase().includes("auth") ||
          detail.toLowerCase().includes("login")
          ? "agent-unavailable"
          : "agent-failed",
        detail
      );
    }

    if (envelope.structured_output !== undefined) {
      return parseJsonObject(envelope.structured_output);
    }

    if (envelope.result !== undefined) {
      return parseJsonObject(envelope.result);
    }

    throw invalidOutput();
  } finally {
    rmSync(cwd, { force: true, recursive: true });
  }
};
