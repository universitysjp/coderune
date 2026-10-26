import type {
  CodingAgentCapabilities,
  CodingAgentId,
} from "@coderune/core";

export interface AgentDefinition {
  id: CodingAgentId;
  name: string;
  executable: string;
  setupUrl: string;
  capabilities: CodingAgentCapabilities;
}

const commonRestrictions = [
  "Compatibility is not yet demonstrated on this platform.",
  "The coding agent keeps its own sign-in, provider, and model configuration.",
  "Draft runs use a temporary working directory and cannot change project files.",
];

export const AGENT_DEFINITIONS: Record<CodingAgentId, AgentDefinition> = {
  codex: {
    id: "codex",
    name: "Codex",
    executable: "codex",
    setupUrl: "https://developers.openai.com/codex/app-server",
    capabilities: {
      structuredOutput: true,
      cancellation: true,
      streaming: true,
      documentOnlyIsolation: "unverified",
      maxInputBytes: 20 * 1024 * 1024,
      restrictions: [
        ...commonRestrictions,
        "Draft turns run with read-only sandbox access.",
      ],
    },
  },
  "claude-code": {
    id: "claude-code",
    name: "Claude Code",
    executable: "claude",
    setupUrl: "https://code.claude.com/docs/en/headless",
    capabilities: {
      structuredOutput: true,
      cancellation: true,
      streaming: false,
      documentOnlyIsolation: "unverified",
      maxInputBytes: 10 * 1024 * 1024,
      restrictions: [
        ...commonRestrictions,
        "Prompts run in non-interactive mode with tools denied.",
      ],
    },
  },
  opencode: {
    id: "opencode",
    name: "OpenCode",
    executable: "opencode",
    setupUrl: "https://docs.opencode.ai/docs/acp/",
    capabilities: {
      structuredOutput: false,
      cancellation: true,
      streaming: true,
      documentOnlyIsolation: "unverified",
      maxInputBytes: 20 * 1024 * 1024,
      restrictions: [
        ...commonRestrictions,
        "ACP permissions requested during a draft are rejected.",
        "Draft JSON is validated by Coderune before review.",
      ],
    },
  },
  antigravity: {
    id: "antigravity",
    name: "Antigravity",
    executable: "agy",
    setupUrl: "https://antigravity.google/docs/cli/headless/",
    capabilities: {
      structuredOutput: true,
      cancellation: true,
      streaming: true,
      documentOnlyIsolation: "unverified",
      maxInputBytes: 20 * 1024 * 1024,
      restrictions: [
        ...commonRestrictions,
        "Headless runs use Antigravity sandbox mode and a temporary workspace.",
      ],
    },
  },
};
