import { createFileRoute, Link } from "@tanstack/react-router";

import { SectionHeading } from "@/components/section-heading";
import { SiteLogo } from "@/components/site-logo";
import { VerificationDemo } from "@/components/verification-demo";
import { site } from "@/lib/site";

const snapshot = [
  {
    label: "Git state",
    status: "Captured",
    value: "main · 3 uncommitted files",
  },
  {
    label: "Runtime",
    status: "Captured",
    value: "Node 22 · TypeScript 7",
  },
  {
    label: "Configuration",
    status: "Mismatch",
    value: "config.toml detected",
  },
  {
    label: "Deployment",
    status: "Unresolved",
    value: "Outside capture scope",
  },
] as const;

const comparison = [
  {
    bridge: "Checked against sources",
    checklist: "Stated only",
    feature: "Grounds output in workspace evidence",
    plain: "Model priors",
  },
  {
    bridge: "Verified · Contradicted · Unresolved",
    checklist: "Manual judgement",
    feature: "Shows a verdict",
    plain: "None",
  },
  {
    bridge: "Path, hash, timestamp",
    checklist: "Sometimes",
    feature: "Keeps evidence traceable",
    plain: "Rarely",
  },
  {
    bridge: "Progressive disclosure",
    checklist: "Not addressed",
    feature: "Protects attention",
    plain: "Not addressed",
  },
  {
    bridge: "Local-first",
    checklist: "Varies by tool",
    feature: "Runs against local data",
    plain: "Varies by tool",
  },
] as const;

const steps = [
  {
    text: "Developer profile, project rules, Git state, configuration, and selected files are collected locally under explicit scope controls.",
    title: "Capture with consent",
  },
  {
    text: "Each assumption is checked against captured sources. A verdict never comes from a model guess or an embedding alone.",
    title: "Verify against evidence",
  },
  {
    text: "Evidence stays on demand. The interface surfaces the smallest useful explanation instead of interrupting every change.",
    title: "Explain when useful",
  },
] as const;

const explore = [
  {
    text: "The verification gap, research questions, and project objectives.",
    title: "Research",
    to: "/research" as const,
  },
  {
    text: "The local-first architecture and the data boundary behind it.",
    title: "Design",
    to: "/design" as const,
  },
  {
    text: "The three-condition study and the outcomes it measures.",
    title: "Evaluation",
    to: "/evaluation" as const,
  },
  {
    text: "The people and institution behind the project.",
    title: "Team",
    to: "/team" as const,
  },
] as const;

const HomePage = () => (
  <div>
    <section className="home-hero">
      <SiteLogo className="hero-logo" />
      <h1 className="lede max-w-[42rem]">
        <span className="text-primary">{site.name}</span> is a research
        prototype that checks the assumptions behind AI-generated code against{" "}
        <Link to="/design">local workspace evidence</Link>. It shows what the
        assistant assumed, what the repository actually says, and where the two{" "}
        <Link to="/research">disagree</Link>.
      </h1>
    </section>

    <section aria-labelledby="how-it-works" id="how-it-works">
      <SectionHeading title="How it works" />
      <p className="mt-4 max-w-[40rem]">
        An assistant states what it assumed. Context Bridge compares those
        claims with the evidence the developer allowed it to collect, then shows
        a verdict and the source behind it.
      </p>

      <VerificationDemo />

      <p className="mt-4 max-w-[40rem]">
        The same layer carries developer preferences, project rules, capture
        scope, and provenance. Nothing is shared with an agent unless the
        developer requests it.
      </p>

      <div className="example-panel">
        <div className="example-block">
          <div className="example-label">
            <span>Context snapshot</span>
            <span className="ml-auto">Local</span>
          </div>
          <ul className="mt-3">
            {snapshot.map((item) => (
              <li
                className="border-border flex items-start justify-between gap-4 border-b py-3 first:pt-1 last:border-b-0 last:pb-0"
                key={item.label}
              >
                <div>
                  <p className="text-sm">{item.label}</p>
                  <p className="text-muted-foreground mt-0.5 text-xs">
                    {item.value}
                  </p>
                </div>
                <span className="text-muted-foreground shrink-0 text-xs">
                  {item.status}
                </span>
              </li>
            ))}
          </ul>
        </div>

        <div className="example-block">
          <div className="example-label">
            <span>What the layer does</span>
          </div>
          <ol className="mt-3 grid gap-5">
            {steps.map((step, index) => (
              <li key={step.title}>
                <span className="text-muted-foreground text-xs">
                  0{index + 1}
                </span>
                <h3 className="mt-1 text-sm leading-6 font-medium">
                  {step.title}
                </h3>
                <p className="text-muted-foreground mt-1 text-xs leading-5">
                  {step.text}
                </p>
              </li>
            ))}
          </ol>
        </div>
      </div>
    </section>

    <section aria-labelledby="why-context-bridge" id="why-context-bridge">
      <SectionHeading title="Why Context Bridge?" />
      <p className="mt-4 max-w-[40rem]">
        Listing assumptions is useful. Checking them is different. This study
        tests whether grounding them in workspace evidence improves reliability
        without adding cognitive load.
      </p>

      <section aria-label="Capability comparison" className="table-wrap">
        <table>
          <thead>
            <tr>
              <th scope="col">Capability</th>
              <th scope="col">Plain assistant</th>
              <th scope="col">Assumption list</th>
              <th scope="col">Context Bridge</th>
            </tr>
          </thead>
          <tbody>
            {comparison.map((row) => (
              <tr key={row.feature}>
                <td>{row.feature}</td>
                <td
                  className="text-muted-foreground"
                  data-label="Plain assistant"
                >
                  {row.plain}
                </td>
                <td
                  className="text-muted-foreground"
                  data-label="Assumption list"
                >
                  {row.checklist}
                </td>
                <td data-label="Context Bridge">{row.bridge}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>
    </section>

    <section aria-labelledby="current-scope" id="current-scope">
      <SectionHeading title="Current scope" />
      <p className="mt-4 max-w-[40rem]">
        The current increment covers onboarding, local context capture, storage,
        and reviewed document imports. Verification and learning follow in later
        increments.
      </p>
      <div className="bg-card mt-6 rounded-[var(--radius-panel)] p-5">
        <p className="text-sm leading-6">
          What works today: developer profile entry, project knowledge, Git and
          configuration capture, local SQLite persistence, history, export, and
          deletion. Assumption verification is the next milestone.
        </p>
        <Link
          className="mt-4 inline-flex text-sm underline underline-offset-4"
          to="/design"
        >
          Read the design
        </Link>
      </div>
    </section>

    <section aria-labelledby="explore" id="explore">
      <SectionHeading title="Explore" />
      <nav aria-label="Project pages" className="guide-links">
        {explore.map((item) => (
          <Link className="guide-link-card" key={item.to} to={item.to}>
            <h3>{item.title}</h3>
            <p>{item.text}</p>
          </Link>
        ))}
      </nav>
    </section>

    <section aria-labelledby="research-in-progress" id="research-in-progress">
      <SectionHeading title="Research in progress" />
      <p className="mt-4 max-w-[40rem]">
        Context Bridge is a final-year research project at the {site.university}
        . The{" "}
        <a
          className="underline underline-offset-4"
          href={site.proposalUrl}
          rel="noreferrer"
          target="_blank"
        >
          proposal
        </a>{" "}
        defines the problem, the system, and a three-condition evaluation. The{" "}
        <a
          className="underline underline-offset-4"
          href={site.repositoryUrl}
          rel="noreferrer"
          target="_blank"
        >
          repository
        </a>{" "}
        tracks the implementation.
      </p>
    </section>
  </div>
);

export const Route = createFileRoute("/")({
  component: HomePage,
});
