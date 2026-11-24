import { Link } from "@/components/link";
import { SiteLogo } from "@/components/site-logo";
import { navItems, pageSections, site } from "@/lib/site";

const isActive = (pathname: string, target: string) => {
  if (target === "/") {
    return pathname === target;
  }

  return pathname === target || pathname.startsWith(`${target}/`);
};

export const SiteHeader = ({ pathname }: { pathname: string }) => {
  const activeItem = navItems.find(({ to }) => isActive(pathname, to));
  const sections = activeItem ? pageSections[activeItem.to] : [];

  return (
    <header className="site-sidebar">
      <div className="flex items-center justify-between gap-4">
        <Link
          aria-label={`${site.name} home`}
          className="flex items-center gap-2 text-base leading-none font-semibold tracking-[0.01em] whitespace-nowrap uppercase"
          to="/"
        >
          <SiteLogo className="size-6" />
          <span>{site.name}</span>
        </Link>
        <a
          aria-label="GitHub repository"
          className="text-muted-foreground hover:text-foreground font-mono text-sm tracking-[0.04em] uppercase transition-colors min-[60rem]:hidden"
          href={site.repositoryUrl}
          rel="noreferrer"
          target="_blank"
        >
          GitHub
        </a>
      </div>

      <nav
        aria-label="Main navigation"
        className="sidebar-nav hidden min-[60rem]:grid"
      >
        {navItems.map((item) => {
          const active = activeItem?.to === item.to;

          return (
            <div key={item.to}>
              <Link aria-current={active ? "page" : undefined} to={item.to}>
                {item.label}
              </Link>
              {active && sections.length > 0 ? (
                <div className="sidebar-outline">
                  {sections.map((section) => (
                    <a href={`#${section.id}`} key={section.id}>
                      {section.label}
                    </a>
                  ))}
                </div>
              ) : null}
            </div>
          );
        })}
        <a
          className="sidebar-meta"
          href={site.repositoryUrl}
          rel="noreferrer"
          target="_blank"
        >
          GitHub
        </a>
      </nav>

      <nav
        aria-label="Main navigation"
        className="mt-4 flex flex-wrap gap-x-4 gap-y-2 font-mono text-sm tracking-[0.04em] uppercase min-[60rem]:hidden"
      >
        {navItems.map((item) => (
          <Link
            aria-current={activeItem?.to === item.to ? "page" : undefined}
            className={
              activeItem?.to === item.to
                ? "text-foreground shrink-0 transition-colors"
                : "text-muted-foreground hover:text-foreground shrink-0 transition-colors"
            }
            key={item.to}
            to={item.to}
          >
            {item.label}
          </Link>
        ))}
      </nav>
    </header>
  );
};
