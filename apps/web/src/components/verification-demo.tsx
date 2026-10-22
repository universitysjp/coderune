import { CheckCircle, Question, WarningCircle } from "@phosphor-icons/react";
import type { Icon } from "@phosphor-icons/react";
import { cn } from "cn";

type Verdict = "Verified" | "Contradicted" | "Unresolved";

const verdictStyles: Record<
  Verdict,
  { badge: string; icon: Icon; label: string }
> = {
  Contradicted: {
    badge: "bg-foreground text-background",
    icon: WarningCircle,
    label: "Conflicts with captured evidence",
  },
  Unresolved: {
    badge: "border border-border text-muted-foreground",
    icon: Question,
    label: "Not enough evidence in scope",
  },
  Verified: {
    badge: "bg-primary text-primary-foreground",
    icon: CheckCircle,
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
      <ul className="mt-3">
        {assumptions.map((assumption) => {
          const style = verdictStyles[assumption.verdict];
          const VerdictIcon = style.icon;

          return (
            <li
              className="border-border border-b py-3 first:pt-1 last:border-b-0 last:pb-0"
              key={assumption.claim}
            >
              <div className="flex items-start justify-between gap-4">
                <p className="text-sm leading-6">{assumption.claim}</p>
                <span
                  className={cn(
                    "inline-flex h-5 shrink-0 items-center gap-1 rounded-full px-2 text-[0.65rem] font-medium",
                    style.badge
                  )}
                  title={style.label}
                >
                  <VerdictIcon aria-hidden="true" weight="bold" />
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
