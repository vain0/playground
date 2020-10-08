export type JobStatus =
  {
    kind: "RUNNING"
  } | {
    kind: "EXITED"
    exitCode: number
  }

export const Running: JobStatus = { kind: "RUNNING" }

export const Exited = (exitCode: number): JobStatus =>
  ({ kind: "EXITED", exitCode })

export const ExitedOk = Exited(0)

export const ExitedErr = Exited(1)
