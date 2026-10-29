import { ApplicationError } from "@coderune/core";
import type {
  AgentDraftRequest,
  CodingAgentGateway,
  CodingAgentId,
  CodingAgentStatus,
  JsonObject,
} from "@coderune/core";

import { draftWithAntigravity } from "./adapters/antigravity";
import { draftWithClaude } from "./adapters/claude";
import { draftWithCodex } from "./adapters/codex";
import { draftWithOpencode } from "./adapters/opencode";
import { AGENT_DEFINITIONS } from "./definitions";

const AGENT_ORDER: CodingAgentId[] = [
  "codex",
  "claude-code",
  "opencode",
  "antigravity",
];

export interface LocalCodingAgentGatewayOptions {
  which?: (executable: string) => string | null;
  timeoutMs?: number;
}

export class LocalCodingAgentGateway implements CodingAgentGateway {
  readonly #which: (executable: string) => string | null;
  readonly #timeoutMs: number | undefined;

  constructor(options: LocalCodingAgentGatewayOptions = {}) {
    this.#which = options.which ?? ((executable) => Bun.which(executable));
    this.#timeoutMs = options.timeoutMs;
  }

  listAgents(): CodingAgentStatus[] {
    return AGENT_ORDER.map((id) => this.#status(id));
  }

  getAgent(id: CodingAgentId): CodingAgentStatus | null {
    if (!(id in AGENT_DEFINITIONS)) {
      return null;
    }

    return this.#status(id);
  }

  draft(
    request: AgentDraftRequest,
    options: {
      signal?: AbortSignal;
      onProgress?: (message: string) => void;
    } = {}
  ): Promise<JsonObject> {
    const status = this.#status(request.agentId);

    if (status.availability !== "ready") {
      throw new ApplicationError(
        "agent-unavailable",
        `${status.name}: ${status.detail} No other agent was used.`
      );
    }

    options.onProgress?.(`Starting ${status.name}...`);

    const runOptions = {
      signal: options.signal,
      timeoutMs: this.#timeoutMs,
      onProgress: options.onProgress,
    };

    switch (request.agentId) {
      case "codex": {
        return draftWithCodex(request, runOptions);
      }

      case "claude-code": {
        return draftWithClaude(request, runOptions);
      }

      case "opencode": {
        return draftWithOpencode(request, runOptions);
      }

      case "antigravity": {
        return draftWithAntigravity(request, runOptions);
      }

      default: {
        throw new ApplicationError(
          "agent-unavailable",
          `Unsupported coding agent: ${request.agentId}`
        );
      }
    }
  }

  #status(id: CodingAgentId): CodingAgentStatus {
    const definition = AGENT_DEFINITIONS[id];
    const path = this.#which(definition.executable);

    return {
      id: definition.id,
      name: definition.name,
      availability: path ? "ready" : "not-installed",
      detail: path
        ? `Installed at ${path}. Sign-in, provider, and model choice remain with ${definition.name}.`
        : `${definition.name} was not found on PATH. Install it and complete its own sign-in.`,
      setupUrl: definition.setupUrl,
      capabilities: definition.capabilities,
      verified: false,
    };
  }
}
