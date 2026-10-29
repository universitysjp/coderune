import { QuartzPortalProvider } from "@coderune/quartz/web";
import { RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";

import "./index.css";
import { createRoot } from "react-dom/client";

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
