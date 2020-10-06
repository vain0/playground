import React from "react"

interface JobState {
  jobId: string
  command: string
  output: string
}

interface Props {
  workDir: string
  jobs: JobState[]
  text: string
}

export const Main: React.FC<Props> = props => {
  const { workDir, jobs, text } = props

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
        value={text}
        onChange={() => {
          //
        }} />
    </main>
  )
}

export const renderMain = () => {
  // ダミーデータ
  const props: Props = {
    workDir: "/path/to/work-dir",
    jobs: [
      {
        jobId: "1",
        command: "echo 'Hello, world!'",
        output: "Hello, world!",
      },
      {
        jobId: "2",
        command: "pwd",
        output: "/path/to/work-dir",
      },
    ],
    text: "echo 'Bye!'",
  }

  return (<Main {...props} />)
}
