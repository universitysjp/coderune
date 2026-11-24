import { StrictMode } from "react";

import "./index.css";

import { createRoot } from "react-dom/client";

import { App } from "./app";

const root = document.querySelector("#root");

if (!root) {
  throw new Error("Root element not found");
}

createRoot(root).render(
  <StrictMode>
    <App pathname={window.location.pathname} />
  </StrictMode>
);
