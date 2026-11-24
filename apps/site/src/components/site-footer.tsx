import { Separator } from "@ui-sdk/react/separator";

import { Link } from "@/components/link";
import { navItems, site } from "@/lib/site";

export const SiteFooter = () => (
  <footer className="site-footer">
    <p className="max-w-[38rem]">
      {site.name} is a research prototype at {site.university}. Local-first by
      design; no participant data leaves the device.
    </p>
    <Separator className="footer-rule" />
    <nav aria-label="Footer navigation" className="footer-nav">
      {navItems.map((item) => (
        <Link key={item.to} to={item.to}>
          {item.label}
        </Link>
      ))}
      <a href={site.repositoryUrl} rel="noreferrer" target="_blank">
        GitHub
      </a>
      <a href={site.proposalUrl} rel="noreferrer" target="_blank">
        Proposal
      </a>
    </nav>
  </footer>
);
