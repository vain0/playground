import { app, BrowserWindow, ipcMain } from "electron"
import * as path from "path"
import * as ChildProcess from "child_process"

// <[microsoft/node-pty\: Fork pseudoterminals in Node.JS](https://github.com/microsoft/node-pty)>
import * as pty from "node-pty"

type JobId = string

type Status =
  {
    kind: "RUNNING"
  } | {
    kind: "STOPPED"
    exitCode: number
  }

interface JobState {
  id: JobId
  cmdline: string
  proc: pty.IPty
  status: Status
}

let lastJobId = 0
let workDir = process.cwd()
const jobs: JobState[] = []

ipcMain.handle("gt-get-work-dir", () => {
  console.log("[TRACE] gt-get-work-dir")
  return workDir
})

ipcMain.handle("gt-execute", (_ev, cmdline) => {
  lastJobId++
  const jobId = lastJobId.toString()

  console.log("[TRACE] execute", jobId, cmdline)
  const [cmd, ...args] = cmdline.trim().split(" ")

  switch (cmd) {
    case "cd": {
      const [dir] = args

      console.log("[TRACE] cd", dir)
      workDir = dir
      return
    }
    case "pwd": {
      console.log("[TRACE] pwd", workDir)
      return
    }
    default:
      break
  }

  const proc = pty.spawn(cmd, args, {
    name: "xterm-color",
    cols: 80,
    rows: 30,
    cwd: workDir,
    env: process.env as Record<string, string>,
  })

  const job: JobState = {
    id: jobId,
    cmdline,
    proc,
    status: { kind: "RUNNING" },
  }
  jobs.push(job)

  proc.onData(data => {
    console.log("[TRACE] process data", jobId, data.length)
    ipcMain.emit("gt-data", jobId, data)
  })

  proc.onExit(({ exitCode, signal }) => {
    console.log("[TRACE] process exit", jobId, exitCode, signal)
    job.status = { kind: "STOPPED", exitCode }
    ipcMain.emit("gt-exit", jobId, exitCode, signal)
  })
})

ipcMain.handle("gt-kill", (_ev, jobId, signal) => {
  console.log("[TRACE] gt-kill", jobId, signal)

  const job = jobs.find(j => j.id === jobId)
  if (job == null || job.status.kind === "STOPPED") {
    console.log("[WARN]     job missing or stopped", job?.status?.kind)
    return null
  }

  job.proc.kill(signal)
  return true
})

ipcMain.handle("gt-write", (_ev, jobId, data) => {
  console.log("[TRACE] gt-write", jobId, data)

  const job = jobs.find(j => j.id === jobId)
  if (job == null || job.status.kind === "STOPPED") {
    console.log("[WARN]     job missing or stopped", job?.status)
    return null
  }

  job.proc.write(data)
  return true
})

function createWindow() {
  // Create the browser window.
  const mainWindow = new BrowserWindow({
    height: 600,
    webPreferences: {
      nodeIntegration: true,
      preload: path.join(__dirname, "preload.js"),
    },
    width: 800,
  })

  // and load the index.html of the app.
  mainWindow.loadFile(path.join(__dirname, "../index.html"))

  // Open the DevTools.
  // mainWindow.webContents.openDevTools()
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on("ready", () => {
  createWindow()

  app.on("activate", function () {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit()
  }
})

// In this file you can include the rest of your app"s specific main process
// code. You can also put them in separate files and require them here.
