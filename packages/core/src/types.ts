export const EXPERIENCE_STAGES = [
  "new-to-coding",
  "early-career",
  "mid-career",
  "experienced",
] as const;

export type ExperienceStage = (typeof EXPERIENCE_STAGES)[number];

export const SKILL_RATINGS = ["beginner", "intermediate", "advanced"] as const;

export type SkillRating = (typeof SKILL_RATINGS)[number];

export interface SelfReportedSkill {
  topic: string;
  rating: SkillRating | null;
}

export interface EducationEntry {
  institution: string;
  qualification?: string;
  period?: string;
  details?: string;
}

export interface EmploymentEntry {
  employer: string;
  role: string;
  period?: string;
  details?: string;
}

export interface ProjectExperienceEntry {
  name: string;
  role?: string;
  period?: string;
  details?: string;
}

export interface CertificationEntry {
  name: string;
  issuer?: string;
  year?: string;
}

export interface ProfileLink {
  label: string;
  url: string;
}

export interface LearningPreferences {
  formats?: string[];
  notes?: string;
}

export type FieldSource =
  | { kind: "developer" }
  | {
      kind: "document";
      documentId: string;
      documentName: string;
      reference?: string;
    };

export interface DeveloperProfileFields {
  displayName: string;
  experienceStage: ExperienceStage;
  newToCoding: boolean;
  skillsStatement: string;
  learningGoals: string[];
  selfReportedSkills: SelfReportedSkill[];
  education: EducationEntry[];
  employment: EmploymentEntry[];
  projects: ProjectExperienceEntry[];
  certifications: CertificationEntry[];
  links: ProfileLink[];
  learningPreferences: LearningPreferences | null;
  fieldSources: Record<string, FieldSource>;
}

export interface DeveloperProfileInput {
  displayName: string;
  experienceStage: ExperienceStage;
  newToCoding: boolean;
  skillsStatement?: string;
  learningGoals: string[];
  selfReportedSkills?: SelfReportedSkill[];
  education?: EducationEntry[];
  employment?: EmploymentEntry[];
  projects?: ProjectExperienceEntry[];
  certifications?: CertificationEntry[];
  links?: ProfileLink[];
  learningPreferences?: LearningPreferences | null;
  fieldSources?: Record<string, FieldSource>;
}

export interface DeveloperProfile extends DeveloperProfileFields {
  id: string;
  version: number;
  createdAt: string;
  updatedAt: string;
}

export interface OnboardingDraft extends Partial<
  Omit<DeveloperProfileFields, "selfReportedSkills" | "learningGoals">
> {
  selfReportedSkills?: SelfReportedSkill[];
  learningGoals?: string[];
  updatedAt: string;
}

export interface OnboardingState {
  requiredComplete: boolean;
  canStartProject: boolean;
  missing: string[];
  profile: DeveloperProfile | null;
  draft: OnboardingDraft | null;
}

export const NEW_TO_CODING_STATEMENT = "New to coding; no skills listed yet.";

export const profileMissingFields = function profileMissingFields(
  fields: Partial<DeveloperProfileInput>
): string[] {
  const missing: string[] = [];

  if (!fields.displayName?.trim()) {
    missing.push("display name");
  }

  if (
    !fields.experienceStage ||
    !EXPERIENCE_STAGES.includes(fields.experienceStage)
  ) {
    missing.push("experience stage");
  }

  if (!fields.newToCoding && !fields.skillsStatement?.trim()) {
    missing.push("initial skills statement");
  }

  const goals = (fields.learningGoals ?? []).filter((goal) => goal.trim());

  if (goals.length === 0) {
    missing.push("learning goal");
  }

  return missing;
};

export const normalizeProfileFields = function normalizeProfileFields(
  input: DeveloperProfileInput
): DeveloperProfileFields {
  return {
    displayName: input.displayName.trim(),
    experienceStage: input.experienceStage,
    newToCoding: input.newToCoding,
    skillsStatement: input.newToCoding
      ? (input.skillsStatement ?? "").trim() || NEW_TO_CODING_STATEMENT
      : (input.skillsStatement ?? "").trim(),
    learningGoals: input.learningGoals.flatMap((goal) => {
      const trimmed = goal.trim();

      return trimmed ? [trimmed] : [];
    }),
    selfReportedSkills: (input.selfReportedSkills ?? [])
      .map((skill) => ({
        topic: skill.topic.trim(),
        rating: skill.rating ?? null,
      }))
      .filter((skill) => skill.topic),
    education: input.education ?? [],
    employment: input.employment ?? [],
    projects: input.projects ?? [],
    certifications: input.certifications ?? [],
    links: input.links ?? [],
    learningPreferences: input.learningPreferences ?? null,
    fieldSources: input.fieldSources ?? {},
  };
};

export interface ExportBundle {
  exportedAt: string;
  application: "Coderune";
  schemaVersion: number;
  profile: DeveloperProfile | null;
  profileVersions: DeveloperProfile[];
  projects: unknown[];
  projectVersions: unknown[];
  documents: unknown[];
  captures: unknown[];
}

export interface DocumentProvenance {
  kind: "document";
  documentId: string;
  documentName: string;
  reference?: string;
}

export type Provenance = { kind: "developer" } | DocumentProvenance;

export interface ProjectStatement {
  id: string;
  text: string;
  provenance: Provenance;
}

export interface CollectionScope {
  includePaths: string[];
  excludePaths: string[];
  respectGitignore: boolean;
}

export interface CaptureLimits {
  maxFileBytes: number;
  maxCaptureBytes: number;
}

export const DEFAULT_CAPTURE_LIMITS: CaptureLimits = {
  maxFileBytes: 1024 * 1024,
  maxCaptureBytes: 20 * 1024 * 1024,
};

export const DEFAULT_COLLECTION_SCOPE: CollectionScope = {
  includePaths: [],
  excludePaths: [],
  respectGitignore: true,
};

export interface Project {
  id: string;
  version: number;
  name: string;
  rootPath: string;
  goals: ProjectStatement[];
  purpose: ProjectStatement | null;
  businessLogic: ProjectStatement | null;
  rules: ProjectStatement[];
  constraints: ProjectStatement[];
  decisions: ProjectStatement[];
  acceptanceCriteria: ProjectStatement[];
  knowledge: ProjectStatement[];
  scope: CollectionScope;
  limits: CaptureLimits;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectInput {
  name: string;
  rootPath: string;
  goals: string[];
  purpose?: string;
  businessLogic?: string;
  rules?: string[];
  constraints?: string[];
  decisions?: string[];
  acceptanceCriteria?: string[];
  knowledge?: string[];
  scope?: Partial<CollectionScope>;
  limits?: Partial<CaptureLimits>;
}

export const projectMissingFields = function projectMissingFields(
  input: ProjectInput
): string[] {
  const missing: string[] = [];

  if (!input.name?.trim()) {
    missing.push("project name");
  }

  if (!input.rootPath?.trim()) {
    missing.push("root folder");
  }

  if (!(input.goals ?? []).some((goal) => goal.trim())) {
    missing.push("project goal");
  }

  return missing;
};

export const DEFAULT_DOCUMENT_LIMIT_BYTES = 20 * 1024 * 1024;

export type DocumentKind = "cv" | "srs";

export const SUPPORTED_DOCUMENT_EXTENSIONS = [
  ".pdf",
  ".docx",
  ".md",
  ".markdown",
  ".txt",
] as const;

export type SupportedDocumentExtension =
  (typeof SUPPORTED_DOCUMENT_EXTENSIONS)[number];

export interface ExtractedDocument {
  name: string;
  path: string;
  extension: SupportedDocumentExtension;
  kind: DocumentKind;
  hash: string;
  size: number;
  text: string;
  extractedAt: string;
  pageCount: number | null;
}

export interface ImportedDocument extends ExtractedDocument {
  id: string;
  projectId: string | null;
  importedAt: string;
}

export type DocumentImportErrorCode =
  | "unsupported-document"
  | "unreadable-document"
  | "image-only-document"
  | "document-too-large";

export interface DocumentImportError {
  code: DocumentImportErrorCode;
  message: string;
  path: string;
  manualEntryAvailable: true;
}

export type DocumentExtractionResult =
  | { ok: true; document: ExtractedDocument }
  | { ok: false; error: DocumentImportError };

export interface DocumentExtractionRequest {
  path: string;
  kind: DocumentKind;
  maxBytes?: number;
}

export const CAPTURE_FILE_CATEGORIES = [
  "manifest",
  "configuration",
  "agent-instructions",
  "readme",
  "documentation",
  "source",
  "test",
  "other",
] as const;

export type CaptureFileCategory = (typeof CAPTURE_FILE_CATEGORIES)[number];

export const CAPTURE_SKIP_REASONS = [
  "outside-scope",
  "excluded",
  "ignored",
  "environment-file",
  "private-key",
  "dependency-directory",
  "generated-output",
  "binary",
  "unsupported-type",
  "file-too-large",
  "capture-limit",
  "escapes-repository",
  "not-readable",
  "symbolic-link",
] as const;

export type CaptureSkipReason = (typeof CAPTURE_SKIP_REASONS)[number];

export interface GitState {
  head: string | null;
  branch: string | null;
  dirty: boolean;
  status: string[];
  capturedAt: string;
}

export interface CollectedFile {
  path: string;
  size: number;
  hash: string;
  category: CaptureFileCategory;
  content: string;
}

export interface SkippedFile {
  path: string;
  reason: CaptureSkipReason;
}

export interface RepositoryCollection {
  files: CollectedFile[];
  skipped: SkippedFile[];
  git: GitState | null;
  totalBytes: number;
  fingerprint: string;
}

export interface RepositoryCollectionRequest {
  rootPath: string;
  scope: CollectionScope;
  limits: CaptureLimits;
  extraExcludePaths?: string[];
}

export interface CapturePreviewFile {
  path: string;
  size: number;
  hash: string;
  category: CaptureFileCategory;
}

export interface CapturePreview {
  id: string;
  projectId: string;
  projectVersion: number;
  profileVersion: number;
  files: CapturePreviewFile[];
  skipped: SkippedFile[];
  git: GitState | null;
  totalBytes: number;
  scope: CollectionScope;
  limits: CaptureLimits;
  fingerprint: string;
  createdAt: string;
}

export type CapturedFile = CollectedFile;

export interface ContextCapture {
  id: string;
  projectId: string;
  version: number;
  fingerprint: string;
  createdAt: string;
  files: CapturedFile[];
  skipped: SkippedFile[];
  git: GitState | null;
  totalBytes: number;
  profileSnapshot: DeveloperProfile;
  projectSnapshot: Project;
}

export interface CaptureRefreshResult {
  status: "unchanged" | "changed";
  preview: CapturePreview;
  capture: ContextCapture | null;
}

export interface CaptureSummary {
  fileCount: number;
  skippedCount: number;
  totalBytes: number;
  byCategory: Record<string, number>;
  skippedByReason: Record<string, number>;
  gitAvailable: boolean;
  gitDirty: boolean;
}

export const summarizePreview = function summarizePreview(
  preview: CapturePreview
): CaptureSummary {
  const byCategory: Record<string, number> = {};

  for (const file of preview.files) {
    byCategory[file.category] = (byCategory[file.category] ?? 0) + 1;
  }

  const skippedByReason: Record<string, number> = {};

  for (const skipped of preview.skipped) {
    skippedByReason[skipped.reason] =
      (skippedByReason[skipped.reason] ?? 0) + 1;
  }

  return {
    fileCount: preview.files.length,
    skippedCount: preview.skipped.length,
    totalBytes: preview.totalBytes,
    byCategory,
    skippedByReason,
    gitAvailable: preview.git !== null,
    gitDirty: preview.git?.dirty ?? false,
  };
};
