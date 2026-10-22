import { createFileRoute, Link } from "@tanstack/react-router";

import { PageHeader } from "@/components/page-header";
import { SectionHeading } from "@/components/section-heading";

const principles = [
  {
    text: "Profiles, project knowledge, captures, and evidence stay in one per-user SQLite database outside project repositories.",
    title: "Local by default",
  },
  {
    text: "Verdicts reference captured sources. Retrieved graph context can accelerate a lookup, but it does not become the source of truth.",
    title: "Evidence over memory",
  },
  {
    text: "The interface starts with a summary, then exposes evidence and explanations on demand instead of interrupting every change.",
    title: "Adaptive presentation",
  },
  {
    text: "Imported profile and project information is reviewed and corrected before it counts as confirmed context.",
    title: "Human confirmation",
  },
] as const;

const stages = [
  {
    number: "01",
    text: "Capture the developer profile, project rules, goals, scope permissions, and selected local workspace signals before code generation.",
    title: "Initialise and ingest",
  },
  {
    number: "02",
    text: "Keep versioned captures with paths, hashes, timestamps, Git state, and the evidence associated with each source.",
    title: "Store with provenance",
  },
  {
    number: "03",
    text: "Check assumptions against local evidence, then show the verdict, the evidence, and the impact without making verification a guarantee.",
    title: "Verify and explain",
  },
  {
    number: "04",
    text: "Reuse the same context to identify concepts worth revisiting and offer optional, context-linked learning at appropriate moments.",
    title: "Support learning",
  },
] as const;

const modules = [
  {
    location: "apps/app",
    responsibility:
      "GPUIX React application with native desktop and supported terminal launch modes.",
  },
  {
    location: "packages/core",
    responsibility:
      "Domain models, onboarding rules, draft schemas, capture operations, and effect interfaces.",
  },
  {
    location: "packages/collectors",
    responsibility:
      "Repository and Git capture, selected-document text extraction, scope enforcement, and hashes.",
  },
  {
    location: "packages/storage",
    responsibility:
      "Bun SQLite schema, migrations, transactions, versions, history, retention, and export reads.",
  },
  {
    location: "packages/agents",
    responsibility:
      "Adapters for Codex, Claude Code, OpenCode, and Antigravity, limited to reviewed document-draft jobs in this increment.",
  },
  {
    location: "apps/web",
    responsibility: "Public research site and project documentation.",
  },
  {
    location: "packages/tokens",
    responsibility: "Paper Mono typeface and OKLCH design tokens.",
  },
] as const;

const DesignPage = () => (
  <div>
    <PageHeader
      description="The prototype keeps the system of record local, uses workspace evidence to check AI claims, and only shares selected text when a developer explicitly requests an agent-assisted import."
      title="A local-first system of record for shared context."
    >
      <div className="flex flex-wrap gap-4 text-sm">
        <Link className="underline underline-offset-4" to="/evaluation">
          Read the evaluation plan
        </Link>
        <a
          className="underline underline-offset-4"
          href="https://github.com/universitysjp/context-bridge/blob/main/docs/data-collection-design.md"
          rel="noreferrer"
          target="_blank"
        >
          Data collection design
        </a>
      </div>
    </PageHeader>

    <section aria-labelledby="principles" id="principles">
      <SectionHeading title="Principles" />
      <p className="mt-4 max-w-[40rem]">
        Four constraints decide where data lives, what counts as evidence, and
        how much the interface shows at once.
      </p>
      <div className="mt-6 grid gap-3 sm:grid-cols-2">
        {principles.map((principle) => (
          <div
            className="bg-card rounded-[var(--radius-panel)] p-5"
            key={principle.title}
          >
            <h3 className="text-sm font-medium">{principle.title}</h3>
            <p className="text-muted-foreground mt-2 text-sm leading-6">
              {principle.text}
            </p>
          </div>
        ))}
      </div>
    </section>

    <section aria-labelledby="architecture" id="architecture">
      <SectionHeading title="Architecture" />
      <p className="mt-4 max-w-[40rem]">
        The flow starts before a coding agent runs and continues after generated
        code arrives.
      </p>
      <div className="example-panel">
        <div className="example-block">
          <div className="example-label">
            <span>Capture and store</span>
          </div>
          <ol className="mt-3 grid gap-5">
            {stages.slice(0, 2).map((stage) => (
              <li key={stage.number}>
                <span className="text-muted-foreground text-xs">
                  {stage.number}
                </span>
                <h3 className="mt-1 text-sm font-medium">{stage.title}</h3>
                <p className="text-muted-foreground mt-1 text-xs leading-5">
                  {stage.text}
                </p>
              </li>
            ))}
          </ol>
        </div>
        <div className="example-block">
          <div className="example-label">
            <span>Verify and learn</span>
          </div>
          <ol className="mt-3 grid gap-5">
            {stages.slice(2).map((stage) => (
              <li key={stage.number}>
                <span className="text-muted-foreground text-xs">
                  {stage.number}
                </span>
                <h3 className="mt-1 text-sm font-medium">{stage.title}</h3>
                <p className="text-muted-foreground mt-1 text-xs leading-5">
                  {stage.text}
                </p>
              </li>
            ))}
          </ol>
        </div>
      </div>
    </section>

    <section aria-labelledby="data-boundary" id="data-boundary">
      <SectionHeading title="Data boundary" />
      <p className="mt-4 max-w-[40rem]">
        The boundary is explicit so a local capture does not imply that the
        selected coding agent performs inference locally.
      </p>
      <div className="example-panel">
        <div className="example-block">
          <div className="example-label">
            <span>Stays on the device</span>
          </div>
          <ul className="text-muted-foreground mt-3 grid gap-3 text-sm leading-6">
            <li>Developer profile and confirmed onboarding information.</li>
            <li>Project goals, rules, constraints, and acceptance criteria.</li>
            <li>Versioned repository captures, hashes, and provenance.</li>
            <li>Extracted document text approved by the developer.</li>
            <li>Capture history, export data, and deletion controls.</li>
          </ul>
        </div>
        <div className="example-block">
          <div className="example-label">
            <span>Shared only on request</span>
          </div>
          <ul className="text-muted-foreground mt-3 grid gap-3 text-sm leading-6">
            <li>Previewed document text for a CV or SRS draft job.</li>
            <li>The selected agent and its actual capabilities.</li>
            <li>
              Agent-generated drafts, which remain unconfirmed until reviewed.
            </li>
            <li>
              Provider authentication and model choice, which stay with the
              agent rather than Context Bridge.
            </li>
          </ul>
        </div>
      </div>
      <p className="text-muted-foreground mt-4 max-w-[40rem] text-sm leading-6">
        Repository scanning and local text extraction execute no project scripts
        and initiate no external calls. Agent-assisted interpretation is a
        separate, explicitly requested operation.
      </p>
    </section>

    <section aria-labelledby="current-increment" id="current-increment">
      <SectionHeading title="Current increment" />
      <p className="mt-4 max-w-[40rem]">
        This site does not present later-increment work as complete. The first
        increment is local onboarding, capture, storage, and reviewed document
        import.
      </p>
      <div className="example-panel">
        <div className="example-block">
          <div className="example-label">
            <span>In scope</span>
          </div>
          <ul className="text-muted-foreground mt-3 grid gap-3 text-sm leading-6">
            <li>Mandatory developer onboarding with a resumable profile.</li>
            <li>Project context from optional SRS imports and manual entry.</li>
            <li>Scoped repository capture with review before save.</li>
            <li>
              Local SQLite persistence, history, refresh, export, and deletion.
            </li>
            <li>Reviewed CV and SRS draft jobs through supported agents.</li>
          </ul>
        </div>
        <div className="example-block">
          <div className="example-label">
            <span>Later increments</span>
          </div>
          <ul className="text-muted-foreground mt-3 grid gap-3 text-sm leading-6">
            <li>AST and code-graph extraction for TypeScript and Python.</li>
            <li>Assumption extraction and evidence-backed verdicts.</li>
            <li>
              Developer learning, knowledge-gap identification, and
              gamification.
            </li>
            <li>
              Interactive coding sessions and passive activity collection.
            </li>
            <li>Scanned-document OCR and broader platform support.</li>
          </ul>
        </div>
      </div>
    </section>

    <section aria-labelledby="module-scope" id="module-scope">
      <SectionHeading title="Module scope" />
      <p className="mt-4 max-w-[40rem]">
        Planned responsibilities, not a claim that every module is implemented.
      </p>
      <section aria-label="Package responsibilities" className="table-wrap">
        <table>
          <thead>
            <tr>
              <th scope="col">Location</th>
              <th scope="col">Responsibility</th>
            </tr>
          </thead>
          <tbody>
            {modules.map((module) => (
              <tr key={module.location}>
                <td>{module.location}</td>
                <td
                  className="text-muted-foreground"
                  data-label="Responsibility"
                >
                  {module.responsibility}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
      <p className="mt-4 text-sm">
        <Link className="underline underline-offset-4" to="/evaluation">
          Continue to the evaluation plan
        </Link>
      </p>
    </section>
  </div>
);

export const Route = createFileRoute("/design")({
  component: DesignPage,
});
