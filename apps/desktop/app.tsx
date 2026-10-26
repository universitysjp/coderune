import type {
  ApplicationOperations,
  CodingAgentId,
  CodingAgentStatus,
  ContextCapture,
  DeveloperProfile,
  DeveloperProfileInput,
  DocumentExtractionResult,
  ExperienceStage,
  ImportedDocument,
  Project,
  ProjectInput,
  CapturePreview,
} from "@coderune/core";
import { resolveApplicationDataDir } from "@coderune/storage";
import { Button, isTerminal, render } from "@gpuix/react";
import { useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";

import { createApplication } from "./src/application";
import type { ApplicationRuntime } from "./src/application";

const colors = {
  bg: "#f4f2fa",
  panel: "#ffffff",
  ink: "#1d1a24",
  muted: "#6e6878",
  line: "#ddd8e6",
  accent: "#5b3df5",
  accentSoft: "#ede9ff",
  danger: "#b42318",
  dangerSoft: "#fee4e2",
  success: "#17663a",
  successSoft: "#e7f6ec",
  code: "#f7f6fb",
};

const terminalWarning =
  isTerminal() && (process.platform === "win32" || Boolean(process.env.TMUX))
    ? "This terminal combination is not supported for GPUIX terminal mode. Use native desktop mode, or a compatible macOS/Linux terminal without tmux."
    : "";

const buttonBase = {
  paddingTop: 7,
  paddingBottom: 7,
  paddingLeft: 12,
  paddingRight: 12,
  borderRadius: 7,
  borderWidth: 1,
  borderColor: colors.line,
  backgroundColor: colors.panel,
  cursor: "pointer",
} as const;

const inputBase = {
  height: 32,
  paddingLeft: 9,
  paddingRight: 9,
  borderWidth: 1,
  borderColor: colors.line,
  borderRadius: 6,
  backgroundColor: "#ffffff",
  fontSize: 13,
  color: colors.ink,
} as const;

function lines(value: string): string[] {
  return value
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
}

function errorText(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

function stageLabel(stage: ExperienceStage): string {
  return stage.replaceAll("-", " ");
}

const Section = ({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children: ReactNode;
}) => {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: 12,
        padding: 18,
        borderRadius: 10,
        borderWidth: 1,
        borderColor: colors.line,
        backgroundColor: colors.panel,
      }}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
        <text style={{ fontSize: 18, fontWeight: 650, color: colors.ink }}>
          {title}
        </text>
        {description ? (
          <text
            style={{ fontSize: 12, color: colors.muted, whiteSpace: "normal" }}
          >
            {description}
          </text>
        ) : null}
      </div>
      {children}
    </div>
  );
};

const Labeled = ({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) => {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
      <text style={{ fontSize: 12, fontWeight: 600, color: colors.ink }}>
        {label}
      </text>
      {hint ? (
        <text style={{ fontSize: 11, color: colors.muted }}>{hint}</text>
      ) : null}
      {children}
    </div>
  );
};

const Notice = ({ error, message }: { error: string; message: string }) => {
  if (!error && !message) {
    return null;
  }

  return (
    <div
      style={{
        padding: 10,
        borderRadius: 7,
        backgroundColor: error ? colors.dangerSoft : colors.successSoft,
      }}
    >
      <text
        style={{
          fontSize: 12,
          color: error ? colors.danger : colors.success,
          whiteSpace: "normal",
        }}
      >
        {error || message}
      </text>
    </div>
  );
};

const ProfileEditor = ({
  initial,
  submitLabel,
  onSubmit,
  onDraft,
}: {
  initial: Partial<DeveloperProfile> | null;
  submitLabel: string;
  onSubmit: (input: DeveloperProfileInput) => Promise<void>;
  onDraft?: (input: DeveloperProfileInput) => Promise<void>;
}) => {
  const [displayName, setDisplayName] = useState(initial?.displayName ?? "");

  const [stage, setStage] = useState<ExperienceStage>(
    initial?.experienceStage ?? "new-to-coding"
  );

  const [newToCoding, setNewToCoding] = useState(initial?.newToCoding ?? true);
  const [skills, setSkills] = useState(initial?.skillsStatement ?? "");
  const [goals, setGoals] = useState((initial?.learningGoals ?? []).join("\n"));

  const [details, setDetails] = useState(
    JSON.stringify(
      {
        selfReportedSkills: initial?.selfReportedSkills ?? [],
        education: initial?.education ?? [],
        employment: initial?.employment ?? [],
        projects: initial?.projects ?? [],
        certifications: initial?.certifications ?? [],
        links: initial?.links ?? [],
        learningPreferences: initial?.learningPreferences ?? null,
      },
      null,
      2
    )
  );

  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const draftHandler = useRef(onDraft);

  draftHandler.current = onDraft;

  const buildInput = (): DeveloperProfileInput => {
    const extra = JSON.parse(details) as Partial<DeveloperProfileInput>;

    return {
      displayName,
      experienceStage: stage,
      newToCoding,
      skillsStatement: skills,
      learningGoals: lines(goals),
      selfReportedSkills: extra.selfReportedSkills ?? [],
      education: extra.education ?? [],
      employment: extra.employment ?? [],
      projects: extra.projects ?? [],
      certifications: extra.certifications ?? [],
      links: extra.links ?? [],
      learningPreferences: extra.learningPreferences ?? null,
      fieldSources: initial?.fieldSources ?? {},
    };
  };

  useEffect(() => {
    const saveDraft = draftHandler.current;

    if (!saveDraft) {
      return;
    }

    const timer = setTimeout(() => {
      try {
        const input = buildInput();
        saveDraft(input).catch(() => undefined);
      } catch {
        // Keep typing while optional JSON is temporarily incomplete.
      }
    }, 700);

    return () => clearTimeout(timer);
  }, [displayName, stage, newToCoding, skills, goals, details]);

  const run = async (
    action: (input: DeveloperProfileInput) => Promise<void>
  ) => {
    setBusy(true);
    setError("");

    try {
      await action(buildInput());
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <Notice error={error} message="" />
      <div style={{ display: "flex", gap: 12 }}>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Display name or pseudonym">
            <input
              value={displayName}
              onChange={(event) => setDisplayName(event.value ?? "")}
              placeholder="Ada"
              style={inputBase}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Experience stage">
            <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
              {(
                [
                  "new-to-coding",
                  "early-career",
                  "mid-career",
                  "experienced",
                ] as ExperienceStage[]
              ).map((option) => (
                <Button
                  key={option}
                  onClick={() => setStage(option)}
                  style={{
                    ...buttonBase,
                    backgroundColor:
                      stage === option ? colors.accentSoft : colors.panel,
                    borderColor: stage === option ? colors.accent : colors.line,
                  }}
                >
                  <text style={{ fontSize: 12 }}>{stageLabel(option)}</text>
                </Button>
              ))}
            </div>
          </Labeled>
        </div>
      </div>

      <Button
        onClick={() => setNewToCoding((value) => !value)}
        style={{
          ...buttonBase,
          alignSelf: "flex-start",
          backgroundColor: newToCoding ? colors.successSoft : colors.panel,
        }}
      >
        <text style={{ fontSize: 12 }}>
          {newToCoding
            ? "New to coding; no skills listed yet"
            : "I have an initial skills statement"}
        </text>
      </Button>

      {newToCoding ? null : (
        <Labeled label="Initial skills statement">
          <textarea
            value={skills}
            onChange={(event) => setSkills(event.value ?? "")}
            minRows={2}
            style={{ ...inputBase, height: 64, paddingTop: 7 }}
          />
        </Labeled>
      )}

      <Labeled
        label="Learning goals"
        hint="One goal per line. At least one is required."
      >
        <textarea
          value={goals}
          onChange={(event) => setGoals(event.value ?? "")}
          minRows={3}
          style={{ ...inputBase, height: 72, paddingTop: 7 }}
        />
      </Labeled>

      <Labeled
        label="Optional profile detail (JSON)"
        hint="Education, employment, projects, certifications, links, skills, and preferences. Missing fields are allowed."
      >
        <textarea
          value={details}
          onChange={(event) => setDetails(event.value ?? "")}
          minRows={8}
          style={{
            ...inputBase,
            height: 170,
            paddingTop: 7,
            fontFamily: "monospace",
          }}
        />
      </Labeled>

      <div style={{ display: "flex", gap: 8 }}>
        {onDraft ? (
          <Button
            onClick={() => void run((input) => onDraft(input))}
            disabled={busy}
            style={{ ...buttonBase, opacity: busy ? 0.5 : 1 }}
          >
            <text style={{ fontSize: 12 }}>Save draft</text>
          </Button>
        ) : null}
        <Button
          onClick={() => void run(onSubmit)}
          disabled={busy}
          style={{
            ...buttonBase,
            backgroundColor: colors.accent,
            borderColor: colors.accent,
            opacity: busy ? 0.5 : 1,
          }}
        >
          <text style={{ fontSize: 12, color: "#ffffff", fontWeight: 600 }}>
            {busy ? "Saving..." : submitLabel}
          </text>
        </Button>
      </div>
    </div>
  );
};

const ProjectEditor = ({
  project,
  onCreate,
  onUpdate,
}: {
  project: Project | null;
  onCreate: (input: ProjectInput) => Promise<void>;
  onUpdate: (id: string, input: ProjectInput) => Promise<void>;
}) => {
  const [name, setName] = useState(project?.name ?? "");
  const [rootPath, setRootPath] = useState(project?.rootPath ?? "");

  const [goals, setGoals] = useState(
    (project?.goals ?? []).map((item) => item.text).join("\n")
  );

  const [purpose, setPurpose] = useState(project?.purpose?.text ?? "");

  const [businessLogic, setBusinessLogic] = useState(
    project?.businessLogic?.text ?? ""
  );

  const [rules, setRules] = useState(
    (project?.rules ?? []).map((item) => item.text).join("\n")
  );

  const [constraints, setConstraints] = useState(
    (project?.constraints ?? []).map((item) => item.text).join("\n")
  );

  const [decisions, setDecisions] = useState(
    (project?.decisions ?? []).map((item) => item.text).join("\n")
  );

  const [acceptance, setAcceptance] = useState(
    (project?.acceptanceCriteria ?? []).map((item) => item.text).join("\n")
  );

  const [knowledge, setKnowledge] = useState(
    (project?.knowledge ?? []).map((item) => item.text).join("\n")
  );

  const [includePaths, setIncludePaths] = useState(
    (project?.scope.includePaths ?? []).join("\n")
  );

  const [excludePaths, setExcludePaths] = useState(
    (project?.scope.excludePaths ?? []).join("\n")
  );

  const [respectGitignore, setRespectGitignore] = useState(
    project?.scope.respectGitignore ?? true
  );

  const [maxFileMiB, setMaxFileMiB] = useState(
    String((project?.limits.maxFileBytes ?? 1_048_576) / 1_048_576)
  );

  const [maxCaptureMiB, setMaxCaptureMiB] = useState(
    String((project?.limits.maxCaptureBytes ?? 20_971_520) / 1_048_576)
  );

  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  const input = (): ProjectInput => ({
    name,
    rootPath,
    goals: lines(goals),
    purpose,
    businessLogic,
    rules: lines(rules),
    constraints: lines(constraints),
    decisions: lines(decisions),
    acceptanceCriteria: lines(acceptance),
    knowledge: lines(knowledge),
    scope: {
      includePaths: lines(includePaths),
      excludePaths: lines(excludePaths),
      respectGitignore,
    },
    limits: {
      maxFileBytes: Math.max(1, Number(maxFileMiB)) * 1_048_576,
      maxCaptureBytes: Math.max(1, Number(maxCaptureMiB)) * 1_048_576,
    },
  });

  const submit = async () => {
    setBusy(true);
    setError("");

    try {
      if (project) {
        await onUpdate(project.id, input());
      } else {
        await onCreate(input());
      }
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      <Notice error={error} message="" />
      <div style={{ display: "flex", gap: 10 }}>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Project name">
            <input
              value={name}
              onChange={(event) => setName(event.value ?? "")}
              style={inputBase}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 2 }}>
          <Labeled label="Readable root folder">
            <input
              value={rootPath}
              onChange={(event) => setRootPath(event.value ?? "")}
              placeholder="/Users/me/project"
              style={inputBase}
            />
          </Labeled>
        </div>
      </div>
      <Labeled label="Goals" hint="One per line. At least one is required.">
        <textarea
          value={goals}
          onChange={(event) => setGoals(event.value ?? "")}
          minRows={2}
          style={{ ...inputBase, height: 56, paddingTop: 6 }}
        />
      </Labeled>
      <div style={{ display: "flex", gap: 10 }}>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Purpose">
            <textarea
              value={purpose}
              onChange={(event) => setPurpose(event.value ?? "")}
              minRows={2}
              style={{ ...inputBase, height: 58, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Business logic">
            <textarea
              value={businessLogic}
              onChange={(event) => setBusinessLogic(event.value ?? "")}
              minRows={2}
              style={{ ...inputBase, height: 58, paddingTop: 6 }}
            />
          </Labeled>
        </div>
      </div>
      <div style={{ display: "flex", gap: 10 }}>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Rules">
            <textarea
              value={rules}
              onChange={(event) => setRules(event.value ?? "")}
              style={{ ...inputBase, height: 70, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Constraints">
            <textarea
              value={constraints}
              onChange={(event) => setConstraints(event.value ?? "")}
              style={{ ...inputBase, height: 70, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Decisions">
            <textarea
              value={decisions}
              onChange={(event) => setDecisions(event.value ?? "")}
              style={{ ...inputBase, height: 70, paddingTop: 6 }}
            />
          </Labeled>
        </div>
      </div>
      <div style={{ display: "flex", gap: 10 }}>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Acceptance criteria">
            <textarea
              value={acceptance}
              onChange={(event) => setAcceptance(event.value ?? "")}
              style={{ ...inputBase, height: 70, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Other knowledge">
            <textarea
              value={knowledge}
              onChange={(event) => setKnowledge(event.value ?? "")}
              style={{ ...inputBase, height: 70, paddingTop: 6 }}
            />
          </Labeled>
        </div>
      </div>
      <div style={{ display: "flex", gap: 10 }}>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Include paths" hint="Optional. One per line.">
            <textarea
              value={includePaths}
              onChange={(event) => setIncludePaths(event.value ?? "")}
              style={{ ...inputBase, height: 54, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Exclude paths" hint="Optional. One per line.">
            <textarea
              value={excludePaths}
              onChange={(event) => setExcludePaths(event.value ?? "")}
              style={{ ...inputBase, height: 54, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Capture limits" hint="MiB per file and per capture.">
            <div style={{ display: "flex", gap: 6 }}>
              <input
                value={maxFileMiB}
                onChange={(event) => setMaxFileMiB(event.value ?? "")}
                style={{ ...inputBase, flexGrow: 1 }}
              />
              <input
                value={maxCaptureMiB}
                onChange={(event) => setMaxCaptureMiB(event.value ?? "")}
                style={{ ...inputBase, flexGrow: 1 }}
              />
            </div>
          </Labeled>
        </div>
      </div>
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <Button
          onClick={() => setRespectGitignore((value) => !value)}
          style={{ ...buttonBase }}
        >
          <text style={{ fontSize: 12 }}>
            {respectGitignore ? "Respect .gitignore" : "Ignore .gitignore"}
          </text>
        </Button>
        <Button
          onClick={() => void submit()}
          disabled={busy}
          style={{
            ...buttonBase,
            backgroundColor: colors.accent,
            borderColor: colors.accent,
            opacity: busy ? 0.5 : 1,
          }}
        >
          <text style={{ fontSize: 12, color: "#ffffff", fontWeight: 600 }}>
            {busy
              ? "Saving..."
              : project
                ? "Save project version"
                : "Create project"}
          </text>
        </Button>
      </div>
    </div>
  );
};

const AgentDraftPanel = ({
  app,
  kind,
  projectId,
  documents,
  agents,
  onDocumentsChanged,
  onSaved,
}: {
  app: ApplicationOperations;
  kind: "cv" | "srs";
  projectId: string | null;
  documents: ImportedDocument[];
  agents: CodingAgentStatus[];
  onDocumentsChanged: () => Promise<void>;
  onSaved: () => Promise<void>;
}) => {
  const availableDocs = documents.filter((document) => document.kind === kind);
  const [documentPath, setDocumentPath] = useState("");
  const [maxDocumentMiB, setMaxDocumentMiB] = useState("20");

  const [extraction, setExtraction] = useState<DocumentExtractionResult | null>(
    null
  );

  const [selectedDocumentId, setSelectedDocumentId] = useState(
    availableDocs[0]?.id ?? ""
  );

  const [agentId, setAgentId] = useState<CodingAgentId | "">(
    agents.find(
      (agent) => agent.availability === "ready" && agent.id !== "codex"
    )?.id ??
      agents.find((agent) => agent.availability === "ready")?.id ??
      ""
  );

  const [preview, setPreview] = useState<Awaited<
    ReturnType<ApplicationOperations["prepareCvDraft"]>
  > | null>(null);

  const [shareConfirmed, setShareConfirmed] = useState(false);
  const [draftJson, setDraftJson] = useState("");
  const [draftAgent, setDraftAgent] = useState("");
  const [applyConfirmed, setApplyConfirmed] = useState(false);
  const [controller, setController] = useState<AbortController | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");

  const selected = availableDocs.find(
    (document) => document.id === selectedDocumentId
  );

  const clear = () => {
    setError("");
    setMessage("");
  };

  const importDocument = async () => {
    setBusy(true);
    clear();

    try {
      const result = await app.extractDocument({
        path: documentPath,
        kind,
        maxBytes:
          Math.max(1, Number(maxDocumentMiB) || 20) * 1_048_576,
      });
      setExtraction(result);

      if (!result.ok) {
        setError(
          `${result.error.message} You can keep entering the ${kind.toUpperCase()} manually.`
        );
      }
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  const confirmDocument = async () => {
    if (!extraction?.ok) {
      return;
    }

    setBusy(true);
    clear();

    try {
      const imported = await app.confirmDocumentImport(extraction.document, {
        confirmed: true,
        projectId: kind === "srs" ? projectId : null,
      });

      setSelectedDocumentId(imported.id);
      setExtraction(null);
      setDocumentPath("");
      await onDocumentsChanged();
      setMessage("Document text saved locally after review.");
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  const prepare = async () => {
    if (!selected || !agentId) {
      setError("Select an imported document and a coding agent.");

      return;
    }

    if (kind === "srs" && !projectId) {
      setError("Select a project before preparing an SRS draft.");

      return;
    }

    setBusy(true);
    clear();

    try {
      const next =
        kind === "cv"
          ? await app.prepareCvDraft(selected.id, agentId)
          : await app.prepareSrsDraft(selected.id, agentId, projectId!);

      setPreview(next);
      setShareConfirmed(false);
      setDraftJson("");
      setApplyConfirmed(false);
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  const requestDraft = async () => {
    if (!selected || !preview) {
      return;
    }

    const nextController = new AbortController();
    setController(nextController);
    setBusy(true);
    clear();

    try {
      setMessage("Waiting for the selected coding agent...");

      const result =
        kind === "cv"
          ? await app.requestCvDraft(selected.id, preview.agent.id, {
              confirmed: true,
              signal: nextController.signal,
              onProgress: (text) => setMessage(text),
            })
          : await app.requestSrsDraft(
              selected.id,
              preview.agent.id,
              projectId!,
              {
                confirmed: true,
                signal: nextController.signal,
                onProgress: (text) => setMessage(text),
              }
            );

      setDraftJson(JSON.stringify(result.draft, null, 2));
      setDraftAgent(result.agent.name);
      setMessage(
        `${result.agent.name} returned a schema-valid draft. Review and correct every field below.`
      );
    } catch (error) {
      setError(errorText(error));
    } finally {
      setController(null);
      setBusy(false);
    }
  };

  const applyDraft = async () => {
    if (!selected) {
      return;
    }

    setBusy(true);
    clear();

    try {
      const parsed = JSON.parse(draftJson) as unknown;

      if (kind === "cv") {
        await app.applyCvDraft(selected.id, parsed, { confirmed: true });
      } else {
        await app.applySrsDraft(projectId!, selected.id, parsed, {
          confirmed: true,
        });
      }

      setMessage("Reviewed draft saved.");
      setPreview(null);
      setDraftJson("");
      setApplyConfirmed(false);
      await onSaved();
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <Notice error={error} message={message} />
      <Labeled
        label={`Import a local ${kind.toUpperCase()}`}
        hint="PDF, DOCX, Markdown, or plain text. Text is extracted locally; the original file is not copied. The number is the per-document MiB limit."
      >
        <div style={{ display: "flex", gap: 6 }}>
          <input
            value={documentPath}
            onChange={(event) => setDocumentPath(event.value ?? "")}
            placeholder={`/Users/me/${kind}.pdf`}
            style={{ ...inputBase, flexGrow: 1 }}
          />
          <input
            value={maxDocumentMiB}
            onChange={(event) => setMaxDocumentMiB(event.value ?? "")}
            style={{ ...inputBase, width: 54 }}
          />
          <Button
            onClick={() => void importDocument()}
            disabled={busy || !documentPath}
            style={{ ...buttonBase, opacity: busy ? 0.5 : 1 }}
          >
            <text style={{ fontSize: 12 }}>Extract</text>
          </Button>
        </div>
      </Labeled>

      {extraction?.ok ? (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <div
            style={{
              maxHeight: 130,
              overflow: "scroll",
              padding: 9,
              borderRadius: 6,
              backgroundColor: colors.code,
            }}
          >
            <text
              style={{
                fontSize: 11,
                color: colors.muted,
                whiteSpace: "normal",
              }}
            >
              {extraction.document.text}
            </text>
          </div>
          <Button
            onClick={() => void confirmDocument()}
            disabled={busy}
            style={{ ...buttonBase, alignSelf: "flex-start" }}
          >
            <text style={{ fontSize: 12 }}>Save reviewed document text</text>
          </Button>
        </div>
      ) : null}

      <Labeled label="Imported document">
        <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
          {availableDocs.length === 0 ? (
            <text style={{ fontSize: 12, color: colors.muted }}>
              No imported {kind.toUpperCase()} yet.
            </text>
          ) : (
            availableDocs.map((document) => (
              <Button
                key={document.id}
                onClick={() => {
                  setSelectedDocumentId(document.id);
                  setPreview(null);
                  clear();
                }}
                style={{
                  ...buttonBase,
                  backgroundColor:
                    selectedDocumentId === document.id
                      ? colors.accentSoft
                      : colors.panel,
                  borderColor:
                    selectedDocumentId === document.id
                      ? colors.accent
                      : colors.line,
                }}
              >
                <text style={{ fontSize: 12 }}>{document.name}</text>
              </Button>
            ))
          )}
        </div>
      </Labeled>

      <Labeled
        label="Coding agent"
        hint="Coderune never switches agents or providers after a failure."
      >
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {agents.map((agent) => (
            <Button
              key={agent.id}
              onClick={() => {
                setAgentId(agent.id);
                setPreview(null);
                clear();
              }}
              disabled={agent.availability !== "ready"}
              style={{
                ...buttonBase,
                textAlign: "left",
                opacity: agent.availability === "ready" ? 1 : 0.55,
                backgroundColor:
                  agentId === agent.id ? colors.accentSoft : colors.panel,
                borderColor: agentId === agent.id ? colors.accent : colors.line,
              }}
            >
              <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
                <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                  <text style={{ fontSize: 12, fontWeight: 600 }}>
                    {agent.name}
                  </text>
                  <text style={{ fontSize: 10, color: colors.muted }}>
                    {agent.availability === "ready"
                      ? "installed"
                      : "not installed"}
                    {agent.verified
                      ? " / verified"
                      : " / compatibility not yet demonstrated"}
                  </text>
                </div>
                <text
                  style={{
                    fontSize: 10,
                    color: colors.muted,
                    whiteSpace: "normal",
                  }}
                >
                  {agent.detail}
                </text>
                <text
                  style={{
                    fontSize: 10,
                    color: colors.muted,
                    whiteSpace: "normal",
                  }}
                >
                  {agent.capabilities.restrictions.join(" ")}
                </text>
              </div>
            </Button>
          ))}
        </div>
      </Labeled>

      <Button
        onClick={() => void prepare()}
        disabled={busy || !selected || !agentId}
        style={{
          ...buttonBase,
          alignSelf: "flex-start",
          opacity: busy || !selected || !agentId ? 0.5 : 1,
        }}
      >
        <text style={{ fontSize: 12 }}>Prepare exact sharing preview</text>
      </Button>

      {preview ? (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 8,
            padding: 12,
            borderRadius: 8,
            borderWidth: 1,
            borderColor: colors.accent,
            backgroundColor: colors.accentSoft,
          }}
        >
          <text style={{ fontSize: 12, fontWeight: 600, color: colors.ink }}>
            Sharing with {preview.agent.name}
          </text>
          <text style={{ fontSize: 11, color: colors.muted }}>
            {preview.estimatedBytes} bytes
            {preview.maxInputBytes === null
              ? " / no reported input limit"
              : ` / ${preview.maxInputBytes} byte limit`}
            {preview.fits ? " / fits" : " / too large"}
          </text>
          <div
            style={{
              maxHeight: 220,
              overflow: "scroll",
              padding: 9,
              borderRadius: 6,
              backgroundColor: colors.panel,
            }}
          >
            <text
              style={{ fontSize: 11, color: colors.ink, whiteSpace: "normal" }}
            >
              {preview.text}
            </text>
          </div>
          <Button
            onClick={() => setShareConfirmed((value) => !value)}
            style={{
              ...buttonBase,
              backgroundColor: shareConfirmed
                ? colors.successSoft
                : colors.panel,
            }}
          >
            <text style={{ fontSize: 12 }}>
              {shareConfirmed
                ? "I reviewed the exact text to share"
                : "Confirm the exact text before sending"}
            </text>
          </Button>
          <div style={{ display: "flex", gap: 8 }}>
            <Button
              onClick={() => void requestDraft()}
              disabled={!shareConfirmed || busy || !preview.fits}
              style={{
                ...buttonBase,
                backgroundColor: colors.accent,
                borderColor: colors.accent,
                opacity: !shareConfirmed || busy || !preview.fits ? 0.5 : 1,
              }}
            >
              <text style={{ fontSize: 12, color: "#ffffff", fontWeight: 600 }}>
                {busy ? "Drafting..." : "Request reviewed draft"}
              </text>
            </Button>
            {controller ? (
              <Button
                onClick={() => controller.abort()}
                style={{ ...buttonBase, borderColor: colors.danger }}
              >
                <text style={{ fontSize: 12, color: colors.danger }}>
                  Cancel
                </text>
              </Button>
            ) : null}
          </div>
        </div>
      ) : null}

      {draftJson ? (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <Labeled
            label={`Review and correct the ${kind.toUpperCase()} draft`}
            hint={`Returned by ${draftAgent}. Schema-valid output is not treated as fact. Edit any field before saving.`}
          >
            <textarea
              value={draftJson}
              onChange={(event) => setDraftJson(event.value ?? "")}
              minRows={14}
              style={{
                ...inputBase,
                height: 300,
                paddingTop: 7,
                fontFamily: "monospace",
              }}
            />
          </Labeled>
          <Button
            onClick={() => setApplyConfirmed((value) => !value)}
            style={{
              ...buttonBase,
              alignSelf: "flex-start",
              backgroundColor: applyConfirmed
                ? colors.successSoft
                : colors.panel,
            }}
          >
            <text style={{ fontSize: 12 }}>
              {applyConfirmed
                ? "I reviewed every field in this draft"
                : "Confirm review before saving"}
            </text>
          </Button>
          <Button
            onClick={() => void applyDraft()}
            disabled={!applyConfirmed || busy}
            style={{
              ...buttonBase,
              alignSelf: "flex-start",
              backgroundColor: colors.accent,
              borderColor: colors.accent,
              opacity: !applyConfirmed || busy ? 0.5 : 1,
            }}
          >
            <text style={{ fontSize: 12, color: "#ffffff", fontWeight: 600 }}>
              Save reviewed {kind.toUpperCase()}
            </text>
          </Button>
        </div>
      ) : null}
    </div>
  );
};

const CapturePanel = ({
  app,
  project,
  onChanged,
}: {
  app: ApplicationOperations;
  project: Project;
  onChanged: () => Promise<void>;
}) => {
  const [preview, setPreview] = useState<CapturePreview | null>(null);
  const [captures, setCaptures] = useState<ContextCapture[]>([]);
  const [viewCapture, setViewCapture] = useState<ContextCapture | null>(null);
  const [viewFile, setViewFile] = useState<string | null>(null);

  const [includePaths, setIncludePaths] = useState(
    project.scope.includePaths.join("\n")
  );

  const [excludePaths, setExcludePaths] = useState(
    project.scope.excludePaths.join("\n")
  );

  const [respectGitignore, setRespectGitignore] = useState(
    project.scope.respectGitignore
  );

  const [maxFileMiB, setMaxFileMiB] = useState(
    String(project.limits.maxFileBytes / 1_048_576)
  );

  const [maxCaptureMiB, setMaxCaptureMiB] = useState(
    String(project.limits.maxCaptureBytes / 1_048_576)
  );

  const [confirmed, setConfirmed] = useState(false);
  const [deleteId, setDeleteId] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");

  const loadCaptures = async () => {
    setCaptures(await app.listCaptures(project.id));
  };

  const inspectCapture = async (captureId: string) => {
    setViewFile(null);
    setViewCapture(await app.getCapture(captureId));
  };

  useEffect(() => {
    void loadCaptures();
  }, [project.id, project.version]);

  const runPreview = async () => {
    setBusy(true);
    setError("");
    setMessage("");

    try {
      const next = await app.previewCapture(project.id, {
        scope: {
          includePaths: lines(includePaths),
          excludePaths: lines(excludePaths),
          respectGitignore,
        },
        limits: {
          maxFileBytes: Math.max(1, Number(maxFileMiB)) * 1_048_576,
          maxCaptureBytes: Math.max(1, Number(maxCaptureMiB)) * 1_048_576,
        },
      });

      setPreview(next);
      setConfirmed(false);
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  const save = async () => {
    if (!preview) {
      return;
    }

    setBusy(true);
    setError("");
    setMessage("");

    try {
      const capture = await app.saveCapture(preview.id, { confirmed: true });
      setMessage(`Saved capture version ${capture.version}.`);
      setPreview(null);
      setConfirmed(false);
      await loadCaptures();
      await onChanged();
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  const refresh = async () => {
    setBusy(true);
    setError("");
    setMessage("");

    try {
      const result = await app.refreshProjectContext(project.id);
      setPreview(result.preview);
      setConfirmed(false);

      if (result.status === "unchanged") {
        setMessage("Context is unchanged. No duplicate capture was added.");
      } else {
        setMessage("Context changed. Review the fresh preview before saving.");
      }

      await loadCaptures();
    } catch (error) {
      setError(errorText(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <Notice error={error} message={message} />
      <div style={{ display: "flex", gap: 10 }}>
        <div style={{ flexGrow: 1 }}>
          <Labeled
            label="Include paths"
            hint="Optional. Empty means the whole repository."
          >
            <textarea
              value={includePaths}
              onChange={(event) => setIncludePaths(event.value ?? "")}
              style={{ ...inputBase, height: 54, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div style={{ flexGrow: 1 }}>
          <Labeled label="Exclude paths" hint="One per line.">
            <textarea
              value={excludePaths}
              onChange={(event) => setExcludePaths(event.value ?? "")}
              style={{ ...inputBase, height: 54, paddingTop: 6 }}
            />
          </Labeled>
        </div>
        <div>
          <Labeled label="Limits" hint="MiB per file / capture.">
            <div style={{ display: "flex", gap: 6 }}>
              <input
                value={maxFileMiB}
                onChange={(event) => setMaxFileMiB(event.value ?? "")}
                style={{ ...inputBase, width: 58 }}
              />
              <input
                value={maxCaptureMiB}
                onChange={(event) => setMaxCaptureMiB(event.value ?? "")}
                style={{ ...inputBase, width: 58 }}
              />
            </div>
          </Labeled>
        </div>
      </div>
      <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
        <Button
          onClick={() => setRespectGitignore((value) => !value)}
          style={{ ...buttonBase }}
        >
          <text style={{ fontSize: 12 }}>
            {respectGitignore ? "Respect .gitignore" : "Ignore .gitignore"}
          </text>
        </Button>
        <Button
          onClick={() => void runPreview()}
          disabled={busy}
          style={{ ...buttonBase, opacity: busy ? 0.5 : 1 }}
        >
          <text style={{ fontSize: 12 }}>Preview capture</text>
        </Button>
        <Button
          onClick={() => void refresh()}
          disabled={busy}
          style={{ ...buttonBase, opacity: busy ? 0.5 : 1 }}
        >
          <text style={{ fontSize: 12 }}>Refresh context</text>
        </Button>
      </div>

      {preview ? (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <div style={{ display: "flex", gap: 12 }}>
            <text style={{ fontSize: 12, fontWeight: 600 }}>
              {preview.files.length} files / {preview.totalBytes} bytes
            </text>
            <text style={{ fontSize: 12, color: colors.muted }}>
              {preview.skipped.length} skipped
            </text>
            <text style={{ fontSize: 12, color: colors.muted }}>
              Git:{" "}
              {preview.git
                ? `${preview.git.branch ?? "detached"}${preview.git.dirty ? " (dirty)" : " (clean)"}`
                : "not available"}
            </text>
          </div>
          <div
            style={{
              maxHeight: 220,
              overflow: "scroll",
              padding: 9,
              borderRadius: 6,
              backgroundColor: colors.code,
            }}
          >
            {preview.files.map((file) => (
              <text
                key={file.path}
                style={{
                  fontSize: 11,
                  color: colors.ink,
                  whiteSpace: "normal",
                }}
              >
                {file.category} / {file.path} / {file.size} bytes
              </text>
            ))}
          </div>
          {preview.skipped.length > 0 ? (
            <div
              style={{
                maxHeight: 100,
                overflow: "scroll",
                padding: 9,
                borderRadius: 6,
                backgroundColor: colors.panel,
                borderWidth: 1,
                borderColor: colors.line,
              }}
            >
              {preview.skipped.map((item) => (
                <text
                  key={`${item.path}:${item.reason}`}
                  style={{
                    fontSize: 10,
                    color: colors.muted,
                    whiteSpace: "normal",
                  }}
                >
                  skipped {item.path}: {item.reason}
                </text>
              ))}
            </div>
          ) : null}
          <Button
            onClick={() => setConfirmed((value) => !value)}
            style={{
              ...buttonBase,
              alignSelf: "flex-start",
              backgroundColor: confirmed ? colors.successSoft : colors.panel,
            }}
          >
            <text style={{ fontSize: 12 }}>
              {confirmed
                ? "I reviewed this exact capture"
                : "Confirm reviewed capture before saving"}
            </text>
          </Button>
          <Button
            onClick={() => void save()}
            disabled={!confirmed || busy}
            style={{
              ...buttonBase,
              alignSelf: "flex-start",
              backgroundColor: colors.accent,
              borderColor: colors.accent,
              opacity: !confirmed || busy ? 0.5 : 1,
            }}
          >
            <text style={{ fontSize: 12, color: "#ffffff", fontWeight: 600 }}>
              Save reviewed capture
            </text>
          </Button>
        </div>
      ) : null}

      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        <text style={{ fontSize: 13, fontWeight: 600 }}>Capture history</text>
        {captures.length === 0 ? (
          <text style={{ fontSize: 12, color: colors.muted }}>
            No captures yet.
          </text>
        ) : (
          captures.map((capture) => (
            <div
              key={capture.id}
              style={{
                display: "flex",
                gap: 8,
                alignItems: "center",
                padding: 8,
                borderRadius: 6,
                borderWidth: 1,
                borderColor: colors.line,
              }}
            >
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 2,
                  flexGrow: 1,
                }}
              >
                <text style={{ fontSize: 12, fontWeight: 600 }}>
                  Version {capture.version}
                </text>
                <text style={{ fontSize: 10, color: colors.muted }}>
                  {capture.createdAt} / {capture.files.length} files /{" "}
                  {capture.totalBytes} bytes / profile v
                  {capture.profileSnapshot.version} / project v
                  {capture.projectSnapshot.version}
                </text>
              </div>
              <Button
                onClick={() => void inspectCapture(capture.id)}
                style={buttonBase}
              >
                <text style={{ fontSize: 11 }}>View</text>
              </Button>
              <Button
                onClick={() => setDeleteId(capture.id)}
                style={{ ...buttonBase, borderColor: colors.danger }}
              >
                <text style={{ fontSize: 11, color: colors.danger }}>
                  Delete
                </text>
              </Button>
              {deleteId === capture.id ? (
                <Button
                  onClick={() => {
                    void (async () => {
                      setBusy(true);

                      try {
                        await app.deleteCapture(capture.id);
                        setDeleteId("");
                        setMessage("Capture deleted.");
                        await loadCaptures();
                      } catch (error) {
                        setError(errorText(error));
                      } finally {
                        setBusy(false);
                      }
                    })();
                  }}
                  style={{
                    ...buttonBase,
                    backgroundColor: colors.dangerSoft,
                    borderColor: colors.danger,
                  }}
                >
                  <text style={{ fontSize: 11, color: colors.danger }}>
                    Confirm delete
                  </text>
                </Button>
              ) : null}
            </div>
          ))
        )}
        {viewCapture ? (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 7,
              padding: 9,
              borderRadius: 7,
              borderWidth: 1,
              borderColor: colors.line,
              backgroundColor: colors.code,
            }}
          >
            <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
              <text style={{ fontSize: 12, fontWeight: 600, flexGrow: 1 }}>
                Capture version {viewCapture.version} details
              </text>
              <Button
                onClick={() => {
                  setViewCapture(null);
                  setViewFile(null);
                }}
                style={buttonBase}
              >
                <text style={{ fontSize: 11 }}>Close</text>
              </Button>
            </div>
            <text style={{ fontSize: 11, color: colors.muted }}>
              Saved {viewCapture.createdAt} / {viewCapture.files.length} files /{" "}
              {viewCapture.totalBytes} bytes / skipped{" "}
              {viewCapture.skipped.length}
            </text>
            <div style={{ display: "flex", flexWrap: "wrap", gap: 5 }}>
              {viewCapture.files.map((file) => (
                <Button
                  key={file.path}
                  onClick={() => setViewFile(file.path)}
                  style={{
                    ...buttonBase,
                    backgroundColor:
                      viewFile === file.path ? colors.accentSoft : colors.panel,
                  }}
                >
                  <text style={{ fontSize: 11 }}>
                    {file.path} / {file.size} bytes / {file.hash.slice(0, 12)}
                  </text>
                </Button>
              ))}
            </div>
            {viewFile ? (
              <textarea
                readOnly
                value={
                  viewCapture.files.find((file) => file.path === viewFile)
                    ?.content ?? ""
                }
                style={{
                  ...inputBase,
                  height: 180,
                  paddingTop: 7,
                  fontFamily: "monospace",
                }}
              />
            ) : null}
            {viewCapture.skipped.length > 0 ? (
              <div
                style={{
                  maxHeight: 100,
                  overflow: "scroll",
                  display: "flex",
                  flexDirection: "column",
                  gap: 3,
                }}
              >
                {viewCapture.skipped.map((item) => (
                  <text
                    key={`${item.path}:${item.reason}`}
                    style={{ fontSize: 10, color: colors.muted }}
                  >
                    skipped {item.path}: {item.reason}
                  </text>
                ))}
              </div>
            ) : null}
            <text style={{ fontSize: 11, color: colors.muted }}>
              Git:{" "}
              {viewCapture.git
                ? `${viewCapture.git.branch ?? "detached"}${viewCapture.git.dirty ? " (dirty)" : " (clean)"}`
                : "not available"}
            </text>
          </div>
        ) : null}
      </div>
    </div>
  );
};

const App = ({ runtime }: { runtime: ApplicationRuntime }) => {
  const app = runtime.operations;
  const [profile, setProfile] = useState<DeveloperProfile | null>(null);
  const [draft, setDraft] = useState<Partial<DeveloperProfile> | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [documents, setDocuments] = useState<ImportedDocument[]>([]);
  const [agents, setAgents] = useState<CodingAgentStatus[]>([]);

  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(
    null
  );

  const [section, setSection] = useState("Profile");
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [exportText, setExportText] = useState("");
  const [exportPath, setExportPath] = useState("");
  const [deleteProjectId, setDeleteProjectId] = useState("");
  const [deleteAll, setDeleteAll] = useState(false);
  const [loading, setLoading] = useState(true);

  const selectedProject =
    projects.find((project) => project.id === selectedProjectId) ?? null;

  const saveExportFile = async () => {
    setError("");

    try {
      const text =
        exportText || JSON.stringify(await app.exportData(), null, 2);
      const stamp = new Date().toISOString().replaceAll(":", "-");
      const target = `${process.cwd()}/coderune-export-${stamp}.json`;

      await Bun.write(target, text);
      setExportPath(target);
      setMessage(`JSON export written to ${target}`);
    } catch (error) {
      setError(errorText(error));
    }
  };

  const refresh = async () => {
    setError("");

    try {
      const [onboarding, projectList, documentList, agentList] =
        await Promise.all([
          app.getOnboardingState(),
          app.listProjects().catch(() => []),
          app.exportData().then((data) => data.documents as ImportedDocument[]),
          app.listCodingAgents(),
        ]);

      setProfile(onboarding.profile);
      setDraft(onboarding.draft as Partial<DeveloperProfile> | null);
      setProjects(projectList);
      setDocuments(documentList);
      setAgents(agentList);
      setSelectedProjectId((current) => {
        if (current && projectList.some((project) => project.id === current)) {
          return current;
        }

        return projectList[0]?.id ?? null;
      });
    } catch (error) {
      setError(errorText(error));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void refresh();
  }, []);

  const notify = (text: string) => {
    setError("");
    setMessage(text);
  };

  if (loading) {
    return (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: colors.bg,
        }}
      >
        <text style={{ fontSize: 13, color: colors.muted }}>
          Loading Coderune...
        </text>
      </div>
    );
  }

  if (!profile) {
    return (
      <div
        style={{
          width: "100%",
          height: "100%",
          overflow: "scroll",
          backgroundColor: colors.bg,
          padding: 24,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
        }}
      >
        <div
          style={{
            width: "100%",
            maxWidth: 980,
            display: "flex",
            flexDirection: "column",
            gap: 16,
          }}
        >
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <text style={{ fontSize: 26, fontWeight: 700, color: colors.ink }}>
              Coderune
            </text>
            <text
              style={{
                fontSize: 13,
                color: colors.muted,
                whiteSpace: "normal",
              }}
            >
              Confirm a developer baseline before project work. Manual entry and
              reviewed CV import both work without a coding-agent connection.
            </text>
          </div>
          {terminalWarning ? (
            <Notice error={terminalWarning} message="" />
          ) : null}
          <Notice error={error} message={message} />
          <Section
            title="Developer baseline"
            description="A display name or pseudonym, experience stage, skills starting point, and at least one learning goal are required."
          >
            <ProfileEditor
              key="onboarding"
              initial={draft}
              submitLabel="Confirm developer baseline"
              onDraft={async (input) => {
                const saved = await app.saveOnboardingDraft(input);
                setDraft(saved as Partial<DeveloperProfile>);
                notify(
                  "Onboarding draft saved. You can close and resume later."
                );
              }}
              onSubmit={async (input) => {
                const saved = await app.confirmOnboarding(input);
                setProfile(saved);
                setDraft(null);
                setMessage("Developer baseline confirmed.");
              }}
            />
          </Section>
          <Section
            title="Assisted CV import"
            description="Extract a local CV, preview the exact text shared with your selected agent, then review and correct the structured draft."
          >
            <AgentDraftPanel
              app={app}
              kind="cv"
              projectId={null}
              documents={documents}
              agents={agents}
              onDocumentsChanged={refresh}
              onSaved={refresh}
            />
          </Section>
        </div>
      </div>
    );
  }

  const nav = ["Profile", "Projects", "Documents", "Agents", "Capture", "Data"];

  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        backgroundColor: colors.bg,
      }}
    >
      <div
        style={{
          width: 190,
          height: "100%",
          padding: 16,
          display: "flex",
          flexDirection: "column",
          gap: 4,
          borderRightWidth: 1,
          borderColor: colors.line,
          backgroundColor: colors.panel,
        }}
      >
        <text style={{ fontSize: 18, fontWeight: 700, marginBottom: 12 }}>
          Coderune
        </text>
        {nav.map((item) => (
          <Button
            key={item}
            onClick={() => setSection(item)}
            style={{
              ...buttonBase,
              borderColor: section === item ? colors.accent : "transparent",
              backgroundColor:
                section === item ? colors.accentSoft : "transparent",
              justifyContent: "flex-start",
            }}
          >
            <text style={{ fontSize: 13 }}>{item}</text>
          </Button>
        ))}
        <div style={{ flexGrow: 1 }} />
        <text
          style={{ fontSize: 10, color: colors.muted, whiteSpace: "normal" }}
        >
          {isTerminal() ? "GPUIX terminal mode" : "GPUIX desktop mode"}
        </text>
        <text
          style={{ fontSize: 10, color: colors.muted, whiteSpace: "normal" }}
        >
          Data stays in {runtime.store ? "local SQLite" : "local storage"}.
        </text>
      </div>

      <div
        style={{
          flexGrow: 1,
          height: "100%",
          overflow: "scroll",
          padding: 22,
        }}
      >
        <div
          style={{
            maxWidth: 1000,
            display: "flex",
            flexDirection: "column",
            gap: 16,
          }}
        >
          {terminalWarning ? (
            <Notice error={terminalWarning} message="" />
          ) : null}
          <Notice error={error} message={message} />

          {section === "Profile" ? (
            <>
              <Section
                title={profile.displayName}
                description={`Developer profile version ${profile.version}. ${profile.experienceStage.replaceAll("-", " ")}.`}
              >
                <div
                  style={{ display: "flex", flexDirection: "column", gap: 8 }}
                >
                  <text style={{ fontSize: 12, whiteSpace: "normal" }}>
                    Skills: {profile.skillsStatement}
                  </text>
                  <text style={{ fontSize: 12, whiteSpace: "normal" }}>
                    Learning goals: {profile.learningGoals.join("; ")}
                  </text>
                  {profile.fieldSources &&
                  Object.keys(profile.fieldSources).length > 0 ? (
                    <text
                      style={{
                        fontSize: 11,
                        color: colors.muted,
                        whiteSpace: "normal",
                      }}
                    >
                      Some fields retain document provenance for review.
                    </text>
                  ) : null}
                </div>
              </Section>
              <Section
                title="Edit developer profile"
                description="Edits affect future captures; existing captures keep their profile snapshot."
              >
                <ProfileEditor
                  key={`profile-${profile.version}`}
                  initial={profile}
                  submitLabel="Save profile version"
                  onSubmit={async (input) => {
                    const saved = await app.updateProfile(input);
                    setProfile(saved);
                    notify(`Profile version ${saved.version} saved.`);
                  }}
                />
              </Section>
            </>
          ) : null}

          {section === "Projects" ? (
            <>
              <Section
                title="Projects"
                description="A project needs a name, readable root folder, and at least one goal."
              >
                <div style={{ display: "flex", flexWrap: "wrap", gap: 6 }}>
                  {projects.map((project) => (
                    <Button
                      key={project.id}
                      onClick={() => {
                        setSelectedProjectId(project.id);
                        setDeleteProjectId("");
                      }}
                      style={{
                        ...buttonBase,
                        backgroundColor:
                          selectedProjectId === project.id
                            ? colors.accentSoft
                            : colors.panel,
                        borderColor:
                          selectedProjectId === project.id
                            ? colors.accent
                            : colors.line,
                      }}
                    >
                      <text style={{ fontSize: 12 }}>
                        {project.name} / v{project.version}
                      </text>
                    </Button>
                  ))}
                  <Button
                    onClick={() => {
                      setSelectedProjectId(null);
                      setDeleteProjectId("");
                    }}
                    style={buttonBase}
                  >
                    <text style={{ fontSize: 12 }}>New project</text>
                  </Button>
                </div>
              </Section>
              <Section
                title={selectedProject ? "Edit project" : "Create project"}
                description="Project context combines developer statements with optional reviewed SRS-derived fields."
              >
                <ProjectEditor
                  key={
                    selectedProject
                      ? `${selectedProject.id}-${selectedProject.version}`
                      : "new-project"
                  }
                  project={selectedProject}
                  onCreate={async (input) => {
                    const created = await app.createProject(input);
                    setSelectedProjectId(created.id);
                    notify(`Project ${created.name} created.`);
                    await refresh();
                  }}
                  onUpdate={async (id, input) => {
                    const saved = await app.updateProject(id, input);
                    setSelectedProjectId(saved.id);
                    notify(`Project version ${saved.version} saved.`);
                    await refresh();
                  }}
                />
                {selectedProject ? (
                  <div style={{ display: "flex", gap: 8 }}>
                    <Button
                      onClick={() => setDeleteProjectId(selectedProject.id)}
                      style={{ ...buttonBase, borderColor: colors.danger }}
                    >
                      <text style={{ fontSize: 11, color: colors.danger }}>
                        Delete project
                      </text>
                    </Button>
                    {deleteProjectId === selectedProject.id ? (
                      <Button
                        onClick={() => {
                          void (async () => {
                            try {
                              await app.deleteProject(selectedProject.id);
                              setDeleteProjectId("");
                              setSelectedProjectId(null);
                              notify(
                                "Project and its captures deleted. Profile and other projects remain."
                              );
                              await refresh();
                            } catch (error) {
                              setError(errorText(error));
                            }
                          })();
                        }}
                        style={{
                          ...buttonBase,
                          backgroundColor: colors.dangerSoft,
                          borderColor: colors.danger,
                        }}
                      >
                        <text style={{ fontSize: 11, color: colors.danger }}>
                          Confirm project delete
                        </text>
                      </Button>
                    ) : null}
                  </div>
                ) : null}
              </Section>
            </>
          ) : null}

          {section === "Documents" ? (
            <>
              <Section
                title="CV document import"
                description="Use local extraction and reviewed agent assistance for profile fields."
              >
                <AgentDraftPanel
                  app={app}
                  kind="cv"
                  projectId={null}
                  documents={documents}
                  agents={agents}
                  onDocumentsChanged={refresh}
                  onSaved={refresh}
                />
              </Section>
              <Section
                title="SRS document import"
                description={
                  selectedProject
                    ? `Project: ${selectedProject.name}`
                    : "Select a project first."
                }
              >
                {selectedProject ? (
                  <AgentDraftPanel
                    key={`srs-${selectedProject.id}`}
                    app={app}
                    kind="srs"
                    projectId={selectedProject.id}
                    documents={documents}
                    agents={agents}
                    onDocumentsChanged={refresh}
                    onSaved={refresh}
                  />
                ) : (
                  <text style={{ fontSize: 12, color: colors.muted }}>
                    Create or select a project in Projects.
                  </text>
                )}
              </Section>
              <Section
                title="Saved document text"
                description="Original files are not copied. The reviewed extracted text remains available."
              >
                {documents.length === 0 ? (
                  <text style={{ fontSize: 12, color: colors.muted }}>
                    No documents saved.
                  </text>
                ) : (
                  documents.map((document) => (
                    <div
                      key={document.id}
                      style={{
                        padding: 8,
                        borderRadius: 6,
                        borderWidth: 1,
                        borderColor: colors.line,
                      }}
                    >
                      <text style={{ fontSize: 12, fontWeight: 600 }}>
                        {document.kind.toUpperCase()} / {document.name}
                      </text>
                      <text style={{ fontSize: 10, color: colors.muted }}>
                        {document.size} bytes / hash{" "}
                        {document.hash.slice(0, 12)} / imported{" "}
                        {document.importedAt}
                      </text>
                    </div>
                  ))
                )}
              </Section>
            </>
          ) : null}

          {section === "Agents" ? (
            <Section
              title="Coding-agent compatibility"
              description="Coderune connects only when you request a draft. Each agent owns sign-in, provider, and model configuration. No adapter is marked verified until compatibility is demonstrated."
            >
              <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
                {agents.map((agent) => (
                  <div
                    key={agent.id}
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      gap: 5,
                      padding: 10,
                      borderRadius: 8,
                      borderWidth: 1,
                      borderColor: colors.line,
                    }}
                  >
                    <div
                      style={{ display: "flex", gap: 8, alignItems: "center" }}
                    >
                      <text style={{ fontSize: 13, fontWeight: 600 }}>
                        {agent.name}
                      </text>
                      <text style={{ fontSize: 10, color: colors.muted }}>
                        {agent.availability} / verified:{" "}
                        {agent.verified ? "yes" : "no"}
                      </text>
                    </div>
                    <text
                      style={{
                        fontSize: 11,
                        color: colors.muted,
                        whiteSpace: "normal",
                      }}
                    >
                      {agent.detail}
                    </text>
                    <text style={{ fontSize: 10, whiteSpace: "normal" }}>
                      Structured output:{" "}
                      {agent.capabilities.structuredOutput
                        ? "yes"
                        : "validated JSON"}{" "}
                      / cancellation:{" "}
                      {agent.capabilities.cancellation ? "yes" : "no"} /
                      streaming: {agent.capabilities.streaming ? "yes" : "no"} /
                      isolation: {agent.capabilities.documentOnlyIsolation}
                    </text>
                    <text
                      style={{
                        fontSize: 10,
                        color: colors.muted,
                        whiteSpace: "normal",
                      }}
                    >
                      {agent.capabilities.restrictions.join(" ")}
                    </text>
                  </div>
                ))}
              </div>
            </Section>
          ) : null}

          {section === "Capture" ? (
            selectedProject ? (
              <Section
                title={`Capture ${selectedProject.name}`}
                description="Collection is explicit. Preview the exact file list before saving. Saving is bound to the reviewed content."
              >
                <CapturePanel
                  key={`${selectedProject.id}-${selectedProject.version}`}
                  app={app}
                  project={selectedProject}
                  onChanged={refresh}
                />
              </Section>
            ) : (
              <Section title="Capture">
                <text style={{ fontSize: 12, color: colors.muted }}>
                  Create or select a project first.
                </text>
              </Section>
            )
          ) : null}

          {section === "Data" ? (
            <>
              <Section
                title="Export JSON"
                description="Includes profile versions, projects, documents, and capture history."
              >
                <Button
                  onClick={() => {
                    void (async () => {
                      try {
                        const data = await app.exportData();
                        setExportText(JSON.stringify(data, null, 2));
                        notify("Export prepared locally.");
                      } catch (error) {
                        setError(errorText(error));
                      }
                    })();
                  }}
                  style={buttonBase}
                >
                  <text style={{ fontSize: 12 }}>Prepare export</text>
                </Button>
                <Button
                  onClick={() => void saveExportFile()}
                  style={buttonBase}
                >
                  <text style={{ fontSize: 12 }}>Save JSON file</text>
                </Button>
                {exportPath ? (
                  <text style={{ fontSize: 12, color: colors.muted }}>
                    Saved to {exportPath}
                  </text>
                ) : null}
                {exportText ? (
                  <textarea
                    value={exportText}
                    readOnly
                    minRows={12}
                    style={{
                      ...inputBase,
                      height: 320,
                      paddingTop: 7,
                      fontFamily: "monospace",
                    }}
                  />
                ) : null}
              </Section>
              <Section
                title="Delete all application data"
                description="Removes the profile, projects, documents, and captures. This cannot be undone."
              >
                <Button
                  onClick={() => setDeleteAll((value) => !value)}
                  style={{
                    ...buttonBase,
                    alignSelf: "flex-start",
                    borderColor: colors.danger,
                    backgroundColor: deleteAll
                      ? colors.dangerSoft
                      : colors.panel,
                  }}
                >
                  <text style={{ fontSize: 12, color: colors.danger }}>
                    {deleteAll
                      ? "Delete everything is armed"
                      : "Delete all data"}
                  </text>
                </Button>
                {deleteAll ? (
                  <Button
                    onClick={() => {
                      void (async () => {
                        try {
                          await app.deleteAllData();
                          setDeleteAll(false);
                          setExportText("");
                          setExportPath("");
                          setProfile(null);
                          setProjects([]);
                          setDocuments([]);
                          setSelectedProjectId(null);
                          notify("All application data deleted.");
                          await refresh();
                        } catch (error) {
                          setError(errorText(error));
                        }
                      })();
                    }}
                    style={{
                      ...buttonBase,
                      alignSelf: "flex-start",
                      borderColor: colors.danger,
                      backgroundColor: colors.danger,
                    }}
                  >
                    <text
                      style={{
                        fontSize: 12,
                        color: "#ffffff",
                        fontWeight: 600,
                      }}
                    >
                      Confirm delete everything
                    </text>
                  </Button>
                ) : null}
              </Section>
            </>
          ) : null}
        </div>
      </div>
    </div>
  );
};

const runtime = await createApplication({
  dataDir: resolveApplicationDataDir(),
});

process.on("exit", () => runtime.close());

render(<App runtime={runtime} />, {
  title: "Coderune",
  width: 1240,
  height: 820,
});
