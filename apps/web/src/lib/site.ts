export const site = {
  degree: "BICT",
  description:
    "A privacy-preserving shared-context layer that verifies AI coding assumptions against local workspace evidence.",
  group: "Group 23",
  name: "Coderune",
  presentationUrl:
    "https://github.com/universitysjp/coderune/blob/main/docs/presentation.pdf",
  proposalUrl:
    "https://github.com/universitysjp/coderune/blob/main/docs/proposal_group_23.pdf",
  repositoryUrl: "https://github.com/universitysjp/coderune",
  university: "University of Sri Jayewardenepura",
} as const;

export const navItems = [
  { label: "Overview", to: "/" },
  { label: "Research", to: "/research" },
  { label: "Design", to: "/design" },
  { label: "Evaluation", to: "/evaluation" },
  { label: "Team", to: "/team" },
] as const;

export const pageSections = {
  "/": [
    { id: "how-it-works", label: "How it works" },
    { id: "why-coderune", label: "Why Coderune?" },
    { id: "current-scope", label: "Current scope" },
    { id: "explore", label: "Explore" },
    { id: "research-in-progress", label: "Research in progress" },
  ],
  "/design": [
    { id: "principles", label: "Principles" },
    { id: "architecture", label: "Architecture" },
    { id: "data-boundary", label: "Data boundary" },
    { id: "current-increment", label: "Current increment" },
    { id: "module-scope", label: "Module scope" },
  ],
  "/evaluation": [
    { id: "method", label: "Method" },
    { id: "conditions", label: "Conditions" },
    { id: "measures", label: "Measures" },
  ],
  "/research": [
    { id: "verification-gap", label: "The verification gap" },
    { id: "research-questions", label: "Research questions" },
    { id: "objectives", label: "Objectives" },
  ],
  "/team": [
    { id: "team", label: "Team" },
    { id: "milestones", label: "Milestones" },
  ],
} as const;

export const team = [
  {
    focus:
      "Builds the native GPUI desktop application, local capture flow, and adapters for existing coding agents.",
    index: "ICT/22/930",
    name: "Pruthivi Thejan",
    role: "Application, collection, and agent integration",
  },
  {
    focus:
      "Owns the SQLite schema, migrations, transactions, history, retention, and export boundary.",
    index: "ICT/22/867",
    name: "Nuwan Konara",
    role: "Backend, local storage, and database",
  },
  {
    focus:
      "Leads the study design, evidence evaluation, verification work, and research reporting.",
    index: "ICT/22/911",
    name: "Raviru Rathnaweera",
    role: "Research, verification, and evaluation",
  },
] as const;
