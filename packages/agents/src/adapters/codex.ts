import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { ApplicationError, draftJsonSchema } from "@coderune/core";
import type { AgentDraftRequest } from "@coderune/core";

import { AGENT_DEFINITIONS } from "../definitions";
import { invalidOutput, parseJsonObject } from "../parse";
import { JsonRpcPeer } from "../process";
import type { JsonRpcRequest } from "../process";

interface ThreadResult {
  thread?: { id?: string };
}

interface TurnResult {
  turn?: { id?: string };
}

interface TurnCompleted {
  turn?: {
    id?: string;
    status?: string;
    error?: { message?: string };
  };
}

interface ItemCompleted {
  item?: { type?: string; text?: string };
}

const approvalResponse = function approvalResponse(request: JsonRpcRequest) {
  if (
    request.method === "item/commandExecution/requestApproval" ||
    request.method === "item/fileChange/requestApproval"
  ) {
    return { decision: "decline" };
  }

  if (request.method === "item/tool/requestUserInput") {
    throw new Error("Interactive tool input is not available for draft jobs.");
  }

  if (request.method === "mcpServer/elicitation/request") {
    return { action: "decline", content: null };
  }

  throw new Error(`Unsupported agent request: ${request.method}`);
};

export const draftWithCodex = async function draftWithCodex(
  request: AgentDraftRequest,
  options: { signal?: AbortSignal; timeoutMs?: number } = {}
): Promise<unknown> {
  const definition = AGENT_DEFINITIONS.codex;
  const cwd = mkdtempSync(path.join(tmpdir(), "coderune-codex-"));
  const timeoutMs = options.timeoutMs ?? 120_000;

  const peer = new JsonRpcPeer(
    [definition.executable, "app-server"],
    { cwd, signal: options.signal, timeoutMs },
    approvalResponse
  );

  let finalText: string | null = null;
  let turnId: string | null = null;

  let settle: ((result: { status: string; error?: string }) => void) | null =
    null;

  const turnCompleted = new Promise<{ status: string; error?: string }>(
    (resolve) => {
      settle = resolve;
    }
  );

  const off = peer.onNotification((notification) => {
    if (notification.method === "item/completed") {
      // SAFETY: The item/completed notification payload follows the Codex item schema checked below.
      const params = notification.params as ItemCompleted | undefined;

      if (params?.item?.type === "agentMessage") {
        finalText = params.item.text ?? finalText;
      }

      return;
    }

    if (notification.method === "turn/completed") {
      // SAFETY: The turn/completed notification payload is matched against the active turn id below.
      const params = notification.params as TurnCompleted | undefined;
      const turn = params?.turn;

      if (turnId && turn?.id && turn.id !== turnId) {
        return;
      }

      settle?.({
        status: turn?.status ?? "completed",
        error: turn?.error?.message,
      });
    }
  });

  let timeout: ReturnType<typeof setTimeout> | null = null;

  try {
    await peer.request("initialize", {
      clientInfo: {
        name: "coderune",
        title: "Coderune",
        version: "0.0.0",
      },
    });
    peer.notify("initialized", {});

    // SAFETY: The Codex thread/start response is checked for a thread id below.
    const threadResult = (await peer.request("thread/start", {
      cwd,
      approvalPolicy: "never",
      sandbox: "readOnly",
      serviceName: "coderune",
    })) as ThreadResult;

    const threadId = threadResult.thread?.id;

    if (!threadId) {
      throw new ApplicationError(
        "agent-failed",
        "Codex did not return a thread identifier."
      );
    }

    // SAFETY: The Codex turn/start response is read only after the request succeeds.
    const started = (await peer.request("turn/start", {
      threadId,
      cwd,
      approvalPolicy: "never",
      sandboxPolicy: { type: "readOnly" },
      input: [{ type: "text", text: request.text }],
      outputSchema: draftJsonSchema(request.kind),
    })) as TurnResult;

    turnId = started.turn?.id ?? null;

    const timedOut = new Promise<never>((_resolve, reject) => {
      timeout = setTimeout(() => {
        reject(
          new ApplicationError(
            "agent-timeout",
            "Codex did not finish the draft within the allowed time."
          )
        );
      }, timeoutMs);
    });

    const completion = await Promise.race([turnCompleted, timedOut]);

    if (completion.status === "failed") {
      throw new ApplicationError(
        "agent-failed",
        completion.error ?? "Codex failed to complete the draft."
      );
    }

    if (completion.status === "interrupted") {
      throw new ApplicationError(
        "agent-cancelled",
        "The Codex draft was cancelled."
      );
    }

    if (!finalText) {
      throw invalidOutput();
    }

    return parseJsonObject(finalText);
  } finally {
    if (timeout) {
      clearTimeout(timeout);
    }

    off();
    peer.close();
    rmSync(cwd, { force: true, recursive: true });
  }
};
