import type {
  ContextCapture,
  DeveloperProfile,
  DeveloperProfileFields,
  DocumentExtractionRequest,
  DocumentExtractionResult,
  ImportedDocument,
  OnboardingDraft,
  Project,
  RepositoryCollection,
  RepositoryCollectionRequest,
} from "./types";

export interface OnboardingStore {
  getProfile: () => DeveloperProfile | null;
  getProfileVersions: () => DeveloperProfile[];
  saveProfile: (fields: DeveloperProfileFields) => DeveloperProfile;
  getOnboardingDraft: () => OnboardingDraft | null;
  saveOnboardingDraft: (draft: OnboardingDraft) => OnboardingDraft;
  clearOnboardingDraft: () => void;
}

export interface ProjectStore {
  createProject: (project: Project) => Project;
  updateProject: (project: Project) => Project;
  getProject: (id: string) => Project | null;
  getProjectVersions: (id: string) => Project[];
  listProjects: () => Project[];
  deleteProject: (id: string) => void;
}

export interface DocumentStore {
  saveDocument: (document: ImportedDocument) => ImportedDocument;
  getDocument: (id: string) => ImportedDocument | null;
  listDocuments: (projectId?: string | null) => ImportedDocument[];
  deleteDocument: (id: string) => void;
}

export interface CaptureStore {
  saveCapture: (capture: ContextCapture) => ContextCapture;
  getCapture: (id: string) => ContextCapture | null;
  listCaptures: (projectId: string) => ContextCapture[];
  getCaptureContent: (hash: string) => string | null;
  deleteCapture: (id: string) => void;
  deleteProjectCaptures: (projectId: string) => void;
}

export interface DataStore {
  deleteAllData: () => void;
}

export interface ContextStore
  extends
    OnboardingStore,
    ProjectStore,
    DocumentStore,
    CaptureStore,
    DataStore {
  initialize: () => Promise<void>;
  close: () => void;
}

export interface Clock {
  now: () => string;
}

export interface DocumentExtractor {
  extract: (
    request: DocumentExtractionRequest
  ) => Promise<DocumentExtractionResult>;
}

export interface RepositoryCollector {
  validateRoot: (rootPath: string) => void;
  collect: (
    request: RepositoryCollectionRequest
  ) => Promise<RepositoryCollection>;
}
