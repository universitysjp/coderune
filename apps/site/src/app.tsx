import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import { DesignPage } from "@/routes/design";
import { EvaluationPage } from "@/routes/evaluation";
import { HomePage } from "@/routes/index";
import { NotFoundPage } from "@/routes/not-found";
import { ResearchPage } from "@/routes/research";
import { TeamPage } from "@/routes/team";

const base = import.meta.env.BASE_URL.replace(/\/$/u, "");

const currentPath = (pathname: string): string => {
  let path = pathname;

  if (base && path.startsWith(base)) {
    path = path.slice(base.length);
  }

  return path.replace(/\/+$/u, "") || "/";
};

const page = (path: string) => {
  switch (path) {
    case "/": {
      return <HomePage />;
    }

    case "/research": {
      return <ResearchPage />;
    }

    case "/design": {
      return <DesignPage />;
    }

    case "/evaluation": {
      return <EvaluationPage />;
    }

    case "/team": {
      return <TeamPage />;
    }

    default: {
      return <NotFoundPage />;
    }
  }
};

export const App = ({ pathname }: { pathname: string }) => {
  const path = currentPath(pathname);

  return (
    <div className="site-page min-h-svh">
      <SiteHeader pathname={path} />
      <main className="site-layout" id="top">
        {page(path)}
        <SiteFooter />
      </main>
    </div>
  );
};
