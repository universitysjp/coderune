import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import "./index.css";
import { QuartzPortalProvider } from "./lib/quartz/portal";
import { router } from "./router";

const root = document.querySelector("#root");

if (!root) {
  throw new Error("Root element not found");
}

createRoot(root).render(
  <StrictMode>
    <QuartzPortalProvider>
      <RouterProvider router={router} />
    </QuartzPortalProvider>
  </StrictMode>
);
