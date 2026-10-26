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
  <header className="page-header">
    <h1 className="page-title">{title}</h1>
    <p className="page-description text-muted-foreground">{description}</p>
    {children ? <div className="mt-4">{children}</div> : null}
  </header>
);
