import { ipcRenderer } from "electron"
import React from "react"

interface JobState {
  jobId: string
  command: string
  output: string
}

type ExecuteFn = (cmdline: string) => void

const ExecuteContext = React.createContext(null as unknown as ExecuteFn)

export const Main: React.FC = () => {
  // 作業ディレクトリ
  const [workDir, setWorkDir] = React.useState("")
  React.useEffect(() => {
    console.log("[TRACE] get-work-dir BEGIN")
    ipcRenderer.invoke("gt-get-work-dir").then(workDir => {
      console.log("[TRACE] get-work-dir OK", workDir)
      setWorkDir(workDir)
    }).catch(err => console.error(err))
  }, [])

  const [jobs, setJobs] = React.useState<JobState[]>([
    {
      jobId: "-2",
      command: "echo 'Hello, world!'",
      output: "Hello, world!",
    },
    {
      jobId: "-1",
      command: "pwd",
      output: "/path/to/work-dir",
    },
  ])

  // コマンドライン
  const [cmdline, setCmdline] = React.useState("pwd")

  const trimmedCmdline = cmdline.trim()
  const execute = React.useMemo(() => async () => {
    if (trimmedCmdline === "") {
      console.log("[TRACE] can't execute empty string")
      return
    }

    console.log("[TRACE] execute", trimmedCmdline)
    const jobId = await ipcRenderer.invoke("gt-execute", trimmedCmdline)
    console.log("[TRACE]   jobId: ", jobId)

    setJobs([
      ...jobs,
      {
        jobId,
        command: trimmedCmdline,
        output: "",
      },
    ])
  }, [trimmedCmdline, jobs])

  return (
    <main id="app-main">
      <ul id="job-list">
        {jobs.map(({ jobId, command, output }) => (
          <li key={jobId}>
            <div>
              <code>$ {command}</code>
            </div>

            <pre>{output}</pre>
          </li>
        ))}
      </ul>

      <div>
        <code>$ {workDir}</code>
      </div>

      <textarea
        value={cmdline}
        onChange={ev => setCmdline(ev.target.value)}
        onKeyPress={ev => {
          const onlyCtrl = ev.ctrlKey && !ev.shiftKey && !ev.altKey && !ev.metaKey
          if (onlyCtrl && ev.key === "Enter") {
            execute()
          }
        }}
        rows={4} />
    </main>
  )
}

export const renderMain = () => {
  return (<Main />)
}
