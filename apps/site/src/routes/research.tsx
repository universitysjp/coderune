import { ContentSection } from "@/components/content-section";
import { Link } from "@/components/link";
import { PageHeader } from "@/components/page-header";

const evidence = [
  {
    label: "do not fully trust AI-generated code",
    source: "Sonar, 2026",
    value: "96%",
  },
  {
    label: "consistently verify AI code before committing",
    source: "Sonar, 2026",
    value: "48%",
  },
  {
    label: "believed they were faster; measured slower on complex tasks",
    source: "METR, 2025",
    value: "20%",
  },
] as const;

const questions = [
  {
    id: "RQ1",
    text: "Can workspace and task context be used to detect incorrect or unsupported assumptions in AI-generated code?",
    title: "Detect unsupported assumptions",
  },
  {
    id: "RQ2",
    text: "Can a shared-context interface improve a developer's ability to understand and verify AI-generated changes compared with a standard assistant?",
    title: "Support understanding",
  },
  {
    id: "RQ3",
    text: "How does adaptive information presentation affect perceived cognitive load, verification effort, and task completion time?",
    title: "Protect attention",
  },
  {
    id: "RQ4",
    text: "During natural waiting or idle periods, do optional context-linked activities improve attention or recall without reducing productivity?",
    title: "Support engagement",
  },
] as const;

const objectives = [
  "Extract structured assumptions from AI-generated code.",
  "Acquire project, behavioural, and conversational context locally.",
  "Verify assumptions against available evidence.",
  "Develop the shared-context human-AI alignment layer.",
  "Identify knowledge gaps from concepts used in generated code.",
  "Provide optional personalised or gamified learning activities.",
  "Evaluate reliability, comprehension, productivity, and learning outcomes.",
] as const;

const aiFailures = [
  "Wrong assumptions about dependencies, versions, or APIs",
  "Context drift between the conversation and the repository",
  "Hallucinated project facts",
  "Outputs that cannot be verified from the workspace",
];

const humanFailures = [
  "Limited understanding of why a change was generated",
  "Over-reliance on plausible output",
  "Verification fatigue from reviewing large diffs",
  "Loss of situational awareness across changes",
];

export const ResearchPage = () => (
  <div>
    <PageHeader
      description="AI coding assistants fill gaps in context with assumptions. Developers carry the burden of discovering those assumptions, usually without evidence to check them. Coderune treats this as a two-sided alignment problem."
      title="The verification gap is a context problem."
    >
      <div className="flex flex-wrap gap-4 text-sm">
        <Link className="underline underline-offset-4" to="/design">
          See the system design
        </Link>
        <Link className="underline underline-offset-4" to="/evaluation">
          Read the study plan
        </Link>
      </div>
    </PageHeader>

    <ContentSection id="verification-gap" title="The verification gap">
      <p className="max-w-[40rem]">
        Previous work has studied context-aware code generation, output
        verification, over-reliance, and programming knowledge tracing
        separately. The integration is the research gap.
      </p>

      <div className="mt-6 grid gap-3 sm:grid-cols-3">
        {evidence.map((item) => (
          <div
            className="bg-card rounded-[var(--radius-panel)] p-5"
            key={item.value}
          >
            <p className="text-primary text-[2rem] leading-none font-semibold tracking-tight">
              {item.value}
            </p>
            <p className="mt-3 text-sm leading-6">{item.label}</p>
            <p className="text-muted-foreground mt-2 text-xs">{item.source}</p>
          </div>
        ))}
      </div>
      <p className="text-muted-foreground mt-3 text-xs">
        Figures cited in the project proposal. This site describes the research
        plan, not completed results.
      </p>

      <div className="example-panel mt-8">
        <div className="example-block">
          <div className="example-label">
            <span>AI-side failures</span>
          </div>
          <ul className="mt-3 grid gap-3 text-sm leading-6">
            {aiFailures.map((failure) => (
              <li
                className="border-border border-b pb-3 last:border-b-0"
                key={failure}
              >
                {failure}
              </li>
            ))}
          </ul>
        </div>
        <div className="example-block">
          <div className="example-label">
            <span>Developer-side consequences</span>
          </div>
          <ul className="mt-3 grid gap-3 text-sm leading-6">
            {humanFailures.map((failure) => (
              <li
                className="border-border border-b pb-3 last:border-b-0"
                key={failure}
              >
                {failure}
              </li>
            ))}
          </ul>
        </div>
      </div>

      <div className="bg-card mt-6 rounded-[var(--radius-panel)] p-5">
        <p className="text-sm font-medium">The central tension</p>
        <p className="text-muted-foreground mt-2 text-sm leading-6">
          Presenting every assumption, explanation, and piece of evidence can
          create a second burden. The system has to decide what to show, how
          much to show, and when to show it without interrupting focused work.
        </p>
      </div>
    </ContentSection>

    <ContentSection id="research-questions" title="Research questions">
      <p className="max-w-[40rem]">
        The proposal asks whether a context-aware verification and alignment
        layer can improve reliability, comprehension, and learning engagement in
        AI-assisted development.
      </p>

      <div className="mt-6">
        {questions.map((question) => (
          <div
            className="border-border grid gap-2 border-b py-5 first:pt-0 last:border-b-0"
            key={question.id}
          >
            <div className="flex items-baseline gap-3">
              <span className="text-muted-foreground text-xs">
                {question.id}
              </span>
              <h3 className="text-sm font-medium">{question.title}</h3>
            </div>
            <p className="text-muted-foreground max-w-[38rem] text-sm leading-6">
              {question.text}
            </p>
          </div>
        ))}
      </div>
    </ContentSection>

    <ContentSection id="objectives" title="Objectives">
      <p className="max-w-[40rem]">
        Design, implement, and evaluate a privacy-preserving bidirectional
        shared-context framework that verifies AI assumptions and supports
        developer understanding and learning during AI-assisted software
        development.
      </p>

      <ol className="mt-6 grid gap-3 text-sm leading-6">
        {objectives.map((objective, index) => (
          <li className="flex gap-3" key={objective}>
            <span className="text-muted-foreground w-5 shrink-0 text-right">
              {index + 1}
            </span>
            {objective}
          </li>
        ))}
      </ol>

      <div className="bg-card mt-8 rounded-[var(--radius-panel)] p-5">
        <p className="text-sm leading-6">
          The novelty is not verification, retrieval, explanation, or
          gamification alone. It is the shared-context loop that connects them
          inside one coding workflow.
        </p>
        <Link
          className="mt-4 inline-flex text-sm underline underline-offset-4"
          to="/design"
        >
          How the design works
        </Link>
      </div>
    </ContentSection>
  </div>
);
