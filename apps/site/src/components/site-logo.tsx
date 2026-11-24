export const SiteLogo = ({ className }: { className?: string }) => (
  <span aria-hidden="true" className={`site-logo ${className ?? ""}`.trim()}>
    ₪
  </span>
);
