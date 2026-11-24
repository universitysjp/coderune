import type { JSX } from "react";

type Verdict = "Verified" | "Contradicted" | "Unresolved";

const CheckIcon = () => (
  <svg
    aria-hidden="true"
    fill="none"
    height="12"
    viewBox="0 0 12 12"
    width="12"
  >
    <path
      d="m2.5 6.2 2.2 2.2 4.8-4.8"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.6"
    />
  </svg>
);

const AlertIcon = () => (
  <svg
    aria-hidden="true"
    fill="none"
    height="12"
    viewBox="0 0 12 12"
    width="12"
  >
    <path
      d="M6 1.5v5.2M6 9.4v1"
      stroke="currentColor"
      strokeLinecap="round"
      strokeWidth="1.6"
    />
  </svg>
);

const QuestionIcon = () => (
  <svg
    aria-hidden="true"
    fill="none"
    height="12"
    viewBox="0 0 12 12"
    width="12"
  >
    <path
      d="M4.2 4.1a1.8 1.8 0 1 1 2.2 1.8c-.3.1-.4.4-.4.7v.4"
      stroke="currentColor"
      strokeLinecap="round"
      strokeWidth="1.4"
    />
    <circle cx="6" cy="9.3" fill="currentColor" r="0.8" />
  </svg>
);

const verdictStyles: Record<
  Verdict,
  { badge: string; icon: () => JSX.Element; label: string }
> = {
  Contradicted: {
    badge: "bg-foreground text-background",
    icon: AlertIcon,
    label: "Conflicts with captured evidence",
  },
  Unresolved: {
    badge: "border border-border text-muted-foreground",
    icon: QuestionIcon,
    label: "Not enough evidence in scope",
  },
  Verified: {
    badge: "bg-primary text-primary-foreground",
    icon: CheckIcon,
    label: "Supported by captured evidence",
  },
};

const assumptions: {
  claim: string;
  detail: string;
  verdict: Verdict;
}[] = [
  {
    claim: "Configuration lives in config.json at the repository root.",
    detail:
      "Capture found config.toml and a config/ directory. No config.json.",
    verdict: "Contradicted",
  },
  {
    claim: "The project runs on Node.js 22.",
    detail: "package.json declares node >=22 and .nvmrc pins 22.",
    verdict: "Verified",
  },
  {
    claim: "TLS is configured for production.",
    detail: "Deployment configuration is outside the permitted capture scope.",
    verdict: "Unresolved",
  },
];

const source = `const config = await loadConfig(projectRoot);

return startServer({
  port: config.port,
  tls: config.tls,
});`;

export const VerificationDemo = () => (
  <div className="example-panel">
    <div className="example-block">
      <div className="example-label">
        <span>AI output</span>
        <span className="ml-auto">2 files changed</span>
      </div>
      <pre className="example-code">
        <code>{source}</code>
      </pre>
      <p className="text-muted-foreground mt-4 text-xs leading-5">
        Assumptions are requested alongside the change, then checked one by one
        against the evidence the developer allowed the app to capture.
      </p>
    </div>

    <div className="example-block">
      <div className="example-label">
        <span>Workspace evidence</span>
        <span className="ml-auto">Local capture</span>
      </div>
      <ul className="assumption-list mt-3">
        {assumptions.map((assumption) => {
          const style = verdictStyles[assumption.verdict];
          const VerdictIcon = style.icon;

          return (
            <li key={assumption.claim}>
              <div className="flex flex-wrap items-start justify-between gap-x-4 gap-y-2">
                <p className="min-w-0 flex-1 basis-48 text-sm leading-6">
                  {assumption.claim}
                </p>
                <span
                  className={`verdict-badge ${style.badge}`}
                  title={style.label}
                >
                  <VerdictIcon />
                  {assumption.verdict}
                </span>
              </div>
              <p className="text-muted-foreground mt-1.5 text-xs leading-5">
                {assumption.detail}
              </p>
            </li>
          );
        })}
      </ul>
      <p className="text-muted-foreground mt-auto pt-3 text-xs">
        Example only. Verdicts never derive from embeddings alone.
      </p>
    </div>
  </div>
);
