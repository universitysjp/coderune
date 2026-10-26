import { cn } from "cn";

export const SiteLogo = ({ className }: { className?: string }) => (
  <span aria-hidden="true" className={cn("site-logo", className)}>
    ₪
  </span>
);
