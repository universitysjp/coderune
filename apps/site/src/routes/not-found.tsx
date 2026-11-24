import { Link } from "@/components/link";

export const NotFoundPage = () => (
  <section className="flex flex-col items-start py-8">
    <p className="text-muted-foreground font-mono text-xs">404</p>
    <h1 className="page-title mt-4">
      This page is outside the captured scope.
    </h1>
    <p className="text-muted-foreground mt-3 max-w-lg">
      The route does not exist. Return to the overview to continue reading about
      Coderune.
    </p>
    <Link className="text-link mt-7" to="/">
      Back to overview
    </Link>
  </section>
);
