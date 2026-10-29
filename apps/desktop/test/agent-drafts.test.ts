import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { ApplicationError } from "@coderune/core";
import type {
  AgentDraftRequest,
  CodingAgentGateway,
  CodingAgentId,
  CodingAgentStatus,
  ImportedDocument,
  JsonObject,
} from "@coderune/core";

import { captureError, createTestHarness } from "./helpers";
import type { TestHarness } from "./helpers";

const fixtures = join(import.meta.dir, "..", "fixtures");

let harness: TestHarness | undefined;
let workspace: string;

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "coderune-agents-"));
  mkdirSync(join(workspace, "project"));
});

afterEach(() => {
  harness?.close();
  harness?.cleanup();
  harness = undefined;
  rmSync(workspace, { force: true, recursive: true });
});

function agentStatus(
  id: CodingAgentId,
  availability: CodingAgentStatus["availability"] = "ready",
  maxInputBytes: number | null = 10 * 1024 * 1024
): CodingAgentStatus {
  return {
    id,
    name: id === "claude-code" ? "Claude Code" : id === "codex" ? "Codex" : id,
    availability,
    detail:
      availability === "ready"
        ? "Installed. Sign-in remains with the agent."
        : "Not installed.",
    setupUrl: "https://example.test/setup",
    capabilities: {
      structuredOutput: true,
      cancellation: true,
      streaming: false,
      documentOnlyIsolation: "enforced",
      maxInputBytes,
      restrictions: ["Runs in a temporary directory."],
    },
    verified: false,
  };
}

class ScriptedAgentGateway implements CodingAgentGateway {
  readonly calls: AgentDraftRequest[] = [];
  readonly statuses: CodingAgentStatus[];
  response: JsonObject;
  handler: (() => Promise<JsonObject>) | null = null;

  constructor(statuses: CodingAgentStatus[], response: JsonObject) {
    this.statuses = statuses;
    this.response = response;
  }

  listAgents(): CodingAgentStatus[] {
    return this.statuses;
  }

  getAgent(id: CodingAgentId): CodingAgentStatus | null {
    return this.statuses.find((agent) => agent.id === id) ?? null;
  }

  async draft(request: AgentDraftRequest): Promise<JsonObject> {
    this.calls.push(request);

    if (this.handler) {
      return this.handler();
    }

    return this.response;
  }
}

async function completeOnboarding(
  gateway: ScriptedAgentGateway
): Promise<TestHarness> {
  const created = await createTestHarness({ agents: gateway });
  await created.app.confirmOnboarding({
    displayName: "Sam",
    experienceStage: "early-career",
    newToCoding: false,
    skillsStatement: "Some JavaScript",
    learningGoals: ["Learn testing"],
  });

  return created;
}

async function importDocument(
  app: TestHarness["app"],
  name: string,
  kind: "cv" | "srs",
  projectId: string | null = null
): Promise<ImportedDocument> {
  const extracted = await app.extractDocument({
    path: join(fixtures, name),
    kind,
  });

  if (!extracted.ok) {
    throw new Error(extracted.error.message);
  }

  return app.confirmDocumentImport(extracted.document, {
    confirmed: true,
    projectId,
  });
}

const cvDraft = {
  kind: "cv",
  displayName: "Ada Lovelace",
  experienceStage: "early-career",
  newToCoding: false,
  skillsStatement: "Learning compiler design",
  learningGoals: ["Learn compiler design"],
  selfReportedSkills: [{ topic: "compiler design", rating: null }],
  education: [],
  employment: [],
  projects: [],
  certifications: [],
  links: [],
  sources: [
    { field: "displayName", reference: "# CV" },
    { field: "learningGoals", reference: "Learning: compiler design" },
  ],
  warnings: [],
};

describe("coding-agent CV drafts", () => {
  test("shows the exact sharing text and saves only reviewed fields with source references", async () => {
    const gateway = new ScriptedAgentGateway(
      [agentStatus("claude-code"), agentStatus("codex", "not-installed")],
      cvDraft
    );

    harness = await createTestHarness({ agents: gateway });
    const document = await importDocument(harness.app, "notes.md", "cv");

    const agents = await harness.app.listCodingAgents();
    expect(agents.map((agent) => agent.id)).toEqual(["claude-code", "codex"]);
    expect(agents[1]?.availability).toBe("not-installed");

    const preview = await harness.app.prepareCvDraft(
      document.id,
      "claude-code"
    );

    expect(gateway.calls).toHaveLength(0);
    expect(preview.agent.name).toBe("Claude Code");
    expect(preview.document.id).toBe(document.id);
    expect(preview.fits).toBe(true);
    expect(preview.text).toContain("Ada Lovelace");
    expect(preview.text).toContain("Learning: compiler design");
    expect(preview.text).toContain(document.hash);
    expect(preview.text).toContain("Return one JSON object only.");

    const unconfirmed = await harness.app
      .requestCvDraft(document.id, "claude-code", { confirmed: false })
      .catch((error: unknown) => error);

    expect(unconfirmed).toBeInstanceOf(ApplicationError);
    expect((unconfirmed as ApplicationError).code).toBe(
      "confirmation-required"
    );
    expect(gateway.calls).toHaveLength(0);

    const progress: string[] = [];

    const result = await harness.app.requestCvDraft(
      document.id,
      "claude-code",
      {
        confirmed: true,
        onProgress: (message) => progress.push(message),
      }
    );

    expect(progress.length).toBeGreaterThanOrEqual(2);
    expect(progress.at(-1)).toContain("Validating");
    expect(gateway.calls).toHaveLength(1);
    expect(gateway.calls[0]?.text).toBe(preview.text);
    expect(gateway.calls[0]?.agentId).toBe("claude-code");
    expect(result.draft.kind).toBe("cv");
    expect(await harness.app.getProfile()).toBeNull();

    const saved = await harness.app.applyCvDraft(document.id, result.draft, {
      confirmed: true,
    });

    expect(saved.displayName).toBe("Ada Lovelace");
    expect(saved.learningGoals).toEqual(["Learn compiler design"]);
    expect(saved.selfReportedSkills[0]?.rating).toBeNull();
    expect(saved.fieldSources.displayName).toMatchObject({
      kind: "document",
      documentId: document.id,
      documentName: document.name,
      reference: "# CV",
    });

    const persisted = await harness.app.getProfile();
    expect(persisted).toEqual(saved);
  });

  test("invalid output, missing requirements, unavailable agents, and unsupported sizes never save or fall back", async () => {
    const gateway = new ScriptedAgentGateway(
      [agentStatus("claude-code"), agentStatus("codex", "not-installed", 10)],
      { kind: "cv", displayName: 42 }
    );

    harness = await createTestHarness({ agents: gateway });
    const document = await importDocument(harness.app, "notes.md", "cv");

    const invalid = await harness.app
      .requestCvDraft(document.id, "claude-code", { confirmed: true })
      .catch((error: unknown) => error);

    expect(invalid).toBeInstanceOf(ApplicationError);
    expect((invalid as ApplicationError).code).toBe("agent-invalid-output");
    expect(await harness.app.getProfile()).toBeNull();

    gateway.response = {
      ...cvDraft,
      learningGoals: undefined,
    } as unknown as JsonObject;

    const missingGoal = await harness.app.requestCvDraft(
      document.id,
      "claude-code",
      { confirmed: true }
    );

    const blocked = await captureError(() =>
      harness!.app.applyCvDraft(document.id, missingGoal.draft, {
        confirmed: true,
      })
    );

    expect(blocked).toBeInstanceOf(ApplicationError);
    expect((blocked as ApplicationError).code).toBe("validation");
    expect((blocked as ApplicationError).message).toContain("learning goal");
    expect(await harness.app.getProfile()).toBeNull();

    const callsBeforeUnavailable = gateway.calls.length;

    const unavailable = await harness.app
      .requestCvDraft(document.id, "codex", { confirmed: true })
      .catch((error: unknown) => error);

    expect(unavailable).toBeInstanceOf(ApplicationError);
    expect((unavailable as ApplicationError).code).toBe("agent-unavailable");
    expect(gateway.calls).toHaveLength(callsBeforeUnavailable);

    const huge = agentStatus("claude-code", "ready", 2);
    gateway.statuses[0] = huge;

    const oversized = await harness.app
      .requestCvDraft(document.id, "claude-code", { confirmed: true })
      .catch((error: unknown) => error);

    expect(oversized).toBeInstanceOf(ApplicationError);
    expect((oversized as ApplicationError).code).toBe("agent-input-too-large");
    expect(gateway.calls).toHaveLength(callsBeforeUnavailable);

    const manual = await harness.app.confirmOnboarding({
      displayName: "Manual Entry",
      experienceStage: "new-to-coding",
      newToCoding: true,
      learningGoals: ["Learn by hand"],
    });

    expect(manual.displayName).toBe("Manual Entry");
  });

  test("a cancelled or failed agent run preserves the existing profile", async () => {
    const gateway = new ScriptedAgentGateway(
      [agentStatus("claude-code")],
      cvDraft
    );

    harness = await completeOnboarding(gateway);
    const document = await importDocument(harness.app, "notes.md", "cv");
    const before = await harness.app.getProfile();

    gateway.handler = async () => {
      throw new ApplicationError(
        "agent-cancelled",
        "The coding agent draft was cancelled."
      );
    };

    const cancelled = await harness.app
      .requestCvDraft(document.id, "claude-code", { confirmed: true })
      .catch((error: unknown) => error);

    expect(cancelled).toBeInstanceOf(ApplicationError);
    expect((cancelled as ApplicationError).code).toBe("agent-cancelled");
    expect(await harness.app.getProfile()).toEqual(before);
  });
});

describe("coding-agent SRS drafts", () => {
  test("applies reviewed project fields with document provenance and a new project version", async () => {
    const gateway = new ScriptedAgentGateway([agentStatus("claude-code")], {
      kind: "srs",
      purpose: "Coordinate a local context record.",
      businessLogic: "Capture only approved repository files.",
      goals: ["Capture context with explicit consent"],
      rules: ["Never read outside the selected scope"],
      constraints: ["Local SQLite only"],
      decisions: ["Use a versioned capture history"],
      acceptanceCriteria: ["A preview shows every captured path"],
      knowledge: ["The developer owns project knowledge"],
      sources: [
        { field: "purpose", reference: "Overview" },
        { field: "goals", reference: "Goals" },
        { field: "rules", reference: "Rules" },
      ],
      warnings: [],
    });

    harness = await completeOnboarding(gateway);

    const project = await harness.app.createProject({
      name: "Coderune",
      rootPath: join(workspace, "project"),
      goals: ["Initial goal"],
    });

    const document = await importDocument(
      harness.app,
      "notes.txt",
      "srs",
      project.id
    );

    const preview = await harness.app.prepareSrsDraft(
      document.id,
      "claude-code",
      project.id
    );

    expect(preview.project?.name).toBe("Coderune");
    expect(preview.text).toContain("Project: Coderune");

    const result = await harness.app.requestSrsDraft(
      document.id,
      "claude-code",
      project.id,
      { confirmed: true }
    );

    const unconfirmed = await captureError(() =>
      harness!.app.applySrsDraft(project.id, document.id, result.draft, {
        confirmed: false,
      })
    );

    expect(unconfirmed).toBeInstanceOf(ApplicationError);
    expect((unconfirmed as ApplicationError).code).toBe(
      "confirmation-required"
    );

    const saved = await harness.app.applySrsDraft(
      project.id,
      document.id,
      result.draft,
      { confirmed: true }
    );

    expect(saved.version).toBe(2);
    expect(saved.purpose?.text).toBe("Coordinate a local context record.");
    expect(saved.purpose?.provenance).toMatchObject({
      kind: "document",
      documentId: document.id,
      documentName: document.name,
      reference: "Overview",
    });
    expect(saved.goals[0]?.provenance.kind).toBe("document");
    expect(saved.rules[0]?.text).toBe("Never read outside the selected scope");
    expect(saved.acceptanceCriteria[0]?.provenance.kind).toBe("document");

    const persisted = await harness.app.getProject(project.id);
    expect(persisted).toEqual(saved);

    const edited = await harness.app.updateProject(project.id, {
      name: saved.name,
      rootPath: saved.rootPath,
      goals: saved.goals.map((goal) => goal.text),
      purpose: saved.purpose?.text,
      businessLogic: saved.businessLogic?.text,
      rules: saved.rules.map((rule) => rule.text),
      constraints: saved.constraints.map((constraint) => constraint.text),
      decisions: saved.decisions.map((decision) => decision.text),
      acceptanceCriteria: saved.acceptanceCriteria.map(
        (criterion) => criterion.text
      ),
      knowledge: saved.knowledge.map((item) => item.text),
      scope: saved.scope,
      limits: saved.limits,
    });

    expect(edited.purpose?.provenance.kind).toBe("document");
    expect(edited.goals[0]?.provenance.kind).toBe("document");
    expect(edited.rules[0]?.provenance.kind).toBe("document");
  });
});
