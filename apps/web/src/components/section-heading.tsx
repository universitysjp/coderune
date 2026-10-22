import type { ReactNode } from "react";

export const SectionHeading = ({
  title,
  description,
  action,
}: {
  title: string;
  description?: string;
  action?: ReactNode;
}) => (
  <div className="mt-14 flex flex-col gap-4 sm:flex-row sm:items-baseline sm:justify-between">
    <div>
      <h2 className="text-[length:var(--font-size-heading)] leading-[1.4] font-medium">
        {title}
      </h2>
      {description ? (
        <p className="text-muted-foreground mt-4 max-w-[38rem] leading-[1.55]">
          {description}
        </p>
      ) : null}
    </div>
    {action ? <div className="shrink-0">{action}</div> : null}
  </div>
);
