import { createRootRoute, Link, Outlet } from "@tanstack/react-router";

import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { buttonVariants } from "@/components/ui/button-variants";

const RootLayout = () => (
  <div className="site-page min-h-svh">
    <SiteHeader />
    <main className="site-layout" id="top">
      <Outlet />
      <SiteFooter />
    </main>
  </div>
);

const NotFoundPage = () => (
  <section className="flex flex-col items-start py-16">
    <p className="text-muted-foreground font-mono text-sm">404</p>
    <h1 className="mt-5 text-[clamp(2rem,4vw,2.5rem)] leading-[1.15] font-semibold tracking-[-0.025em]">
      This page is outside the captured scope.
    </h1>
    <p className="text-muted-foreground mt-4 max-w-lg leading-7">
      The route does not exist. Return to the overview to continue reading about
      Context Bridge.
    </p>
    <Link className={buttonVariants({ className: "mt-7" })} to="/">
      Back to overview
    </Link>
  </section>
);

export const Route = createRootRoute({
  component: RootLayout,
  notFoundComponent: NotFoundPage,
});
