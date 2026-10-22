export const SiteLogo = ({ className }: { className?: string }) => (
  <svg
    aria-hidden="true"
    className={className ?? "size-7 shrink-0"}
    fill="none"
    viewBox="0 0 64 64"
  >
    <path
      d="M25 18H14v28h11M39 18h11v28H39"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="4"
    />
    <path
      d="M19 32h26"
      className="stroke-primary"
      strokeLinecap="round"
      strokeWidth="4"
    />
    <circle className="fill-primary" cx="32" cy="32" r="4" />
  </svg>
);
