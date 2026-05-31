import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import CommandBar from "./CommandBar";
import "./App.css";

// Both windows load the same bundle; we render a different root per window.
// `main` = the full app; `palette` = the floating command bar (M3).
const isPalette = getCurrentWindow().label === "palette";
if (isPalette) document.body.classList.add("palette-window");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>{isPalette ? <CommandBar /> : <App />}</React.StrictMode>,
);
