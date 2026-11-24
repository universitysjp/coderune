import type { ReactNode } from "react";

import { SectionHeading } from "@/components/section-heading";

export const ContentSection = ({
  id,
  title,
  children,
}: {
  id: string;
  title: string;
  children: ReactNode;
}) => (
  <section aria-labelledby={`${id}-title`} className="content-section" id={id}>
    <SectionHeading headingId={`${id}-title`} title={title} />
    <div className="section-body">{children}</div>
  </section>
);
