import React from "react";
import { createRoot } from "react-dom/client";
import { Workbench } from "./Workbench";
import { applicationHost } from "./hosts/application-host";
import "./base.css";

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Workbench host={applicationHost} />
  </React.StrictMode>,
);
