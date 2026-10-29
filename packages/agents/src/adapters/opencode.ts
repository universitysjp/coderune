import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { ApplicationError } from "@coderune/core";
import type { AgentDraftRequest } from "@coderune/core";

import { AGENT_DEFINITIONS } from "../definitions";
import { invalidOutput, parseJsonObject } from "../parse";
import { JsonRpcPeer } from "../process";
import type { JsonRpcRequest } from "../process";

interface InitializeResult {
  protocolVersion?: number;
  authMethods?: unknown[];
}

interface SessionResult {
  sessionId?: string;
}

interface PromptResult {
  stopReason?: string;
}

interface SessionUpdate {
  update?: {
    sessionUpdate?: string;
    content?: { type?: string; text?: string };
  };
}

interface PermissionOption {
  optionId?: string;
  kind?: string;
}

interface PermissionRequest {
  options?: PermissionOption[];
}

const permissionResponse = function permissionResponse(
  request: JsonRpcRequest
) {
  if (request.method !== "session/request_permission") {
    throw new Error(`Unsupported agent request: ${request.method}`);
  }

  // SAFETY: The request method was checked above, so params follows the ACP permission request schema.
  const params = request.params as PermissionRequest | undefined;

  const reject = params?.options?.find(
    (option) => option.kind === "reject_once" || option.kind === "reject_always"
  );

  if (reject?.optionId) {
    return {
      outcome: { outcome: "selected", optionId: reject.optionId },
    };
  }

  return { outcome: { outcome: "cancelled" } };
};

export const draftWithOpencode = async function draftWithOpencode(
  request: AgentDraftRequest,
  options: { signal?: AbortSignal; timeoutMs?: number } = {}
): Promise<unknown> {
  const definition = AGENT_DEFINITIONS.opencode;
  const cwd = mkdtempSync(path.join(tmpdir(), "coderune-opencode-"));
  const timeoutMs = options.timeoutMs ?? 120_000;

  const peer = new JsonRpcPeer(
    [definition.executable, "acp"],
    { cwd, signal: options.signal, timeoutMs },
    permissionResponse
  );

  let text = "";

  const off = peer.onNotification((notification) => {
    if (notification.method !== "session/update") {
      return;
    }

    // SAFETY: The notification method was checked above, so params follows the ACP session update schema.
    const params = notification.params as SessionUpdate | undefined;
    const update = params?.update;

    if (
      update?.sessionUpdate === "agent_message_chunk" &&
      update.content?.type === "text"
    ) {
      text += update.content.text ?? "";
    }
  });

  try {
    // SAFETY: The ACP initialize response is validated by the protocol version check below.
    const initialized = (await peer.request("initialize", {
      protocolVersion: 1,
      clientCapabilities: {
        fs: { readTextFile: false, writeTextFile: false },
        terminal: false,
      },
      clientInfo: {
        name: "coderune",
        title: "Coderune",
        version: "0.0.0",
      },
    })) as InitializeResult;

    if (initialized.protocolVersion !== 1) {
      throw new ApplicationError(
        "agent-unavailable",
        "OpenCode does not support the required ACP protocol version."
      );
    }

    // SAFETY: The ACP session/new response is checked for a session id below.
    const session = (await peer.request("session/new", {
      cwd,
      mcpServers: [],
    })) as SessionResult;

    if (!session.sessionId) {
      throw new ApplicationError(
        "agent-failed",
        "OpenCode did not return an ACP session identifier."
      );
    }

    // SAFETY: The ACP session/prompt response is checked for its stop reason below.
    const prompted = (await peer.request("session/prompt", {
      sessionId: session.sessionId,
      prompt: [{ type: "text", text: request.text }],
    })) as PromptResult;

    if (prompted.stopReason === "cancelled") {
      throw new ApplicationError(
        "agent-cancelled",
        "The OpenCode draft was cancelled."
      );
    }

    if (prompted.stopReason !== "end_turn") {
      throw new ApplicationError(
        "agent-failed",
        `OpenCode stopped before completing the draft (${prompted.stopReason ?? "unknown"}).`
      );
    }

    if (!text.trim()) {
      throw invalidOutput();
    }

    return parseJsonObject(text);
  } finally {
    off();
    peer.close();
    rmSync(cwd, { force: true, recursive: true });
  }
};
