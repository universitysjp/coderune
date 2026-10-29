export type ApplicationErrorCode =
  | "validation"
  | "onboarding-required"
  | "not-found"
  | "unreadable-path"
  | "unsupported-document"
  | "unreadable-document"
  | "image-only-document"
  | "document-too-large"
  | "capture-stale"
  | "capture-limit"
  | "preview-expired"
  | "confirmation-required"
  | "agent-unavailable"
  | "agent-input-too-large"
  | "agent-cancelled"
  | "agent-timeout"
  | "agent-invalid-output"
  | "agent-failed";

export class ApplicationError extends Error {
  readonly code: ApplicationErrorCode;

  constructor(code: ApplicationErrorCode, message: string) {
    super(message);
    this.name = "ApplicationError";
    this.code = code;
  }
}
