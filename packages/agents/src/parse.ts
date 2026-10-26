import { ApplicationError } from "@coderune/core";

export function parseJsonObject(value: unknown): Record<string, unknown> {
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }

  if (typeof value !== "string") {
    throw invalidOutput();
  }

  const trimmed = value.trim();

  const unfenced = trimmed
    .replace(/^```(?:json)?\s*/i, "")
    .replace(/\s*```$/, "")
    .trim();

  try {
    return parseJsonObject(JSON.parse(unfenced));
  } catch {
    const start = unfenced.indexOf("{");
    const end = unfenced.lastIndexOf("}");

    if (start !== -1 && end > start) {
      try {
        return parseJsonObject(JSON.parse(unfenced.slice(start, end + 1)));
      } catch {
        throw invalidOutput();
      }
    }

    throw invalidOutput();
  }
}

export function invalidOutput(): ApplicationError {
  return new ApplicationError(
    "agent-invalid-output",
    "The coding agent did not return a usable JSON draft."
  );
}

export function requireSuccess(
  exitCode: number,
  stderr: string,
  commandName: string
): void {
  if (exitCode === 0) {
    return;
  }

  const detail = stderr.trim();
  const lower = detail.toLowerCase();

  const message =
    lower.includes("not logged in") ||
    lower.includes("authentication") ||
    lower.includes("sign in") ||
    lower.includes("unauthorized")
      ? `${commandName} needs you to sign in using its own tool before drafting.`
      : detail
        ? `${commandName} failed: ${detail.slice(-800)}`
        : `${commandName} exited with code ${exitCode}.`;

  throw new ApplicationError(
    message.includes("sign in") ? "agent-unavailable" : "agent-failed",
    message
  );
}
