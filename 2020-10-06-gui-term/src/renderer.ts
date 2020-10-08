// This file is required by the index.html file and will
// be executed in the renderer process for that window.
// No Node.js APIs are available in this process unless
// nodeIntegration is set to true in webPreferences.
// Use preload.js to selectively enable features
// needed in the renderer process.

import ReactDOM from "react-dom"
import { renderMain } from "./ui_main"

document.addEventListener("DOMContentLoaded", () => {
  const appContainerElement = document.getElementById("app-container") as HTMLDivElement

  ReactDOM.render(renderMain(), appContainerElement)
})
