import { ipcRenderer } from "electron"
import React from "react"
import { JobStatus, Exited, ExitedOk, ExitedErr } from "./shared/job_status"

type JobId = string

interface JobState {
  jobId: JobId
  cmdline: string
  status: JobStatus
  output: string
}

export const Main: React.FC = () => {
  // 作業ディレクトリ
  const [workDir, setWorkDir] = React.useState("...")

  // 起動時に作業ディレクトリを受け取る。
  React.useEffect(() => {
    console.log("[TRACE] get-work-dir BEGIN")
    ipcRenderer.invoke("gt-get-work-dir").then(workDir => {
      console.log("[TRACE] get-work-dir OK", workDir)
      setWorkDir(workDir)
    }).catch(err => console.error(err))
  }, [])

  // ジョブリスト

  const [jobs, setJobs] = React.useState<JobState[]>([
    // ダミーデータ
    {
      jobId: "-2",
      cmdline: "echo 'Hello, world!'",
      output: "Hello, world!",
      status: ExitedOk,
    },
    {
      jobId: "-1",
      cmdline: "pwd",
      output: "/path/to/work-dir",
      status: ExitedErr,
    },
  ])

  // ジョブの出力を追記する。
  React.useEffect(() => {
    ipcRenderer.on("gt-job-created", (_ev, jobId: JobId, cmdline: string, status) => {
      console.log("[TRACE] gt-job-created", jobId)

      setJobs(jobs => [
        ...jobs,
        {
          jobId,
          cmdline,
          output: "",
          status,
        },
      ])
    })

    ipcRenderer.on("gt-job-data", (_ev, jobId: JobId, data: string) => {
      console.log("[TRACE] gt-job-data", jobId, data.length)

      setJobs(jobs => jobs.map(job => {
        if (job.jobId !== jobId) {
          return job
        }

        const output = job.output !== "" ? job.output + "\n" + data : data
        return { ...job, output }
      }))
    })

    ipcRenderer.on("gt-job-exit", (_ev, jobId: JobId, exitCode: number, signal: unknown) => {
      console.log("[TRACE] gt-job-exit", jobId, exitCode, signal)

      setJobs(jobs => jobs.map(job => {
        if (job.jobId !== jobId) {
          return job
        }

        return { ...job, status: Exited(exitCode) }
      }))
    })
  }, [])

  // コマンドライン
  const [cmdline, setCmdline] = React.useState("pwd")

  const trimmedCmdline = cmdline.trim()
  const execute = React.useMemo(() => async () => {
    if (trimmedCmdline === "") {
      console.log("[TRACE] can't execute empty string")
      return
    }

    // const channel = new MessageChannel()

    console.log("[TRACE] execute", trimmedCmdline)
    ipcRenderer.send("gt-execute", trimmedCmdline)
  }, [trimmedCmdline, jobs])

  return (
    <main id="app-main">
      <ul id="job-list">
        {jobs.map(({ jobId, cmdline: command, output, status }) => (
          <li key={jobId} data-status={status.kind === "EXITED" ? (status.exitCode === 0 ? "ok" : "error") : "running"}>
            <details>
              <summary>
                <code>$ {command}</code>
                <code color="#666">#{jobId}</code>
              </summary>

              <pre style={{ background: "#eee" }}>{output}</pre>
            </details>
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
