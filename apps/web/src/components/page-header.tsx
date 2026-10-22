import type { ReactNode } from "react";

export const PageHeader = ({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children?: ReactNode;
}) => (
  <header className="mb-12">
    <h1 className="max-w-[42rem] text-[length:var(--font-size-title)] leading-[1.2] font-semibold tracking-[-0.025em] text-balance">
      {title}
    </h1>
    <p className="text-muted-foreground mt-[18px] max-w-[650px] text-[length:var(--font-size-heading)] leading-[1.55]">
      {description}
    </p>
    {children ? <div className="mt-6">{children}</div> : null}
  </header>
);
