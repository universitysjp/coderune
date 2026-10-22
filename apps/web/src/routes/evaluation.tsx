import { createFileRoute, Link } from "@tanstack/react-router";

import { PageHeader } from "@/components/page-header";
import { SectionHeading } from "@/components/section-heading";

const method = [
  {
    step: "01",
    text: "Start from the verification gap: developers do not fully trust AI output, but consistent verification is uncommon.",
    title: "Problem identification",
  },
  {
    step: "02",
    text: "Build the context collector, assumption pipeline, verification engine, and human-alignment layer.",
    title: "Design and develop",
  },
  {
    step: "03",
    text: "Compare a baseline assistant, an assumption checklist, and context-verified assumptions with human alignment.",
    title: "Three-group study",
  },
  {
    step: "04",
    text: "Measure reliability, comprehension, productivity, cognitive load, and learning engagement.",
    title: "Evaluate",
  },
] as const;

const conditions = [
  {
    highlight: false,
    label: "Condition A",
    text: "The participant works with a standard coding assistant and no structured assumption support.",
    title: "Plain assistant",
  },
  {
    highlight: false,
    label: "Condition B",
    text: "The assistant states its assumptions, but the participant verifies them without workspace evidence.",
    title: "Assumption checklist",
  },
  {
    highlight: true,
    label: "Condition C",
    text: "Assumptions are checked against local workspace evidence and presented with verdicts and provenance.",
    title: "Context-verified",
  },
] as const;

const measures = [
  {
    text: "Assumption-related defects, unsupported claims, and task correctness.",
    title: "Reliability",
  },
  {
    text: "The participant's understanding of the generated code, changes, and evidence.",
    title: "Comprehension",
  },
  {
    text: "Task completion time and verification effort under each condition.",
    title: "Productivity",
  },
  {
    text: "Attention, recall, and participation in optional context-linked activities.",
    title: "Learning",
  },
] as const;

const parameters = [
  ["Pilot", "2–3 developers"],
  ["Main study", "15–20 participants"],
  ["Allocation", "One condition per participant"],
  ["Order", "Counterbalanced across three conditions"],
  ["Tasks", "Small controlled Python tasks with planted assumption mismatches"],
  ["Setting", "Participants' personal computers"],
] as const;

const EvaluationPage = () => (
  <div>
    <PageHeader
      description="The evaluation compares three levels of support, from a plain assistant to a context-aware verification layer. The aim is to test whether grounding assumptions in workspace evidence improves outcomes without harming productivity."
      title="A three-condition study, not a product benchmark."
    >
      <Link className="text-sm underline underline-offset-4" to="/team">
        Meet the research team
      </Link>
    </PageHeader>

    <section aria-labelledby="method" id="method">
      <SectionHeading title="Method" />
      <p className="mt-4 max-w-[40rem]">
        Design Science Research connects the identified problem to an artefact
        and an empirical evaluation.
      </p>
      <div className="example-panel">
        <div className="example-block">
          <div className="example-label">
            <span>Problem to artefact</span>
          </div>
          <ol className="mt-3 grid gap-5">
            {method.slice(0, 2).map((item) => (
              <li key={item.step}>
                <span className="text-muted-foreground text-xs">
                  {item.step}
                </span>
                <h3 className="mt-1 text-sm font-medium">{item.title}</h3>
                <p className="text-muted-foreground mt-1 text-xs leading-5">
                  {item.text}
                </p>
              </li>
            ))}
          </ol>
        </div>
        <div className="example-block">
          <div className="example-label">
            <span>Study to evaluation</span>
          </div>
          <ol className="mt-3 grid gap-5">
            {method.slice(2).map((item) => (
              <li key={item.step}>
                <span className="text-muted-foreground text-xs">
                  {item.step}
                </span>
                <h3 className="mt-1 text-sm font-medium">{item.title}</h3>
                <p className="text-muted-foreground mt-1 text-xs leading-5">
                  {item.text}
                </p>
              </li>
            ))}
          </ol>
        </div>
      </div>
    </section>

    <section aria-labelledby="conditions" id="conditions">
      <SectionHeading title="Conditions" />
      <p className="mt-4 max-w-[40rem]">
        Each condition adds one layer of support so the study can separate the
        effect of listing assumptions from the effect of verifying them.
      </p>
      <div className="mt-6 grid gap-3">
        {conditions.map((condition) => (
          <div
            className="bg-card grid gap-2 rounded-[var(--radius-panel)] p-5"
            key={condition.title}
          >
            <div className="flex flex-wrap items-baseline justify-between gap-2">
              <span className="text-muted-foreground text-xs">
                {condition.label}
              </span>
              {condition.highlight ? (
                <span className="text-primary text-xs">
                  Proposed intervention
                </span>
              ) : null}
            </div>
            <h3 className="text-sm font-medium">{condition.title}</h3>
            <p className="text-muted-foreground max-w-[38rem] text-sm leading-6">
              {condition.text}
            </p>
          </div>
        ))}
      </div>
    </section>

    <section aria-labelledby="measures" id="measures">
      <SectionHeading title="Measures" />
      <p className="mt-4 max-w-[40rem]">
        The study combines a controlled task with quantitative and qualitative
        measures. The pilot runs with 2–3 developers; the main study targets
        15–20 participants.
      </p>

      <section aria-label="Study parameters" className="table-wrap">
        <table>
          <thead>
            <tr>
              <th scope="col">Parameter</th>
              <th scope="col">Plan</th>
            </tr>
          </thead>
          <tbody>
            {parameters.map(([label, value]) => (
              <tr key={label}>
                <td>{label}</td>
                <td className="text-muted-foreground" data-label="Plan">
                  {value}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <div className="mt-6 grid gap-3 sm:grid-cols-2">
        {measures.map((measure) => (
          <div
            className="bg-card rounded-[var(--radius-panel)] p-5"
            key={measure.title}
          >
            <h3 className="text-sm font-medium">{measure.title}</h3>
            <p className="text-muted-foreground mt-2 text-sm leading-6">
              {measure.text}
            </p>
          </div>
        ))}
      </div>

      <div className="bg-card mt-6 rounded-[var(--radius-panel)] p-5">
        <p className="text-sm leading-6">
          <span className="font-medium">Scope of the claim.</span> Verification
          is constraint alignment, not a guarantee that generated code is
          correct. The study evaluates a research prototype, not a production
          deployment or a model benchmark.
        </p>
      </div>

      <p className="mt-6 text-sm">
        Evaluation is a later milestone.{" "}
        <Link className="underline underline-offset-4" to="/design">
          Read the current scope
        </Link>
        .
      </p>
    </section>
  </div>
);

export const Route = createFileRoute("/evaluation")({
  component: EvaluationPage,
});
