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
  JsonValue,
  Project,
  ProjectInput,
  CapturePreview,
} from "@coderune/core";
import {
  Alert as Notice,
  Badge,
  Button,
  Card as Section,
  Field as Labeled,
  Input,
  PageHeader,
  QuartzFonts,
  Sidebar,
  Text,
  Textarea,
  View,
} from "@coderune/quartz/native";
import { quartz } from "@coderune/quartz/tokens";
import { resolveApplicationDataDir } from "@coderune/storage";
import { isTerminal, render, useWindowSize } from "@gpuix/react";
import { useEffect, useCallback, useRef, useState } from "react";

import { createApplication } from "./src/application";
import type { ApplicationRuntime } from "./src/application";

const { colors } = quartz;

const terminalWarning =
  isTerminal() && (process.platform === "win32" || Boolean(process.env.TMUX))
    ? "This terminal combination is not supported for GPUIX terminal mode. Use native desktop mode, or a compatible macOS/Linux terminal without tmux."
    : "";

const lines = function lines(value: string): string[] {
  return value
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
};

const errorText = function errorText(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
};

const stageLabel = function stageLabel(stage: ExperienceStage): string {
  return stage.replaceAll("-", " ");
};

const projectSaveLabel = function projectSaveLabel(
  busy: boolean,
  hasProject: boolean
): string {
  if (busy) {
    return "Saving...";
  }

  if (hasProject) {
    return "Save project version";
  }

  return "Create project";
};

type AppSection =
  | "Profile"
  | "Projects"
  | "Documents"
  | "Agents"
  | "Capture"
  | "Data";

const sectionTitle = function sectionTitle(section: AppSection): string {
  if (section === "Profile") {
    return "Developer profile";
  }

  if (section === "Data") {
    return "Your data";
  }

  return section;
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

  const [goals, setGoals] = useState(() =>
    (initial?.learningGoals ?? []).join("\n")
  );

  const [details, setDetails] = useState(() =>
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
  const [errorMessage, setErrorMessage] = useState("");
  const draftHandler = useRef(onDraft);

  useEffect(() => {
    draftHandler.current = onDraft;
  }, [onDraft]);

  const buildInput = useCallback((): DeveloperProfileInput => {
    // SAFETY: details is produced by JSON.stringify from the typed DeveloperProfileInput defaults.
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
  }, [
    details,
    displayName,
    goals,
    initial?.fieldSources,
    newToCoding,
    skills,
    stage,
  ]);

  useEffect(() => {
    const saveDraft = draftHandler.current;

    if (!saveDraft) {
      return;
    }

    const timer = setTimeout(() => {
      void (async () => {
        try {
          await saveDraft(buildInput());
        } catch {
          // Optional draft persistence errors are ignored while typing.
        }
      })();
    }, 700);

    return () => clearTimeout(timer);
  }, [buildInput]);

  const run = async (
    action: (input: DeveloperProfileInput) => Promise<void>
  ) => {
    setBusy(true);
    setErrorMessage("");

    try {
      await action(buildInput());
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  return (
    <View
      style={{ display: "flex", flexDirection: "column", gap: quartz.space[3] }}
    >
      <Notice error={errorMessage} message="" />
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[4],
        }}
      >
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Display name or pseudonym">
            <Input
              value={displayName}
              onChange={(event) => setDisplayName(event.value ?? "")}
              placeholder="Ada"
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Experience stage">
            <View
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: quartz.space[2],
              }}
            >
              {(
                [
                  "new-to-coding",
                  "early-career",
                  "mid-career",
                  "experienced",
                ] satisfies ExperienceStage[]
              ).map((option) => (
                <Button
                  key={option}
                  pressed={stage === option}
                  onClick={() => setStage(option)}
                  style={{
                    backgroundColor:
                      stage === option ? colors.accentSoft : colors.panel,
                    borderColor: stage === option ? colors.accent : colors.line,
                  }}
                >
                  <Text style={{ ...quartz.type.bodySm }}>
                    {stageLabel(option)}
                  </Text>
                </Button>
              ))}
            </View>
          </Labeled>
        </View>
      </View>

      <Button
        pressed={newToCoding}
        onClick={() => setNewToCoding((value) => !value)}
        style={{
          alignSelf: "flex-start",
          backgroundColor: newToCoding ? colors.successSoft : colors.panel,
        }}
      >
        <Text style={{ ...quartz.type.bodySm }}>
          {newToCoding
            ? "New to coding; no skills listed yet"
            : "I have an initial skills statement"}
        </Text>
      </Button>

      {newToCoding ? null : (
        <Labeled label="Initial skills statement">
          <Textarea
            value={skills}
            onChange={(event) => setSkills(event.value ?? "")}
            minRows={2}
            style={{ paddingTop: quartz.space[2] }}
          />
        </Labeled>
      )}

      <Labeled
        label="Learning goals"
        hint="One goal per line. At least one is required."
      >
        <Textarea
          value={goals}
          onChange={(event) => setGoals(event.value ?? "")}
          minRows={3}
          style={{ paddingTop: quartz.space[2] }}
        />
      </Labeled>

      <Labeled
        label="Optional profile detail (JSON)"
        hint="Education, employment, projects, certifications, links, skills, and preferences. Missing fields are allowed."
      >
        <Textarea
          value={details}
          onChange={(event) => setDetails(event.value ?? "")}
          minRows={8}
          style={{
            paddingTop: quartz.space[2],
            fontFamily: quartz.fonts.mono,
          }}
        />
      </Labeled>

      <View style={{ display: "flex", flexWrap: "wrap", gap: quartz.space[2] }}>
        {onDraft ? (
          <Button
            onClick={async () => {
              await run((input) => onDraft(input));
            }}
            disabled={busy}
            style={{ opacity: busy ? quartz.opacity.disabled : 1 }}
          >
            <Text style={{ ...quartz.type.bodySm }}>Save draft</Text>
          </Button>
        ) : null}
        <Button
          testId="save-profile"
          onClick={async () => {
            await run(onSubmit);
          }}
          disabled={busy}
          variant="primary"
          style={{}}
        >
          <Text
            style={{
              ...quartz.type.bodySm,
              fontWeight: quartz.type.label.fontWeight,
            }}
          >
            {busy ? "Saving..." : submitLabel}
          </Text>
        </Button>
      </View>
    </View>
  );
};

// Project editing keeps its shared draft state and validation in one place.
// oxlint-disable-next-line react-doctor/no-giant-component
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

  const [goals, setGoals] = useState(() =>
    (project?.goals ?? []).map((item) => item.text).join("\n")
  );

  const [purpose, setPurpose] = useState(project?.purpose?.text ?? "");

  const [businessLogic, setBusinessLogic] = useState(
    project?.businessLogic?.text ?? ""
  );

  const [rules, setRules] = useState(() =>
    (project?.rules ?? []).map((item) => item.text).join("\n")
  );

  const [constraints, setConstraints] = useState(() =>
    (project?.constraints ?? []).map((item) => item.text).join("\n")
  );

  const [decisions, setDecisions] = useState(() =>
    (project?.decisions ?? []).map((item) => item.text).join("\n")
  );

  const [acceptance, setAcceptance] = useState(() =>
    (project?.acceptanceCriteria ?? []).map((item) => item.text).join("\n")
  );

  const [knowledge, setKnowledge] = useState(() =>
    (project?.knowledge ?? []).map((item) => item.text).join("\n")
  );

  const [includePaths, setIncludePaths] = useState(() =>
    (project?.scope.includePaths ?? []).join("\n")
  );

  const [excludePaths, setExcludePaths] = useState(() =>
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
  const [errorMessage, setErrorMessage] = useState("");

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
    setErrorMessage("");

    try {
      await (project ? onUpdate(project.id, input()) : onCreate(input()));
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  return (
    <View
      style={{ display: "flex", flexDirection: "column", gap: quartz.space[3] }}
    >
      <Notice error={errorMessage} message="" />
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[4],
        }}
      >
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Project name">
            <Input
              value={name}
              onChange={(event) => setName(event.value ?? "")}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 2 }}>
          <Labeled label="Readable root folder">
            <Input
              value={rootPath}
              style={{ fontFamily: quartz.fonts.mono }}
              onChange={(event) => setRootPath(event.value ?? "")}
              placeholder="/Users/me/project"
            />
          </Labeled>
        </View>
      </View>
      <Labeled label="Goals" hint="One per line. At least one is required.">
        <Textarea
          value={goals}
          onChange={(event) => setGoals(event.value ?? "")}
          minRows={2}
          style={{ paddingTop: quartz.space[2] }}
        />
      </Labeled>
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[4],
        }}
      >
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Purpose">
            <Textarea
              value={purpose}
              onChange={(event) => setPurpose(event.value ?? "")}
              minRows={2}
              style={{ paddingTop: quartz.space[2] }}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Business logic">
            <Textarea
              value={businessLogic}
              onChange={(event) => setBusinessLogic(event.value ?? "")}
              minRows={2}
              style={{ paddingTop: quartz.space[2] }}
            />
          </Labeled>
        </View>
      </View>
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[4],
        }}
      >
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Rules">
            <Textarea
              value={rules}
              onChange={(event) => setRules(event.value ?? "")}
              style={{ paddingTop: quartz.space[2] }}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Constraints">
            <Textarea
              value={constraints}
              onChange={(event) => setConstraints(event.value ?? "")}
              style={{ paddingTop: quartz.space[2] }}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Decisions">
            <Textarea
              value={decisions}
              onChange={(event) => setDecisions(event.value ?? "")}
              style={{ paddingTop: quartz.space[2] }}
            />
          </Labeled>
        </View>
      </View>
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[4],
        }}
      >
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Acceptance criteria">
            <Textarea
              value={acceptance}
              onChange={(event) => setAcceptance(event.value ?? "")}
              style={{ paddingTop: quartz.space[2] }}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Other knowledge">
            <Textarea
              value={knowledge}
              onChange={(event) => setKnowledge(event.value ?? "")}
              style={{ paddingTop: quartz.space[2] }}
            />
          </Labeled>
        </View>
      </View>
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[4],
        }}
      >
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Include paths" hint="Optional. One per line.">
            <Textarea
              value={includePaths}
              onChange={(event) => setIncludePaths(event.value ?? "")}
              style={{
                paddingTop: quartz.space[2],
                fontFamily: quartz.fonts.mono,
              }}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Exclude paths" hint="Optional. One per line.">
            <Textarea
              value={excludePaths}
              onChange={(event) => setExcludePaths(event.value ?? "")}
              style={{
                paddingTop: quartz.space[2],
                fontFamily: quartz.fonts.mono,
              }}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Capture limits" hint="MiB per file and per capture.">
            <View
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: quartz.space[2],
              }}
            >
              <Input
                aria-label="Per-file size limit in MiB"
                value={maxFileMiB}
                onChange={(event) => setMaxFileMiB(event.value ?? "")}
                style={{ flexGrow: 1 }}
              />
              <Input
                aria-label="Per-capture size limit in MiB"
                value={maxCaptureMiB}
                onChange={(event) => setMaxCaptureMiB(event.value ?? "")}
                style={{ flexGrow: 1 }}
              />
            </View>
          </Labeled>
        </View>
      </View>
      <View
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: quartz.space[2],
          alignItems: "center",
        }}
      >
        <Button
          pressed={respectGitignore}
          onClick={() => setRespectGitignore((value) => !value)}
          style={{}}
        >
          <Text style={{ ...quartz.type.bodySm }}>
            {respectGitignore ? "Respect .gitignore" : "Ignore .gitignore"}
          </Text>
        </Button>
        <Button
          onClick={async () => {
            await submit();
          }}
          disabled={busy}
          variant="primary"
          style={{}}
        >
          <Text
            style={{
              ...quartz.type.bodySm,
              fontWeight: quartz.type.label.fontWeight,
            }}
          >
            {projectSaveLabel(busy, Boolean(project))}
          </Text>
        </Button>
      </View>
    </View>
  );
};

// Draft review keeps the document, agent, and preview state in one workflow.
// oxlint-disable-next-line react-doctor/no-giant-component, eslint/complexity
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
    () =>
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
  const [errorMessage, setErrorMessage] = useState("");
  const [message, setMessage] = useState("");

  const selected = availableDocs.find(
    (document) => document.id === selectedDocumentId
  );

  const clear = () => {
    setErrorMessage("");
    setMessage("");
  };

  const importDocument = async () => {
    setBusy(true);
    clear();

    try {
      const result = await app.extractDocument({
        path: documentPath,
        kind,
        maxBytes: Math.max(1, Number(maxDocumentMiB) || 20) * 1_048_576,
      });

      setExtraction(result);

      if (!result.ok) {
        setErrorMessage(
          `${result.error.message} You can keep entering the ${kind.toUpperCase()} manually.`
        );
      }
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setBusy(false);
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
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  const prepare = async () => {
    if (!selected || !agentId) {
      setErrorMessage("Select an imported document and a coding agent.");

      return;
    }

    if (kind === "srs" && !projectId) {
      setErrorMessage("Select a project before preparing an SRS draft.");

      return;
    }

    setBusy(true);
    clear();

    try {
      const next =
        kind === "srs" && projectId
          ? await app.prepareSrsDraft(selected.id, agentId, projectId)
          : await app.prepareCvDraft(selected.id, agentId);

      setPreview(next);
      setShareConfirmed(false);
      setDraftJson("");
      setApplyConfirmed(false);
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  const requestDraft = async () => {
    if (!selected || !preview) {
      return;
    }

    if (kind === "srs" && !projectId) {
      setErrorMessage("Select a project before requesting an SRS draft.");

      return;
    }

    const nextController = new AbortController();
    setController(nextController);
    setBusy(true);
    clear();

    try {
      setMessage("Waiting for the selected coding agent...");

      const result =
        kind === "srs" && projectId
          ? await app.requestSrsDraft(
              selected.id,
              preview.agent.id,
              projectId,
              {
                confirmed: true,
                signal: nextController.signal,
                onProgress: (text) => setMessage(text),
              }
            )
          : await app.requestCvDraft(selected.id, preview.agent.id, {
              confirmed: true,
              signal: nextController.signal,
              onProgress: (text) => setMessage(text),
            });

      setDraftJson(JSON.stringify(result.draft, null, 2));
      setDraftAgent(result.agent.name);
      setMessage(
        `${result.agent.name} returned a schema-valid draft. Review and correct every field below.`
      );
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setController(null);
    setBusy(false);
  };

  const applyDraft = async () => {
    if (!selected) {
      return;
    }

    if (kind === "srs" && !projectId) {
      setErrorMessage("Select a project before saving an SRS draft.");

      return;
    }

    setBusy(true);
    clear();

    try {
      const parsed: JsonValue = JSON.parse(draftJson);

      await (kind === "srs" && projectId
        ? app.applySrsDraft(projectId, selected.id, parsed, {
            confirmed: true,
          })
        : app.applyCvDraft(selected.id, parsed, { confirmed: true }));

      setMessage("Reviewed draft saved.");
      setPreview(null);
      setDraftJson("");
      setApplyConfirmed(false);
      await onSaved();
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  return (
    <View
      style={{ display: "flex", flexDirection: "column", gap: quartz.space[3] }}
    >
      <Notice error={errorMessage} message={message} />
      <Labeled
        label={`Import a local ${kind.toUpperCase()}`}
        hint="PDF, DOCX, Markdown, or plain text. Text is extracted locally; the original file is not copied. The number is the per-document MiB limit."
      >
        <View
          style={{ display: "flex", flexWrap: "wrap", gap: quartz.space[2] }}
        >
          <Input
            aria-label="Document path"
            value={documentPath}
            onChange={(event) => setDocumentPath(event.value ?? "")}
            placeholder={`/Users/me/${kind}.pdf`}
            style={{ flexGrow: 1, fontFamily: quartz.fonts.mono }}
          />
          <Input
            aria-label="Document size limit in MiB"
            value={maxDocumentMiB}
            onChange={(event) => setMaxDocumentMiB(event.value ?? "")}
            style={{ width: quartz.space[20] }}
          />
          <Button
            onClick={async () => {
              await importDocument();
            }}
            disabled={busy || !documentPath}
            style={{ opacity: busy ? quartz.opacity.disabled : 1 }}
          >
            <Text style={{ ...quartz.type.bodySm }}>Extract</Text>
          </Button>
        </View>
      </Labeled>

      {extraction?.ok ? (
        <View
          style={{
            display: "flex",
            flexDirection: "column",
            gap: quartz.space[2],
          }}
        >
          <View
            style={{
              maxHeight: quartz.layout.fieldMaxHeight,
              overflow: "scroll",
              padding: quartz.space[2],
              borderRadius: quartz.radius.soft,
              backgroundColor: colors.code,
            }}
          >
            <Text
              style={{
                ...quartz.type.bodySm,
                color: colors.muted,
                whiteSpace: "normal",
              }}
            >
              {extraction.document.text}
            </Text>
          </View>
          <Button
            onClick={async () => {
              await confirmDocument();
            }}
            disabled={busy}
            style={{ alignSelf: "flex-start" }}
          >
            <Text style={{ ...quartz.type.bodySm }}>
              Save reviewed document text
            </Text>
          </Button>
        </View>
      ) : null}

      <Labeled label="Imported document">
        <View
          style={{ display: "flex", flexWrap: "wrap", gap: quartz.space[2] }}
        >
          {availableDocs.length === 0 ? (
            <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
              No imported {kind.toUpperCase()} yet.
            </Text>
          ) : (
            availableDocs.map((document) => (
              <Button
                key={document.id}
                pressed={selectedDocumentId === document.id}
                onClick={() => {
                  setSelectedDocumentId(document.id);
                  setPreview(null);
                  clear();
                }}
                style={{
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
                <Text style={{ ...quartz.type.bodySm }}>{document.name}</Text>
              </Button>
            ))
          )}
        </View>
      </Labeled>

      <Labeled
        label="Coding agent"
        hint="Coderune never switches agents or providers after a failure."
      >
        <View
          style={{
            display: "flex",
            flexDirection: "column",
            gap: quartz.space[2],
          }}
        >
          {agents.map((agent) => (
            <Button
              key={agent.id}
              aria-label={`Select ${agent.name}`}
              pressed={agentId === agent.id}
              onClick={() => {
                setAgentId(agent.id);
                setPreview(null);
                clear();
              }}
              disabled={agent.availability !== "ready"}
              style={{
                textAlign: "left",
                opacity:
                  agent.availability === "ready" ? 1 : quartz.opacity.disabled,
                backgroundColor:
                  agentId === agent.id ? colors.accentSoft : colors.panel,
                borderColor: agentId === agent.id ? colors.accent : colors.line,
              }}
            >
              <View
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: quartz.space[1],
                }}
              >
                <View
                  style={{
                    display: "flex",
                    flexWrap: "wrap",
                    gap: quartz.space[2],
                    alignItems: "center",
                  }}
                >
                  <Text
                    style={{
                      ...quartz.type.bodySm,
                      fontWeight: quartz.type.label.fontWeight,
                    }}
                  >
                    {agent.name}
                  </Text>
                  <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
                    {agent.availability === "ready"
                      ? "installed"
                      : "not installed"}
                    {agent.verified
                      ? " / verified"
                      : " / compatibility not yet demonstrated"}
                  </Text>
                </View>
                <Text
                  style={{
                    ...quartz.type.bodySm,
                    color: colors.muted,
                    whiteSpace: "normal",
                  }}
                >
                  {agent.detail}
                </Text>
                <Text
                  style={{
                    ...quartz.type.bodySm,
                    color: colors.muted,
                    whiteSpace: "normal",
                  }}
                >
                  {agent.capabilities.restrictions.join(" ")}
                </Text>
              </View>
            </Button>
          ))}
        </View>
      </Labeled>

      <Button
        onClick={async () => {
          await prepare();
        }}
        disabled={busy || !selected || !agentId}
        style={{
          alignSelf: "flex-start",
          opacity: busy || !selected || !agentId ? 0.5 : 1,
        }}
      >
        <Text style={{ ...quartz.type.bodySm }}>
          Prepare exact sharing preview
        </Text>
      </Button>

      {preview ? (
        <View
          style={{
            display: "flex",
            flexDirection: "column",
            gap: quartz.space[2],
            padding: quartz.space[3],
            borderRadius: quartz.radius.frame,
            borderWidth: quartz.line.hair,
            borderColor: colors.line,
            backgroundColor: colors.accentSoft,
          }}
        >
          <Text
            style={{
              ...quartz.type.bodySm,
              fontWeight: quartz.type.label.fontWeight,
              color: colors.ink,
            }}
          >
            Sharing with {preview.agent.name}
          </Text>
          <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
            {preview.estimatedBytes} bytes
            {preview.maxInputBytes === null
              ? " / no reported input limit"
              : ` / ${preview.maxInputBytes} byte limit`}
            {preview.fits ? " / fits" : " / too large"}
          </Text>
          <View
            style={{
              maxHeight: quartz.layout.overlay / 2,
              overflow: "scroll",
              padding: quartz.space[2],
              borderRadius: quartz.radius.soft,
              backgroundColor: colors.panel,
            }}
          >
            <Text
              style={{
                ...quartz.type.bodySm,
                color: colors.ink,
                whiteSpace: "normal",
              }}
            >
              {preview.text}
            </Text>
          </View>
          <Button
            pressed={shareConfirmed}
            onClick={() => setShareConfirmed((value) => !value)}
            style={{
              backgroundColor: shareConfirmed
                ? colors.successSoft
                : colors.panel,
            }}
          >
            <Text style={{ ...quartz.type.bodySm }}>
              {shareConfirmed
                ? "I reviewed the exact text to share"
                : "Confirm the exact text before sending"}
            </Text>
          </Button>
          <View
            style={{ display: "flex", flexWrap: "wrap", gap: quartz.space[2] }}
          >
            <Button
              onClick={async () => {
                await requestDraft();
              }}
              disabled={!shareConfirmed || busy || !preview.fits}
              variant="primary"
              style={{}}
            >
              <Text
                style={{
                  ...quartz.type.bodySm,
                  fontWeight: quartz.type.label.fontWeight,
                }}
              >
                {busy ? "Drafting..." : "Request reviewed draft"}
              </Text>
            </Button>
            {controller ? (
              <Button
                onClick={() => controller.abort()}
                variant="destructive"
                style={{}}
              >
                <Text style={{ ...quartz.type.bodySm, color: colors.danger }}>
                  Cancel
                </Text>
              </Button>
            ) : null}
          </View>
        </View>
      ) : null}

      {draftJson ? (
        <View
          style={{
            display: "flex",
            flexDirection: "column",
            gap: quartz.space[2],
          }}
        >
          <Labeled
            label={`Review and correct the ${kind.toUpperCase()} draft`}
            hint={`Returned by ${draftAgent}. Schema-valid output is not treated as fact. Edit any field before saving.`}
          >
            <Textarea
              value={draftJson}
              onChange={(event) => setDraftJson(event.value ?? "")}
              minRows={14}
              style={{
                paddingTop: quartz.space[2],
                fontFamily: quartz.fonts.mono,
              }}
            />
          </Labeled>
          <Button
            pressed={applyConfirmed}
            onClick={() => setApplyConfirmed((value) => !value)}
            style={{
              alignSelf: "flex-start",
              backgroundColor: applyConfirmed
                ? colors.successSoft
                : colors.panel,
            }}
          >
            <Text style={{ ...quartz.type.bodySm }}>
              {applyConfirmed
                ? "I reviewed every field in this draft"
                : "Confirm review before saving"}
            </Text>
          </Button>
          <Button
            onClick={async () => {
              await applyDraft();
            }}
            disabled={!applyConfirmed || busy}
            variant="primary"
            style={{
              alignSelf: "flex-start",
            }}
          >
            <Text
              style={{
                ...quartz.type.bodySm,
                fontWeight: quartz.type.label.fontWeight,
              }}
            >
              Save reviewed {kind.toUpperCase()}
            </Text>
          </Button>
        </View>
      ) : null}
    </View>
  );
};

// Capture keeps its preview, selection, and save state in one review surface.
// oxlint-disable-next-line react-doctor/no-giant-component, eslint/complexity
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

  const [includePaths, setIncludePaths] = useState(() =>
    project.scope.includePaths.join("\n")
  );

  const [excludePaths, setExcludePaths] = useState(() =>
    project.scope.excludePaths.join("\n")
  );

  // App keys CapturePanel by project id and version, so prop changes remount it.
  // oxlint-disable-next-line react-doctor/no-derived-useState
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
  const [errorMessage, setErrorMessage] = useState("");
  const [message, setMessage] = useState("");

  const loadCaptures = useCallback(async () => {
    setCaptures(await app.listCaptures(project.id));
  }, [app, project]);

  const inspectCapture = async (captureId: string) => {
    setViewFile(null);
    setViewCapture(await app.getCapture(captureId));
  };

  // Capture data must be loaded after mount and whenever the project changes.
  useEffect(() => {
    // oxlint-disable-next-line react/set-state-in-effect
    void loadCaptures();
  }, [loadCaptures]);

  const runPreview = async () => {
    setBusy(true);
    setErrorMessage("");
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
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  const save = async () => {
    if (!preview) {
      return;
    }

    setBusy(true);
    setErrorMessage("");
    setMessage("");

    try {
      const capture = await app.saveCapture(preview.id, { confirmed: true });
      setMessage(`Saved capture version ${capture.version}.`);
      setPreview(null);
      setConfirmed(false);
      await loadCaptures();
      await onChanged();
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  const refresh = async () => {
    setBusy(true);
    setErrorMessage("");
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
      setErrorMessage(errorText(error));
    }

    setBusy(false);
  };

  return (
    <View
      style={{ display: "flex", flexDirection: "column", gap: quartz.space[3] }}
    >
      <Notice error={errorMessage} message={message} />
      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[4],
        }}
      >
        <View style={{ flexGrow: 1 }}>
          <Labeled
            label="Include paths"
            hint="Optional. Empty means the whole repository."
          >
            <Textarea
              value={includePaths}
              onChange={(event) => setIncludePaths(event.value ?? "")}
              style={{
                paddingTop: quartz.space[2],
                fontFamily: quartz.fonts.mono,
              }}
            />
          </Labeled>
        </View>
        <View style={{ flexGrow: 1 }}>
          <Labeled label="Exclude paths" hint="One per line.">
            <Textarea
              value={excludePaths}
              onChange={(event) => setExcludePaths(event.value ?? "")}
              style={{
                paddingTop: quartz.space[2],
                fontFamily: quartz.fonts.mono,
              }}
            />
          </Labeled>
        </View>
        <View>
          <Labeled label="Limits" hint="MiB per file / capture.">
            <View
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: quartz.space[2],
              }}
            >
              <Input
                aria-label="Per-file size limit in MiB"
                value={maxFileMiB}
                onChange={(event) => setMaxFileMiB(event.value ?? "")}
                style={{ width: quartz.space[20] }}
              />
              <Input
                aria-label="Per-capture size limit in MiB"
                value={maxCaptureMiB}
                onChange={(event) => setMaxCaptureMiB(event.value ?? "")}
                style={{ width: quartz.space[20] }}
              />
            </View>
          </Labeled>
        </View>
      </View>
      <View
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: quartz.space[2],
          alignItems: "center",
        }}
      >
        <Button
          pressed={respectGitignore}
          onClick={() => setRespectGitignore((value) => !value)}
          style={{}}
        >
          <Text style={{ ...quartz.type.bodySm }}>
            {respectGitignore ? "Respect .gitignore" : "Ignore .gitignore"}
          </Text>
        </Button>
        <Button
          onClick={async () => {
            await runPreview();
          }}
          disabled={busy}
          style={{ opacity: busy ? quartz.opacity.disabled : 1 }}
        >
          <Text style={{ ...quartz.type.bodySm }}>Preview capture</Text>
        </Button>
        <Button
          onClick={async () => {
            await refresh();
          }}
          disabled={busy}
          style={{ opacity: busy ? quartz.opacity.disabled : 1 }}
        >
          <Text style={{ ...quartz.type.bodySm }}>Refresh context</Text>
        </Button>
      </View>

      {preview ? (
        <View
          style={{
            display: "flex",
            flexDirection: "column",
            gap: quartz.space[2],
          }}
        >
          <View
            style={{
              display: "flex",
              flexDirection: "column",
              gap: quartz.space[4],
            }}
          >
            <Text
              style={{
                ...quartz.type.bodySm,
                fontWeight: quartz.type.label.fontWeight,
              }}
            >
              {preview.files.length} files / {preview.totalBytes} bytes
            </Text>
            <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
              {preview.skipped.length} skipped
            </Text>
            <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
              Git:{" "}
              {preview.git
                ? `${preview.git.branch ?? "detached"}${preview.git.dirty ? " (dirty)" : " (clean)"}`
                : "not available"}
            </Text>
          </View>
          <View
            style={{
              maxHeight: quartz.layout.overlay / 2,
              overflow: "scroll",
              padding: quartz.space[2],
              borderRadius: quartz.radius.soft,
              backgroundColor: colors.code,
            }}
          >
            {preview.files.map((file) => (
              <View
                key={file.path}
                style={{
                  display: "flex",
                  flexWrap: "wrap",
                  gap: quartz.space[2],
                }}
              >
                <Text size="bodySm">{file.category}</Text>
                <Text size="codeSm">{file.path}</Text>
                <Text size="bodySm">{file.size} bytes</Text>
              </View>
            ))}
          </View>
          {preview.skipped.length > 0 ? (
            <View
              style={{
                maxHeight: quartz.layout.fieldMaxHeight,
                overflow: "scroll",
                padding: quartz.space[2],
                borderRadius: quartz.radius.soft,
                backgroundColor: colors.panel,
                borderWidth: quartz.line.hair,
                borderColor: colors.line,
              }}
            >
              {preview.skipped.map((item) => (
                <View
                  key={`${item.path}:${item.reason}`}
                  style={{
                    display: "flex",
                    flexWrap: "wrap",
                    gap: quartz.space[2],
                  }}
                >
                  <Text size="bodySm" tone="muted">
                    Skipped
                  </Text>
                  <Text size="codeSm">{item.path}</Text>
                  <Text size="bodySm" tone="muted">
                    {item.reason}
                  </Text>
                </View>
              ))}
            </View>
          ) : null}
          <Button
            pressed={confirmed}
            onClick={() => setConfirmed((value) => !value)}
            style={{
              alignSelf: "flex-start",
              backgroundColor: confirmed ? colors.successSoft : colors.panel,
            }}
          >
            <Text style={{ ...quartz.type.bodySm }}>
              {confirmed
                ? "I reviewed this exact capture"
                : "Confirm reviewed capture before saving"}
            </Text>
          </Button>
          <Button
            onClick={async () => {
              await save();
            }}
            disabled={!confirmed || busy}
            variant="primary"
            style={{
              alignSelf: "flex-start",
            }}
          >
            <Text
              style={{
                ...quartz.type.bodySm,
                fontWeight: quartz.type.label.fontWeight,
              }}
            >
              Save reviewed capture
            </Text>
          </Button>
        </View>
      ) : null}

      <View
        style={{
          display: "flex",
          flexDirection: "column",
          gap: quartz.space[2],
        }}
      >
        <Text
          style={{
            ...quartz.type.bodySm,
            fontWeight: quartz.type.label.fontWeight,
          }}
        >
          Capture history
        </Text>
        {captures.length === 0 ? (
          <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
            No captures yet.
          </Text>
        ) : (
          captures.map((capture) => (
            <View
              key={capture.id}
              style={{
                display: "flex",
                gap: quartz.space[2],
                alignItems: "center",
                padding: quartz.space[2],
                borderRadius: quartz.radius.soft,
                borderWidth: quartz.line.hair,
                borderColor: colors.line,
              }}
            >
              <View
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: quartz.space[0.5],
                  flexGrow: 1,
                }}
              >
                <Text
                  style={{
                    ...quartz.type.bodySm,
                    fontWeight: quartz.type.label.fontWeight,
                  }}
                >
                  Version {capture.version}
                </Text>
                <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
                  {capture.createdAt} / {capture.files.length} files /{" "}
                  {capture.totalBytes} bytes / profile v
                  {capture.profileSnapshot.version} / project v
                  {capture.projectSnapshot.version}
                </Text>
              </View>
              <Button
                onClick={async () => {
                  await inspectCapture(capture.id);
                }}
              >
                <Text style={{ ...quartz.type.bodySm }}>View</Text>
              </Button>
              <Button
                onClick={() => setDeleteId(capture.id)}
                variant="destructive"
                style={{}}
              >
                <Text style={{ ...quartz.type.bodySm, color: colors.danger }}>
                  Delete
                </Text>
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
                        setErrorMessage(errorText(error));
                      }

                      setBusy(false);
                    })();
                  }}
                  variant="destructive"
                  style={{}}
                >
                  <Text style={{ ...quartz.type.bodySm, color: colors.danger }}>
                    Confirm delete
                  </Text>
                </Button>
              ) : null}
            </View>
          ))
        )}
        {viewCapture ? (
          <View
            style={{
              display: "flex",
              flexDirection: "column",
              gap: quartz.space[2],
              padding: quartz.space[2],
              borderRadius: quartz.radius.soft,
              borderWidth: quartz.line.hair,
              borderColor: colors.line,
              backgroundColor: colors.code,
            }}
          >
            <View
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: quartz.space[2],
                alignItems: "center",
              }}
            >
              <Text
                style={{
                  ...quartz.type.bodySm,
                  fontWeight: quartz.type.label.fontWeight,
                  flexGrow: 1,
                }}
              >
                Capture version {viewCapture.version} details
              </Text>
              <Button
                onClick={() => {
                  setViewCapture(null);
                  setViewFile(null);
                }}
              >
                <Text style={{ ...quartz.type.bodySm }}>Close</Text>
              </Button>
            </View>
            <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
              Saved {viewCapture.createdAt} / {viewCapture.files.length} files /{" "}
              {viewCapture.totalBytes} bytes / skipped{" "}
              {viewCapture.skipped.length}
            </Text>
            <View
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: quartz.space[2],
              }}
            >
              {viewCapture.files.map((file) => (
                <Button
                  key={file.path}
                  pressed={viewFile === file.path}
                  aria-label={`View file ${file.path}`}
                  onClick={() => setViewFile(file.path)}
                  style={{
                    backgroundColor:
                      viewFile === file.path ? colors.accentSoft : colors.panel,
                  }}
                >
                  <View
                    style={{
                      display: "flex",
                      flexWrap: "wrap",
                      gap: quartz.space[2],
                    }}
                  >
                    <Text size="codeSm">{file.path}</Text>
                    <Text size="bodySm">{file.size} bytes</Text>
                    <Text size="codeSm">{file.hash.slice(0, 12)}</Text>
                  </View>
                </Button>
              ))}
            </View>
            {viewFile ? (
              <Textarea
                readOnly
                aria-label="Saved file contents"
                value={
                  viewCapture.files.find((file) => file.path === viewFile)
                    ?.content ?? ""
                }
                style={{
                  paddingTop: quartz.space[2],
                  fontFamily: quartz.fonts.mono,
                }}
              />
            ) : null}
            {viewCapture.skipped.length > 0 ? (
              <View
                style={{
                  maxHeight: quartz.layout.fieldMaxHeight,
                  overflow: "scroll",
                  display: "flex",
                  flexDirection: "column",
                  gap: quartz.space[1],
                }}
              >
                {viewCapture.skipped.map((item) => (
                  <View
                    key={`${item.path}:${item.reason}`}
                    style={{
                      display: "flex",
                      flexWrap: "wrap",
                      gap: quartz.space[2],
                    }}
                  >
                    <Text size="bodySm" tone="muted">
                      Skipped
                    </Text>
                    <Text size="codeSm">{item.path}</Text>
                    <Text size="bodySm" tone="muted">
                      {item.reason}
                    </Text>
                  </View>
                ))}
              </View>
            ) : null}
            <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
              Git:{" "}
              {viewCapture.git
                ? `${viewCapture.git.branch ?? "detached"}${viewCapture.git.dirty ? " (dirty)" : " (clean)"}`
                : "not available"}
            </Text>
          </View>
        ) : null}
      </View>
    </View>
  );
};

// The app shell keeps section routing and shared workspace state together.
// The workspace fields are refreshed together but are also edited independently by child panels.
// oxlint-disable-next-line react-doctor/no-giant-component, react-doctor/prefer-useReducer, eslint/complexity
export const App = ({ runtime }: { runtime: ApplicationRuntime }) => {
  const app = runtime.operations;
  const { width } = useWindowSize();
  const compact = width < quartz.layout.pageHeaderMax;
  const [navigationOpen, setNavigationOpen] = useState(false);
  const [profile, setProfile] = useState<DeveloperProfile | null>(null);
  const [draft, setDraft] = useState<Partial<DeveloperProfile> | null>(null);
  const [projects, setProjects] = useState<Project[]>([]);
  const [documents, setDocuments] = useState<ImportedDocument[]>([]);
  const [agents, setAgents] = useState<CodingAgentStatus[]>([]);

  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(
    null
  );

  const [section, setSection] = useState<AppSection>("Profile");
  const [errorMessage, setErrorMessage] = useState("");
  const [message, setMessage] = useState("");
  const [exportText, setExportText] = useState("");
  const [exportPath, setExportPath] = useState("");
  const [deleteProjectId, setDeleteProjectId] = useState("");
  const [deleteAll, setDeleteAll] = useState(false);
  const [loading, setLoading] = useState(true);

  const selectedProject =
    projects.find((project) => project.id === selectedProjectId) ?? null;

  const saveExportFile = async () => {
    setErrorMessage("");

    try {
      const text =
        exportText || JSON.stringify(await app.exportData(), null, 2);

      const stamp = new Date().toISOString().replaceAll(":", "-");
      const target = `${process.cwd()}/coderune-export-${stamp}.json`;

      await Bun.write(target, text);
      setExportPath(target);
      setMessage(`JSON export written to ${target}`);
    } catch (error) {
      setErrorMessage(errorText(error));
    }
  };

  const refresh = useCallback(async () => {
    setErrorMessage("");

    try {
      const [onboarding, projectList, exportBundle, agentList] =
        await Promise.all([
          app.getOnboardingState(),
          app.listProjects(),
          app.exportData(),
          app.listCodingAgents(),
        ]);

      setProfile(onboarding.profile);
      // SAFETY: getOnboardingState returns the persisted onboarding draft shape.
      setDraft(onboarding.draft as Partial<DeveloperProfile> | null);
      setProjects(projectList);
      // SAFETY: exportData returns imported documents in the ImportedDocument shape.
      setDocuments(exportBundle.documents as ImportedDocument[]);
      setAgents(agentList);
      setSelectedProjectId((current) => {
        if (current && projectList.some((project) => project.id === current)) {
          return current;
        }

        return projectList[0]?.id ?? null;
      });
    } catch (error) {
      setErrorMessage(errorText(error));
    }

    setLoading(false);
  }, [app]);

  // Initial application data intentionally seeds the component state.
  useEffect(() => {
    // oxlint-disable-next-line react/set-state-in-effect
    void refresh();
  }, [refresh]);

  const notify = (text: string) => {
    setErrorMessage("");
    setMessage(text);
  };

  if (loading) {
    return (
      <View
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: colors.bg,
        }}
      >
        <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
          Loading Coderune...
        </Text>
      </View>
    );
  }

  if (!profile) {
    return (
      <View
        style={{
          width: "100%",
          height: "100%",
          overflow: "scroll",
          backgroundColor: colors.bg,
          padding: compact ? quartz.space[4] : quartz.space[8],
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
        }}
      >
        <View
          style={{
            width: "100%",
            maxWidth: quartz.layout.container,
            display: "flex",
            flexDirection: "column",
            gap: quartz.space[4],
          }}
        >
          <View
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: quartz.space[2],
              alignItems: "center",
              paddingBottom: quartz.space[4],
            }}
          >
            <Text size="headingSm" mono>
              ₪
            </Text>
            <Text size="headingSm">Coderune</Text>
          </View>
          <PageHeader
            title="Your context starts here"
            description="Confirm a developer baseline before project work. Manual entry and reviewed CV import both work without a coding-agent connection."
          />
          {terminalWarning ? (
            <Notice error={terminalWarning} message="" />
          ) : null}
          <Notice error={errorMessage} message={message} />
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
                // SAFETY: saveOnboardingDraft returns the validated onboarding draft shape.
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
        </View>
      </View>
    );
  }

  const nav = [
    "Profile",
    "Projects",
    "Documents",
    "Agents",
    "Capture",
    "Data",
  ] satisfies AppSection[];

  const descriptions = {
    Profile:
      "Your experience, skills, and learning goals, shared across your projects.",
    Projects: "Give your coding agents the context behind the code.",
    Documents: "Local source documents, reviewed before they become context.",
    Agents: "Connect the coding assistants you already use.",
    Capture: "Review repository context and save only what you approve.",
    Data: "Export your local context or manage what you keep.",
  };

  const counts: Partial<Record<AppSection, number>> = {
    Projects: projects.length,
    Documents: documents.length,
    Agents: agents.filter((agent) => agent.availability === "ready").length,
  };

  return (
    <View
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: compact ? "column" : "row",
        backgroundColor: colors.bg,
      }}
    >
      {!compact || navigationOpen ? (
        <Sidebar
          style={
            compact
              ? {
                  width: "100%",
                  height: "auto",
                  borderRightWidth: quartz.space[0],
                  borderBottomWidth: quartz.line.hair,
                }
              : {}
          }
        >
          <View
            style={{
              display: "flex",
              alignItems: "center",
              gap: quartz.space[2],
              minHeight: quartz.layout.topbar,
              paddingLeft: quartz.space[2],
              marginBottom: quartz.space[6],
            }}
          >
            <Text size="headingSm" mono>
              ₪
            </Text>
            <Text size="headingSm">Coderune</Text>
          </View>
          <View
            style={{
              display: "flex",
              flexDirection: "column",
              gap: quartz.space[1],
            }}
          >
            {nav.map((item) => (
              <Button
                key={item}
                variant="ghost"
                pressed={section === item}
                testId={`nav-${item.toLowerCase()}`}
                aria-label={item}
                onClick={() => {
                  setSection(item);
                  setNavigationOpen(false);
                }}
                style={{
                  justifyContent: "flex-start",
                  minHeight: quartz.layout.row,
                }}
              >
                <View
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: quartz.space[2],
                  }}
                >
                  <Text size="label">{item}</Text>
                  {counts[item] === undefined ? null : (
                    <Badge>{counts[item]}</Badge>
                  )}
                </View>
              </Button>
            ))}
          </View>
          <View style={{ flexGrow: 1 }} />
          <View
            style={{
              display: "flex",
              flexDirection: "column",
              gap: quartz.space[1],
              padding: quartz.space[2],
              paddingTop: quartz.space[6],
            }}
          >
            <Text size="label">{profile.displayName}</Text>
            <Text size="bodySm" tone="muted">
              Personal workspace
            </Text>
            <Text size="bodySm" tone="muted">
              Stored on this device
            </Text>
          </View>
        </Sidebar>
      ) : null}

      <View
        style={{
          flexGrow: 1,
          minHeight: quartz.space[0],
          display: "flex",
          flexDirection: "column",
        }}
      >
        <View
          style={{
            display: "flex",
            flexShrink: quartz.space[0],
            alignItems: "center",
            gap: quartz.space[3],
            minHeight: quartz.layout.topbar,
            paddingLeft: compact ? quartz.space[4] : quartz.space[6],
            paddingRight: quartz.space[4],
            borderBottomWidth: quartz.line.hair,
            borderColor: colors.rowLine,
            backgroundColor: colors.panel,
          }}
        >
          {compact ? (
            <Button
              variant="ghost"
              testId="toggle-navigation"
              aria-label={
                navigationOpen ? "Close navigation" : "Open navigation"
              }
              onClick={() => setNavigationOpen(!navigationOpen)}
            >
              Menu
            </Button>
          ) : null}
          <Text size="bodySm" tone="muted">
            Personal workspace
          </Text>
          <Text size="bodySm" tone="muted">
            /
          </Text>
          <Text size="bodySm">{section}</Text>
          <View style={{ flexGrow: 1 }} />
          {compact ? null : <Badge>Local only</Badge>}
        </View>
        <View
          style={{
            flexGrow: 1,
            minHeight: quartz.space[0],
            overflow: "scroll",
            padding: compact ? quartz.space[4] : quartz.space[8],
          }}
        >
          <View
            style={{
              width: "100%",
              maxWidth: quartz.layout.container,
              display: "flex",
              flexDirection: "column",
              gap: quartz.space[6],
            }}
          >
            <PageHeader
              title={sectionTitle(section)}
              description={descriptions[section]}
            />
            {terminalWarning ? (
              <Notice error={terminalWarning} message="" />
            ) : null}
            <Notice error={errorMessage} message={message} />

            {section === "Profile" ? (
              <>
                <Section
                  title={profile.displayName}
                  description={`Developer profile version ${profile.version}. ${profile.experienceStage.replaceAll("-", " ")}.`}
                >
                  <View
                    style={{
                      display: "flex",
                      flexDirection: "column",
                      gap: quartz.space[2],
                    }}
                  >
                    <Text
                      style={{ ...quartz.type.bodySm, whiteSpace: "normal" }}
                    >
                      Skills: {profile.skillsStatement}
                    </Text>
                    <Text
                      style={{ ...quartz.type.bodySm, whiteSpace: "normal" }}
                    >
                      Learning goals: {profile.learningGoals.join("; ")}
                    </Text>
                    {profile.fieldSources &&
                    Object.keys(profile.fieldSources).length > 0 ? (
                      <Text
                        style={{
                          ...quartz.type.bodySm,
                          color: colors.muted,
                          whiteSpace: "normal",
                        }}
                      >
                        Some fields retain document provenance for review.
                      </Text>
                    ) : null}
                  </View>
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
                  <View
                    style={{
                      display: "flex",
                      flexWrap: "wrap",
                      gap: quartz.space[2],
                    }}
                  >
                    {projects.map((project) => (
                      <Button
                        key={project.id}
                        pressed={selectedProjectId === project.id}
                        onClick={() => {
                          setSelectedProjectId(project.id);
                          setDeleteProjectId("");
                        }}
                        style={{
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
                        <Text style={{ ...quartz.type.bodySm }}>
                          {project.name} / v{project.version}
                        </Text>
                      </Button>
                    ))}
                    <Button
                      onClick={() => {
                        setSelectedProjectId(null);
                        setDeleteProjectId("");
                      }}
                    >
                      <Text style={{ ...quartz.type.bodySm }}>New project</Text>
                    </Button>
                  </View>
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
                    <View
                      style={{
                        display: "flex",
                        flexWrap: "wrap",
                        gap: quartz.space[2],
                      }}
                    >
                      <Button
                        onClick={() => setDeleteProjectId(selectedProject.id)}
                        variant="destructive"
                        style={{}}
                      >
                        <Text
                          style={{
                            ...quartz.type.bodySm,
                            color: colors.danger,
                          }}
                        >
                          Delete project
                        </Text>
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
                                setErrorMessage(errorText(error));
                              }
                            })();
                          }}
                          variant="destructive"
                          style={{}}
                        >
                          <Text
                            style={{
                              ...quartz.type.bodySm,
                              color: colors.danger,
                            }}
                          >
                            Confirm project delete
                          </Text>
                        </Button>
                      ) : null}
                    </View>
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
                    <Text
                      style={{ ...quartz.type.bodySm, color: colors.muted }}
                    >
                      Create or select a project in Projects.
                    </Text>
                  )}
                </Section>
                <Section
                  title="Saved document text"
                  description="Original files are not copied. The reviewed extracted text remains available."
                >
                  {documents.length === 0 ? (
                    <Text
                      style={{ ...quartz.type.bodySm, color: colors.muted }}
                    >
                      No documents saved.
                    </Text>
                  ) : (
                    documents.map((document) => (
                      <View
                        key={document.id}
                        style={{
                          padding: quartz.space[2],
                          borderRadius: quartz.radius.soft,
                          borderWidth: quartz.line.hair,
                          borderColor: colors.line,
                        }}
                      >
                        <Text
                          style={{
                            ...quartz.type.bodySm,
                            fontWeight: quartz.type.label.fontWeight,
                          }}
                        >
                          {document.kind.toUpperCase()} / {document.name}
                        </Text>
                        <View
                          style={{
                            display: "flex",
                            flexWrap: "wrap",
                            gap: quartz.space[2],
                          }}
                        >
                          <Text size="bodySm" tone="muted">
                            {document.size} bytes
                          </Text>
                          <Text size="codeSm">
                            {document.hash.slice(0, 12)}
                          </Text>
                          <Text size="bodySm" tone="muted">
                            Imported {document.importedAt}
                          </Text>
                        </View>
                      </View>
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
                <View
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: quartz.space[2],
                  }}
                >
                  {agents.map((agent) => (
                    <View
                      key={agent.id}
                      style={{
                        display: "flex",
                        flexDirection: "column",
                        gap: quartz.space[2],
                        padding: quartz.space[3],
                        borderBottomWidth: quartz.line.hair,
                        borderColor: colors.rowLine,
                      }}
                    >
                      <View
                        style={{
                          display: "flex",
                          flexWrap: "wrap",
                          gap: quartz.space[2],
                          alignItems: "center",
                        }}
                      >
                        <Text
                          style={{
                            ...quartz.type.bodySm,
                            fontWeight: quartz.type.label.fontWeight,
                          }}
                        >
                          {agent.name}
                        </Text>
                        <Text
                          style={{ ...quartz.type.bodySm, color: colors.muted }}
                        >
                          {agent.availability} / verified:{" "}
                          {agent.verified ? "yes" : "no"}
                        </Text>
                      </View>
                      <Text
                        style={{
                          ...quartz.type.bodySm,
                          color: colors.muted,
                          whiteSpace: "normal",
                        }}
                      >
                        {agent.detail}
                      </Text>
                      <Text
                        style={{ ...quartz.type.bodySm, whiteSpace: "normal" }}
                      >
                        Structured output:{" "}
                        {agent.capabilities.structuredOutput
                          ? "yes"
                          : "validated JSON"}{" "}
                        / cancellation:{" "}
                        {agent.capabilities.cancellation ? "yes" : "no"} /
                        streaming: {agent.capabilities.streaming ? "yes" : "no"}{" "}
                        / isolation: {agent.capabilities.documentOnlyIsolation}
                      </Text>
                      <Text
                        style={{
                          ...quartz.type.bodySm,
                          color: colors.muted,
                          whiteSpace: "normal",
                        }}
                      >
                        {agent.capabilities.restrictions.join(" ")}
                      </Text>
                    </View>
                  ))}
                </View>
              </Section>
            ) : null}

            {section === "Capture" && selectedProject ? (
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
            ) : null}

            {section === "Capture" && !selectedProject ? (
              <Section title="Capture">
                <Text style={{ ...quartz.type.bodySm, color: colors.muted }}>
                  Create or select a project first.
                </Text>
              </Section>
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
                          setErrorMessage(errorText(error));
                        }
                      })();
                    }}
                  >
                    <Text style={{ ...quartz.type.bodySm }}>
                      Prepare export
                    </Text>
                  </Button>
                  <Button
                    onClick={async () => {
                      await saveExportFile();
                    }}
                  >
                    <Text style={{ ...quartz.type.bodySm }}>
                      Save JSON file
                    </Text>
                  </Button>
                  {exportPath ? (
                    <View
                      style={{
                        display: "flex",
                        flexWrap: "wrap",
                        gap: quartz.space[2],
                      }}
                    >
                      <Text size="bodySm" tone="muted">
                        Saved to
                      </Text>
                      <Text size="codeSm">{exportPath}</Text>
                    </View>
                  ) : null}
                  {exportText ? (
                    <Textarea
                      value={exportText}
                      aria-label="Export JSON"
                      readOnly
                      minRows={12}
                      style={{
                        paddingTop: quartz.space[2],
                        fontFamily: quartz.fonts.mono,
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
                    variant="destructive"
                    style={{
                      alignSelf: "flex-start",
                      backgroundColor: deleteAll
                        ? colors.dangerSoft
                        : colors.panel,
                    }}
                  >
                    <Text
                      style={{ ...quartz.type.bodySm, color: colors.danger }}
                    >
                      {deleteAll
                        ? "Delete everything is armed"
                        : "Delete all data"}
                    </Text>
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
                            setErrorMessage(errorText(error));
                          }
                        })();
                      }}
                      variant="destructive"
                      style={{
                        alignSelf: "flex-start",
                      }}
                    >
                      <Text
                        style={{
                          ...quartz.type.bodySm,
                          fontWeight: quartz.type.label.fontWeight,
                        }}
                      >
                        Confirm delete everything
                      </Text>
                    </Button>
                  ) : null}
                </Section>
              </>
            ) : null}
          </View>
        </View>
      </View>
    </View>
  );
};

export const mountDesktop = function mountDesktop(runtime: ApplicationRuntime) {
  return render(
    <QuartzFonts>
      <App runtime={runtime} />
    </QuartzFonts>,
    {
      title: "Coderune",
      width: 1240,
      height: 820,
    }
  );
};

if (import.meta.main) {
  const runtime = await createApplication({
    dataDir: resolveApplicationDataDir(),
  });

  process.on("exit", () => runtime.close());
  mountDesktop(runtime);
}
