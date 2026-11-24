import type { AnchorHTMLAttributes, ReactNode } from "react";

import { isExternal, siteHref } from "@/lib/paths";

type Props = AnchorHTMLAttributes<HTMLAnchorElement> & {
  children: ReactNode;
  to: string;
};

export const Link = ({ to, children, ...rest }: Props) => (
  <a href={isExternal(to) ? to : siteHref(to)} {...rest}>
    {children}
  </a>
);
