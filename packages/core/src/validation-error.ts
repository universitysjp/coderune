import { ApplicationError } from "./errors";

export class ValidationError extends ApplicationError {
  readonly details: string[];

  constructor(message: string, details: string[] = []) {
    super("validation", message);
    this.name = "ValidationError";
    this.details = details;
  }
}
