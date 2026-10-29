import { ApplicationError } from "@coderune/core";
import type { JsonObject, JsonValue } from "@coderune/core";
import { z } from "zod";

const stringSchema = z.string();

export const jsonValueSchema: z.ZodType<JsonValue> = z.lazy(() =>
  z.union([
    z.string(),
    z.number(),
    z.boolean(),
    z.null(),
    z.array(jsonValueSchema),
    z.record(z.string(), jsonValueSchema),
  ])
);

export const jsonObjectSchema: z.ZodType<JsonObject> = z.record(
  z.string(),
  jsonValueSchema
);

export const invalidOutput = function invalidOutput(): ApplicationError {
  return new ApplicationError(
    "agent-invalid-output",
    "The coding agent did not return a usable JSON draft."
  );
};

export const parseJsonString = function parseJsonString(
  value: JsonValue | undefined
): string | null {
  const parsed = stringSchema.safeParse(value);

  return parsed.success ? parsed.data : null;
};

export const parseJsonObject = function parseJsonObject(
  value: JsonValue
): JsonObject {
  const direct = jsonObjectSchema.safeParse(value);

  if (direct.success) {
    return direct.data;
  }

  const parsedString = stringSchema.safeParse(value);

  if (!parsedString.success) {
    throw invalidOutput();
  }

  const trimmed = parsedString.data.trim();

  const unfenced = trimmed
    .replace(/^```(?:json)?\s*/iu, "")
    .replace(/\s*```$/u, "")
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
};

export const requireSuccess = function requireSuccess(
  exitCode: number,
  stderr: string,
  commandName: string
): void {
  if (exitCode === 0) {
    return;
  }

  const detail = stderr.trim();
  const lower = detail.toLowerCase();

  let message: string;

  if (
    lower.includes("not logged in") ||
    lower.includes("authentication") ||
    lower.includes("sign in") ||
    lower.includes("unauthorized")
  ) {
    message = `${commandName} needs you to sign in using its own tool before drafting.`;
  } else if (detail) {
    message = `${commandName} failed: ${detail.slice(-800)}`;
  } else {
    message = `${commandName} exited with code ${exitCode}.`;
  }

  throw new ApplicationError(
    message.includes("sign in") ? "agent-unavailable" : "agent-failed",
    message
  );
};
