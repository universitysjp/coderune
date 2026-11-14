import { createRootRoute, Link, Outlet } from "@tanstack/react-router";

import "@/ui.css";

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
  <section className="flex flex-col items-start py-8">
    <p className="type-body-sm text-text-2 data">404</p>
    <h1 className="page-title mt-4">
      This page is outside the captured scope.
    </h1>
    <p className="text-muted-foreground mt-3 max-w-lg">
      The route does not exist. Return to the overview to continue reading about
      Coderune.
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
