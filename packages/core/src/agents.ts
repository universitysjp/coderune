import type { CvDraft, SrsDraft } from "./drafts";

export { type AgentDraft } from "./drafts";

import type { DocumentKind, ImportedDocument, Project } from "./types";

export const CODING_AGENT_IDS = [
  "codex",
  "claude-code",
  "opencode",
  "antigravity",
] as const;

export type CodingAgentId = (typeof CODING_AGENT_IDS)[number];

export type CodingAgentAvailability = "ready" | "not-installed" | "unavailable";

export type IsolationStatus = "enforced" | "unverified" | "unsupported";

export interface CodingAgentCapabilities {
  structuredOutput: boolean;
  cancellation: boolean;
  streaming: boolean;
  documentOnlyIsolation: IsolationStatus;
  maxInputBytes: number | null;
  restrictions: string[];
}

export interface CodingAgentStatus {
  id: CodingAgentId;
  name: string;
  availability: CodingAgentAvailability;
  detail: string;
  setupUrl: string;
  capabilities: CodingAgentCapabilities;
  verified: boolean;
}

export interface AgentDraftRequest {
  agentId: CodingAgentId;
  kind: DocumentKind;
  text: string;
  document: {
    id: string;
    name: string;
    path: string;
    hash: string;
  };
  project: {
    id: string;
    name: string;
  } | null;
}

export interface DraftSharingPreview {
  agent: CodingAgentStatus;
  kind: DocumentKind;
  document: ImportedDocument;
  project: Project | null;
  text: string;
  estimatedBytes: number;
  maxInputBytes: number | null;
  fits: boolean;
  createdAt: string;
}

export interface AgentDraftResult {
  agent: CodingAgentStatus;
  draft: CvDraft | SrsDraft;
}

export interface AgentDraftRunOptions {
  signal?: AbortSignal;
  onProgress?: (message: string) => void;
}

export interface CodingAgentGateway {
  listAgents: () => Promise<CodingAgentStatus[]>;
  getAgent: (id: CodingAgentId) => Promise<CodingAgentStatus | null>;
  draft: (
    request: AgentDraftRequest,
    options?: AgentDraftRunOptions
  ) => Promise<unknown>;
}
