import { z } from "zod";

import { EXPERIENCE_STAGES, SKILL_RATINGS } from "./types";
import type { DocumentKind, ImportedDocument, Project } from "./types";

const educationEntrySchema = z
  .object({
    institution: z.string().min(1),
    qualification: z.string().min(1).optional(),
    period: z.string().min(1).optional(),
    details: z.string().min(1).optional(),
  })
  .strict();

const employmentEntrySchema = z
  .object({
    employer: z.string().min(1),
    role: z.string().min(1),
    period: z.string().min(1).optional(),
    details: z.string().min(1).optional(),
  })
  .strict();

const projectEntrySchema = z
  .object({
    name: z.string().min(1),
    role: z.string().min(1).optional(),
    period: z.string().min(1).optional(),
    details: z.string().min(1).optional(),
  })
  .strict();

const certificationEntrySchema = z
  .object({
    name: z.string().min(1),
    issuer: z.string().min(1).optional(),
    year: z.string().min(1).optional(),
  })
  .strict();

const profileLinkSchema = z
  .object({
    label: z.string().min(1),
    url: z.string().min(1),
  })
  .strict();

const learningPreferencesSchema = z
  .object({
    formats: z.array(z.string().min(1)).optional(),
    notes: z.string().min(1).optional(),
  })
  .strict();

export const draftSourceSchema = z
  .object({
    field: z.string().min(1),
    reference: z.string().min(1).optional(),
  })
  .strict();

const draftWarningsSchema = z.array(z.string().min(1)).default([]);

export const cvDraftSchema = z
  .object({
    kind: z.literal("cv"),
    displayName: z.string().min(1).optional(),
    experienceStage: z.enum(EXPERIENCE_STAGES).optional(),
    newToCoding: z.boolean().optional(),
    skillsStatement: z.string().min(1).optional(),
    learningGoals: z.array(z.string().min(1)).optional(),
    selfReportedSkills: z
      .array(
        z
          .object({
            topic: z.string().min(1),
            rating: z.enum(SKILL_RATINGS).nullable().optional(),
          })
          .strict()
      )
      .optional(),
    education: z.array(educationEntrySchema).optional(),
    employment: z.array(employmentEntrySchema).optional(),
    projects: z.array(projectEntrySchema).optional(),
    certifications: z.array(certificationEntrySchema).optional(),
    links: z.array(profileLinkSchema).optional(),
    learningPreferences: learningPreferencesSchema.nullable().optional(),
    sources: z.array(draftSourceSchema).default([]),
    warnings: draftWarningsSchema,
  })
  .strict();

export const srsDraftSchema = z
  .object({
    kind: z.literal("srs"),
    purpose: z.string().min(1).optional(),
    businessLogic: z.string().min(1).optional(),
    goals: z.array(z.string().min(1)).optional(),
    rules: z.array(z.string().min(1)).optional(),
    constraints: z.array(z.string().min(1)).optional(),
    decisions: z.array(z.string().min(1)).optional(),
    acceptanceCriteria: z.array(z.string().min(1)).optional(),
    knowledge: z.array(z.string().min(1)).optional(),
    sources: z.array(draftSourceSchema).default([]),
    warnings: draftWarningsSchema,
  })
  .strict();

export const agentDraftSchema = z.discriminatedUnion("kind", [
  cvDraftSchema,
  srsDraftSchema,
]);

export type CvDraft = z.infer<typeof cvDraftSchema>;

export type SrsDraft = z.infer<typeof srsDraftSchema>;

export type AgentDraft = z.infer<typeof agentDraftSchema>;

export type DraftSource = z.infer<typeof draftSourceSchema>;

export const parseAgentDraft = function parseAgentDraft(
  value: unknown
): AgentDraft {
  return agentDraftSchema.parse(value);
};

export const draftJsonSchema = function draftJsonSchema(
  kind: DocumentKind
): Record<string, unknown> {
  // SAFETY: z.toJSONSchema returns the JSON Schema shape for the selected Zod draft schema.
  return z.toJSONSchema(
    kind === "cv" ? cvDraftSchema : srsDraftSchema
  ) as Record<string, unknown>;
};

const CV_INSTRUCTIONS = `Create a structured CV draft for the developer.
Use only facts present in the source document. Do not invent or infer facts.
Return one JSON object only. Do not wrap it in Markdown.
Include a short quote or section label in sources[].reference for every field you fill.
If a fact is absent, omit that field instead of guessing.
Use exactly these fields:
kind ("cv"), displayName, experienceStage (one of ${EXPERIENCE_STAGES.join(", ")}), newToCoding (boolean), skillsStatement, learningGoals (string[]), selfReportedSkills ({topic, rating: beginner|intermediate|advanced|null}[]), education, employment, projects, certifications, links, learningPreferences, sources ({field, reference?}[]), warnings (string[]).
The developer will review and correct every field before anything is saved.`;

const SRS_INSTRUCTIONS = `Create a structured project-context draft from the software requirements document.
Use only facts present in the source document. Do not invent requirements.
Return one JSON object only. Do not wrap it in Markdown.
Include a short quote or section label in sources[].reference for every field you fill.
If a fact is absent, omit that field instead of guessing.
Use exactly these fields:
kind ("srs"), purpose, businessLogic, goals (string[]), rules (string[]), constraints (string[]), decisions (string[]), acceptanceCriteria (string[]), knowledge (string[]), sources ({field, reference?}[]), warnings (string[]).
The developer will review and correct every field before anything is saved.`;

export const buildDraftRequestText = function buildDraftRequestText(
  kind: DocumentKind,
  document: ImportedDocument,
  project: Project | null
): string {
  const instructions = kind === "cv" ? CV_INSTRUCTIONS : SRS_INSTRUCTIONS;
  const projectLine = project ? `Project: ${project.name}\n` : "";

  return `${instructions}

SOURCE METADATA
Document name: ${document.name}
Document kind: ${kind}
Document hash: ${document.hash}
${projectLine}
SOURCE DOCUMENT TEXT
${document.text}`;
};
