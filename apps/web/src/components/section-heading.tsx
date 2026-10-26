import type { ReactNode } from "react";

export const SectionHeading = ({
  title,
  headingId,
  description,
  action,
}: {
  title: string;
  headingId?: string;
  description?: string;
  action?: ReactNode;
}) => (
  <div className="section-heading flex flex-col gap-3 sm:flex-row sm:items-baseline sm:justify-between">
    <div>
      <h2 className="section-title" id={headingId}>
        {title}
      </h2>
      {description ? (
        <p className="text-muted-foreground mt-3 max-w-[38rem]">
          {description}
        </p>
      ) : null}
    </div>
    {action ? <div className="shrink-0">{action}</div> : null}
  </div>
);
