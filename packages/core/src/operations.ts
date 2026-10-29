import type {
  AgentDraftRequest,
  AgentDraftResult,
  CodingAgentGateway,
  CodingAgentId,
  CodingAgentStatus,
  DraftSharingPreview,
} from "./agents";
import { buildDraftRequestText, parseAgentDraft } from "./drafts";
import type { CvDraft, SrsDraft } from "./drafts";
import { ApplicationError, ValidationError } from "./errors";
import type {
  Clock,
  ContextStore,
  DocumentExtractor,
  RepositoryCollector,
} from "./ports";
import {
  normalizeProfileFields,
  profileMissingFields,
  DEFAULT_CAPTURE_LIMITS,
  DEFAULT_COLLECTION_SCOPE,
  projectMissingFields,
} from "./types";
import type {
  DeveloperProfile,
  DeveloperProfileFields,
  DeveloperProfileInput,
  DocumentExtractionRequest,
  DocumentExtractionResult,
  DocumentKind,
  ExportBundle,
  ExtractedDocument,
  FieldSource,
  ImportedDocument,
  OnboardingDraft,
  OnboardingState,
  Project,
  ProjectInput,
  ProjectStatement,
  CaptureLimits,
  CapturePreview,
  CapturePreviewFile,
  CaptureRefreshResult,
  ContextCapture,
  CollectionScope,
  RepositoryCollection,
} from "./types";

export interface ApplicationDependencies {
  store: ContextStore;
  documents: DocumentExtractor;
  repository: RepositoryCollector;
  agents: CodingAgentGateway;
  clock?: Clock;
}

const systemClock: Clock = {
  now: () => new Date().toISOString(),
};

export class ApplicationOperations {
  readonly #store: ContextStore;
  readonly #documents: DocumentExtractor;
  readonly #repository: RepositoryCollector;
  readonly #agents: CodingAgentGateway;
  readonly #clock: Clock;
  readonly #previews = new Map<
    string,
    {
      preview: CapturePreview;
      collection: RepositoryCollection;
      profile: DeveloperProfile;
      project: Project;
      extraExcludePaths: string[];
    }
  >();

  constructor(dependencies: ApplicationDependencies) {
    this.#store = dependencies.store;
    this.#documents = dependencies.documents;
    this.#repository = dependencies.repository;
    this.#agents = dependencies.agents;
    this.#clock = dependencies.clock ?? systemClock;
  }

  getOnboardingState(): OnboardingState {
    const profile = this.#store.getProfile();
    const draft = this.#store.getOnboardingDraft();

    if (profile) {
      return {
        requiredComplete: true,
        canStartProject: true,
        missing: [],
        profile,
        draft,
      };
    }

    const missing = profileMissingFields(draft ?? {});

    return {
      requiredComplete: false,
      canStartProject: false,
      missing,
      profile: null,
      draft,
    };
  }

  saveOnboardingDraft(input: Partial<DeveloperProfileFields>): OnboardingDraft {
    const draft: OnboardingDraft = {
      ...input,
      updatedAt: this.#clock.now(),
    };

    return this.#store.saveOnboardingDraft(draft);
  }

  confirmOnboarding(input: DeveloperProfileInput): DeveloperProfile {
    const missing = profileMissingFields(input);

    if (missing.length > 0) {
      throw new ValidationError(
        `Complete the required onboarding fields: ${missing.join(", ")}.`,
        missing
      );
    }

    const profile = this.#store.saveProfile(normalizeProfileFields(input));
    this.#store.clearOnboardingDraft();

    return profile;
  }

  getProfile(): DeveloperProfile | null {
    return this.#store.getProfile();
  }

  updateProfile(input: DeveloperProfileInput): DeveloperProfile {
    this.requireDeveloperBaseline();
    const missing = profileMissingFields(input);

    if (missing.length > 0) {
      throw new ValidationError(
        `Complete the required profile fields: ${missing.join(", ")}.`,
        missing
      );
    }

    return this.#store.saveProfile(normalizeProfileFields(input));
  }

  extractDocument(
    request: DocumentExtractionRequest
  ): Promise<DocumentExtractionResult> {
    return this.#documents.extract(request);
  }

  confirmDocumentImport(
    document: ExtractedDocument,
    options: { confirmed: boolean; projectId?: string | null }
  ): ImportedDocument {
    if (!options.confirmed) {
      throw new ApplicationError(
        "confirmation-required",
        "Review the extracted document text before saving it."
      );
    }

    const projectId = options.projectId ?? null;

    if (projectId && !this.#store.getProject(projectId)) {
      throw new ApplicationError("not-found", "Project not found.");
    }

    const imported: ImportedDocument = {
      ...document,
      id: crypto.randomUUID(),
      projectId,
      importedAt: this.#clock.now(),
    };

    return this.#store.saveDocument(imported);
  }

  createProject(input: ProjectInput): Project {
    this.requireDeveloperBaseline();
    this.#validateProjectInput(input);

    const now = this.#clock.now();

    const project: Project = {
      id: crypto.randomUUID(),
      version: 1,
      ...ApplicationOperations.#normalizeProjectFields(input),
      createdAt: now,
      updatedAt: now,
    };

    return this.#store.createProject(project);
  }

  updateProject(id: string, input: ProjectInput): Project {
    this.requireDeveloperBaseline();
    const current = this.#store.getProject(id);

    if (!current) {
      throw new ApplicationError("not-found", "Project not found.");
    }

    this.#validateProjectInput(input);

    const project: Project = {
      ...current,
      ...ApplicationOperations.#normalizeProjectFields(input, current),
      version: current.version + 1,
      updatedAt: this.#clock.now(),
    };

    return this.#store.updateProject(project);
  }

  listProjects(): Project[] {
    this.requireDeveloperBaseline();

    return this.#store.listProjects();
  }

  getProject(id: string): Project | null {
    return this.#store.getProject(id);
  }

  deleteProject(id: string): void {
    this.requireDeveloperBaseline();

    if (!this.#store.getProject(id)) {
      throw new ApplicationError("not-found", "Project not found.");
    }

    this.#store.deleteProject(id);
  }

  listCodingAgents(): CodingAgentStatus[] {
    return this.#agents.listAgents();
  }

  prepareCvDraft(
    documentId: string,
    agentId: CodingAgentId
  ): Promise<DraftSharingPreview> {
    return this.#prepareDraft(documentId, agentId, "cv", null);
  }

  prepareSrsDraft(
    documentId: string,
    agentId: CodingAgentId,
    projectId: string
  ): Promise<DraftSharingPreview> {
    const project = this.#store.getProject(projectId);

    if (!project) {
      throw new ApplicationError("not-found", "Project not found.");
    }

    return this.#prepareDraft(documentId, agentId, "srs", project);
  }

  async requestCvDraft(
    documentId: string,
    agentId: CodingAgentId,
    options: {
      confirmed: boolean;
      signal?: AbortSignal;
      onProgress?: (message: string) => void;
    }
  ): Promise<AgentDraftResult> {
    const preview = await this.prepareCvDraft(documentId, agentId);

    return this.#requestDraft(preview, options);
  }

  async requestSrsDraft(
    documentId: string,
    agentId: CodingAgentId,
    projectId: string,
    options: {
      confirmed: boolean;
      signal?: AbortSignal;
      onProgress?: (message: string) => void;
    }
  ): Promise<AgentDraftResult> {
    const preview = await this.prepareSrsDraft(documentId, agentId, projectId);

    return this.#requestDraft(preview, options);
  }

  applyCvDraft(
    documentId: string,
    value: unknown,
    options: { confirmed: boolean }
  ): DeveloperProfile {
    if (!options.confirmed) {
      throw new ApplicationError(
        "confirmation-required",
        "Review the CV draft and confirm the fields before saving them."
      );
    }

    const document = this.#draftDocument(documentId, "cv");
    const draft = ApplicationOperations.#parseDraft(value, "cv") as CvDraft;
    const existing = this.#store.getProfile();
    const patch: Partial<DeveloperProfileInput> = {};

    if (draft.displayName !== undefined) {
      patch.displayName = draft.displayName;
    }

    if (draft.experienceStage !== undefined) {
      patch.experienceStage = draft.experienceStage;
    }

    if (draft.newToCoding !== undefined) {
      patch.newToCoding = draft.newToCoding;
    }

    if (draft.skillsStatement !== undefined) {
      patch.skillsStatement = draft.skillsStatement;
    }

    if (draft.learningGoals !== undefined) {
      patch.learningGoals = draft.learningGoals;
    }

    if (draft.selfReportedSkills !== undefined) {
      patch.selfReportedSkills = draft.selfReportedSkills.map((skill) => ({
        topic: skill.topic,
        rating: skill.rating ?? null,
      }));
    }

    if (draft.education !== undefined) {
      patch.education = draft.education;
    }

    if (draft.employment !== undefined) {
      patch.employment = draft.employment;
    }

    if (draft.projects !== undefined) {
      patch.projects = draft.projects;
    }

    if (draft.certifications !== undefined) {
      patch.certifications = draft.certifications;
    }

    if (draft.links !== undefined) {
      patch.links = draft.links;
    }

    if (draft.learningPreferences !== undefined) {
      patch.learningPreferences = draft.learningPreferences;
    }

    const merged: Partial<DeveloperProfileInput> = {
      ...existing,
      ...patch,
      fieldSources: {
        ...existing?.fieldSources,
        ...ApplicationOperations.#draftFieldSources(draft, document),
      },
    };

    const missing = profileMissingFields(merged);

    if (missing.length > 0) {
      throw new ValidationError(
        `Complete the required profile fields: ${missing.join(", ")}.`,
        missing
      );
    }

    if (!merged.displayName || !merged.experienceStage) {
      throw new ValidationError("Complete the required profile fields.");
    }

    return this.#store.saveProfile(
      normalizeProfileFields({
        displayName: merged.displayName,
        experienceStage: merged.experienceStage,
        newToCoding: merged.newToCoding ?? false,
        skillsStatement: merged.skillsStatement,
        learningGoals: merged.learningGoals ?? [],
        selfReportedSkills: merged.selfReportedSkills,
        education: merged.education,
        employment: merged.employment,
        projects: merged.projects,
        certifications: merged.certifications,
        links: merged.links,
        learningPreferences: merged.learningPreferences,
        fieldSources: merged.fieldSources,
      })
    );
  }

  applySrsDraft(
    projectId: string,
    documentId: string,
    value: unknown,
    options: { confirmed: boolean }
  ): Project {
    if (!options.confirmed) {
      throw new ApplicationError(
        "confirmation-required",
        "Review the SRS draft and confirm the fields before saving them."
      );
    }

    this.requireDeveloperBaseline();
    const project = this.#store.getProject(projectId);

    if (!project) {
      throw new ApplicationError("not-found", "Project not found.");
    }

    const document = this.#draftDocument(documentId, "srs");

    if (document.projectId && document.projectId !== projectId) {
      throw new ValidationError(
        "The selected SRS belongs to a different project."
      );
    }

    const draft = ApplicationOperations.#parseDraft(value, "srs") as SrsDraft;

    const sources = new Map(
      draft.sources.map((source) => [
        source.field.split(".")[0] ?? source.field,
        source.reference,
      ])
    );

    const statement = (text: string, field: string): ProjectStatement => ({
      id: crypto.randomUUID(),
      text: text.trim(),
      provenance: {
        kind: "document",
        documentId: document.id,
        documentName: document.name,
        reference: sources.get(field),
      },
    });

    const next: Project = {
      ...project,
      version: project.version + 1,
      updatedAt: this.#clock.now(),
    };

    if (draft.purpose !== undefined) {
      next.purpose = statement(draft.purpose, "purpose");
    }

    if (draft.businessLogic !== undefined) {
      next.businessLogic = statement(draft.businessLogic, "businessLogic");
    }

    if (draft.goals && draft.goals.length > 0) {
      next.goals = draft.goals.map((goal) => statement(goal, "goals"));
    }

    if (draft.rules !== undefined) {
      next.rules = draft.rules.map((rule) => statement(rule, "rules"));
    }

    if (draft.constraints !== undefined) {
      next.constraints = draft.constraints.map((constraint) =>
        statement(constraint, "constraints")
      );
    }

    if (draft.decisions !== undefined) {
      next.decisions = draft.decisions.map((decision) =>
        statement(decision, "decisions")
      );
    }

    if (draft.acceptanceCriteria !== undefined) {
      next.acceptanceCriteria = draft.acceptanceCriteria.map((criterion) =>
        statement(criterion, "acceptanceCriteria")
      );
    }

    if (draft.knowledge !== undefined) {
      next.knowledge = draft.knowledge.map((item) =>
        statement(item, "knowledge")
      );
    }

    if (next.goals.length === 0) {
      throw new ValidationError("A project needs at least one goal.");
    }

    return this.#store.updateProject(next);
  }

  #validateProjectInput(input: ProjectInput): void {
    const missing = projectMissingFields(input);

    if (missing.length > 0) {
      throw new ValidationError(
        `Complete the required project fields: ${missing.join(", ")}.`,
        missing
      );
    }

    try {
      this.#repository.validateRoot(input.rootPath);
    } catch {
      throw new ApplicationError(
        "unreadable-path",
        "The project root must be a readable folder."
      );
    }
  }

  static #normalizeProjectFields(
    input: ProjectInput,
    existing?: Project
  ): Omit<Project, "id" | "version" | "createdAt" | "updatedAt"> {
    return {
      name: input.name.trim(),
      rootPath: input.rootPath,
      goals: ApplicationOperations.#statements(input.goals, existing?.goals),
      purpose: ApplicationOperations.#statement(
        input.purpose,
        existing?.purpose ?? undefined
      ),
      businessLogic: ApplicationOperations.#statement(
        input.businessLogic,
        existing?.businessLogic ?? undefined
      ),
      rules: ApplicationOperations.#statements(input.rules, existing?.rules),
      constraints: ApplicationOperations.#statements(
        input.constraints,
        existing?.constraints
      ),
      decisions: ApplicationOperations.#statements(
        input.decisions,
        existing?.decisions
      ),
      acceptanceCriteria: ApplicationOperations.#statements(
        input.acceptanceCriteria,
        existing?.acceptanceCriteria
      ),
      knowledge: ApplicationOperations.#statements(
        input.knowledge,
        existing?.knowledge
      ),
      scope: ApplicationOperations.#scope(input.scope),
      limits: ApplicationOperations.#limits(input.limits),
    };
  }

  static #statement(
    text: string | undefined,
    existing?: ProjectStatement
  ): ProjectStatement | null {
    const trimmed = text?.trim();

    if (!trimmed) {
      return null;
    }

    if (existing?.text.trim() === trimmed) {
      return existing;
    }

    return {
      id: crypto.randomUUID(),
      text: trimmed,
      provenance: { kind: "developer" },
    };
  }

  static #statements(
    texts: string[] | undefined,
    existing: ProjectStatement[] = []
  ): ProjectStatement[] {
    const preserved = new Map(
      existing.map((statement) => [statement.text.trim(), statement])
    );

    return (texts ?? []).flatMap((text) => {
      const statement = ApplicationOperations.#statement(
        text,
        preserved.get(text.trim())
      );

      return statement ? [statement] : [];
    });
  }

  static #scope(scope: Partial<CollectionScope> | undefined): CollectionScope {
    return {
      includePaths: ApplicationOperations.#uniquePaths(
        scope?.includePaths ?? DEFAULT_COLLECTION_SCOPE.includePaths
      ),
      excludePaths: ApplicationOperations.#uniquePaths(
        scope?.excludePaths ?? DEFAULT_COLLECTION_SCOPE.excludePaths
      ),
      respectGitignore:
        scope?.respectGitignore ?? DEFAULT_COLLECTION_SCOPE.respectGitignore,
    };
  }

  static #limits(limits: Partial<CaptureLimits> | undefined): CaptureLimits {
    const maxFileBytes =
      limits?.maxFileBytes ?? DEFAULT_CAPTURE_LIMITS.maxFileBytes;

    const maxCaptureBytes =
      limits?.maxCaptureBytes ?? DEFAULT_CAPTURE_LIMITS.maxCaptureBytes;

    if (
      !Number.isFinite(maxFileBytes) ||
      !Number.isFinite(maxCaptureBytes) ||
      maxFileBytes <= 0 ||
      maxCaptureBytes <= 0
    ) {
      throw new ValidationError("Capture limits must be positive byte counts.");
    }

    return { maxFileBytes, maxCaptureBytes };
  }

  static #uniquePaths(paths: string[]): string[] {
    return [...new Set(paths.map((path) => path.trim()).filter(Boolean))];
  }

  async previewCapture(
    projectId: string,
    overrides: {
      scope?: Partial<CollectionScope>;
      limits?: Partial<CaptureLimits>;
      excludePaths?: string[];
    } = {}
  ): Promise<CapturePreview> {
    const profile = this.requireDeveloperBaseline();
    const project = this.#store.getProject(projectId);

    if (!project) {
      throw new ApplicationError("not-found", "Project not found.");
    }

    const scope: CollectionScope = {
      includePaths: overrides.scope?.includePaths ?? project.scope.includePaths,
      excludePaths: ApplicationOperations.#uniquePaths([
        ...(overrides.scope?.excludePaths ?? project.scope.excludePaths),
      ]),
      respectGitignore:
        overrides.scope?.respectGitignore ?? project.scope.respectGitignore,
    };

    const limits = ApplicationOperations.#limits({
      ...project.limits,
      ...overrides.limits,
    });
    const extraExcludePaths = ApplicationOperations.#uniquePaths(
      overrides.excludePaths ?? []
    );

    const collection = await this.#repository.collect({
      rootPath: project.rootPath,
      scope,
      limits,
      extraExcludePaths,
    });

    const preview: CapturePreview = {
      id: crypto.randomUUID(),
      projectId,
      projectVersion: project.version,
      profileVersion: profile.version,
      files: collection.files.map((file): CapturePreviewFile => ({
        path: file.path,
        size: file.size,
        hash: file.hash,
        category: file.category,
      })),
      skipped: collection.skipped,
      git: collection.git,
      totalBytes: collection.totalBytes,
      scope,
      limits,
      fingerprint: ApplicationOperations.#captureFingerprint(
        collection.fingerprint,
        profile,
        project
      ),
      createdAt: this.#clock.now(),
    };

    this.#previews.set(preview.id, {
      preview,
      collection,
      profile,
      project,
      extraExcludePaths,
    });

    return preview;
  }

  async saveCapture(
    previewId: string,
    options: { confirmed: boolean }
  ): Promise<ContextCapture> {
    if (!options.confirmed) {
      throw new ApplicationError(
        "confirmation-required",
        "Confirm the reviewed capture before saving it."
      );
    }

    const entry = this.#previews.get(previewId);

    if (!entry) {
      throw new ApplicationError(
        "preview-expired",
        "This capture preview is no longer available. Create a fresh preview."
      );
    }

    const profile = this.requireDeveloperBaseline();
    const project = this.#store.getProject(entry.preview.projectId);

    if (!project) {
      throw new ApplicationError("not-found", "Project not found.");
    }

    const currentCollection = await this.#repository.collect({
      rootPath: project.rootPath,
      scope: entry.preview.scope,
      limits: entry.preview.limits,
      extraExcludePaths: entry.extraExcludePaths,
    });

    const currentFingerprint = ApplicationOperations.#captureFingerprint(
      currentCollection.fingerprint,
      profile,
      project
    );

    if (currentFingerprint !== entry.preview.fingerprint) {
      this.#previews.delete(previewId);
      throw new ApplicationError(
        "capture-stale",
        "The reviewed content changed. Create a fresh preview before saving."
      );
    }

    const existing = this.#store.listCaptures(project.id)[0];

    if (existing?.fingerprint === entry.preview.fingerprint) {
      this.#previews.delete(previewId);

      return existing;
    }

    const capture: ContextCapture = {
      id: crypto.randomUUID(),
      projectId: project.id,
      version: (existing?.version ?? 0) + 1,
      fingerprint: entry.preview.fingerprint,
      createdAt: this.#clock.now(),
      files: entry.collection.files,
      skipped: entry.collection.skipped,
      git: entry.collection.git,
      totalBytes: entry.collection.totalBytes,
      profileSnapshot: entry.profile,
      projectSnapshot: entry.project,
    };

    const saved = this.#store.saveCapture(capture);
    this.#previews.delete(previewId);

    return saved;
  }

  async refreshProjectContext(
    projectId: string
  ): Promise<CaptureRefreshResult> {
    const preview = await this.previewCapture(projectId);
    const latest = this.#store.listCaptures(projectId)[0] ?? null;

    if (latest?.fingerprint === preview.fingerprint) {
      return { status: "unchanged", preview, capture: latest };
    }

    return { status: "changed", preview, capture: null };
  }

  listCaptures(projectId: string): ContextCapture[] {
    return this.#store.listCaptures(projectId);
  }

  getCapture(id: string): ContextCapture | null {
    return this.#store.getCapture(id);
  }

  deleteCapture(id: string): void {
    if (!this.#store.getCapture(id)) {
      throw new ApplicationError("not-found", "Capture not found.");
    }

    this.#store.deleteCapture(id);
  }

  deleteAllData(): void {
    this.#store.deleteAllData();
    this.#previews.clear();
  }

  static #captureFingerprint(
    repositoryFingerprint: string,
    profile: DeveloperProfile,
    project: Project
  ): string {
    return new Bun.CryptoHasher("sha256")
      .update(
        JSON.stringify({
          repository: repositoryFingerprint,
          profileVersion: profile.version,
          profileUpdatedAt: profile.updatedAt,
          projectVersion: project.version,
          projectUpdatedAt: project.updatedAt,
        })
      )
      .digest("hex");
  }

  exportData(): ExportBundle {
    return {
      exportedAt: this.#clock.now(),
      application: "Coderune",
      schemaVersion: 1,
      profile: this.#store.getProfile(),
      profileVersions: this.#store.getProfileVersions(),
      projects: this.#store.listProjects(),
      projectVersions: this.#store
        .listProjects()
        .flatMap((project) => this.#store.getProjectVersions(project.id)),
      documents: this.#store.listDocuments(),
      captures: this.#store
        .listProjects()
        .flatMap((project) => this.#store.listCaptures(project.id)),
    };
  }

  async #prepareDraft(
    documentId: string,
    agentId: CodingAgentId,
    kind: DocumentKind,
    project: Project | null
  ): Promise<DraftSharingPreview> {
    if (kind === "srs") {
      this.requireDeveloperBaseline();
    }

    const document = this.#draftDocument(documentId, kind);
    const agent = await this.#agentStatus(agentId);
    const text = buildDraftRequestText(kind, document, project);
    const estimatedBytes = new TextEncoder().encode(text).byteLength;
    const { maxInputBytes } = agent.capabilities;

    return {
      agent,
      kind,
      document,
      project,
      text,
      estimatedBytes,
      maxInputBytes,
      fits: maxInputBytes === null || estimatedBytes <= maxInputBytes,
      createdAt: this.#clock.now(),
    };
  }

  async #requestDraft(
    preview: DraftSharingPreview,
    options: {
      confirmed: boolean;
      signal?: AbortSignal;
      onProgress?: (message: string) => void;
    }
  ): Promise<AgentDraftResult> {
    if (!options.confirmed) {
      throw new ApplicationError(
        "confirmation-required",
        "Review the exact text to be shared before requesting a draft."
      );
    }

    if (!preview.fits) {
      throw new ApplicationError(
        "agent-input-too-large",
        `${preview.agent.name} accepts at most ${preview.maxInputBytes} bytes. Narrow the selected document before trying again.`
      );
    }

    const request: AgentDraftRequest = {
      agentId: preview.agent.id,
      kind: preview.kind,
      text: preview.text,
      document: {
        id: preview.document.id,
        name: preview.document.name,
        path: preview.document.path,
        hash: preview.document.hash,
      },
      project: preview.project
        ? { id: preview.project.id, name: preview.project.name }
        : null,
    };

    let value: unknown;

    options.onProgress?.("Sending the reviewed text to the selected agent...");

    try {
      value = await this.#agents.draft(request, {
        signal: options.signal,
        onProgress: options.onProgress,
      });
    } catch (error) {
      if (error instanceof ApplicationError) {
        throw error;
      }

      if (
        error instanceof Error &&
        (error.name === "AbortError" || options.signal?.aborted)
      ) {
        throw new ApplicationError(
          "agent-cancelled",
          "The coding agent draft was cancelled."
        );
      }

      throw new ApplicationError(
        "agent-failed",
        error instanceof Error
          ? error.message
          : "The coding agent did not complete the draft."
      );
    }

    options.onProgress?.("Draft received. Validating structured output...");

    return {
      agent: preview.agent,
      draft: ApplicationOperations.#parseDraft(value, preview.kind),
    };
  }

  static #parseDraft(value: unknown, kind: DocumentKind): CvDraft | SrsDraft {
    let draft: CvDraft | SrsDraft;

    try {
      draft = parseAgentDraft(value);
    } catch {
      throw new ApplicationError(
        "agent-invalid-output",
        "The coding agent returned output that did not match the draft schema."
      );
    }

    if (draft.kind !== kind) {
      throw new ApplicationError(
        "agent-invalid-output",
        `The coding agent returned a ${draft.kind} draft instead of a ${kind} draft.`
      );
    }

    return draft;
  }

  #draftDocument(documentId: string, kind: DocumentKind): ImportedDocument {
    const document = this.#store.getDocument(documentId);

    if (!document) {
      throw new ApplicationError("not-found", "Document not found.");
    }

    if (document.kind !== kind) {
      throw new ValidationError(
        `Select an imported ${kind.toUpperCase()} document for this draft.`
      );
    }

    return document;
  }

  async #agentStatus(agentId: CodingAgentId): Promise<CodingAgentStatus> {
    const agent = await this.#agents.getAgent(agentId);

    if (!agent) {
      throw new ApplicationError(
        "agent-unavailable",
        "The selected coding agent is not available. No other agent was used."
      );
    }

    if (agent.availability !== "ready") {
      throw new ApplicationError(
        "agent-unavailable",
        `${agent.name}: ${agent.detail} No other agent was used.`
      );
    }

    return agent;
  }

  static #draftFieldSources(
    draft: CvDraft | SrsDraft,
    document: ImportedDocument
  ): Record<string, FieldSource> {
    const sources: Record<string, FieldSource> = {};

    for (const source of draft.sources) {
      const field = source.field.split(".")[0] ?? source.field;
      sources[field] = {
        kind: "document",
        documentId: document.id,
        documentName: document.name,
        reference: source.reference,
      };
    }

    return sources;
  }

  requireDeveloperBaseline(): DeveloperProfile {
    const profile = this.#store.getProfile();

    if (!profile) {
      throw new ApplicationError(
        "onboarding-required",
        "Confirm the developer baseline before starting project work."
      );
    }

    return profile;
  }
}
