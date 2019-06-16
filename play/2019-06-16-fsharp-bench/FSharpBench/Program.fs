﻿module Program

open System.Text
open BenchmarkDotNet
open BenchmarkDotNet.Attributes
open BenchmarkDotNet.Configs
open BenchmarkDotNet.Jobs

let inline cons head tail = head :: tail

[<Struct>]
type Token =
  | Int
    of intValue:int
  | Str
    of strValue:string

type Benchmarks() =
  [<Benchmark>]
  member __.StringBuilder() =
    let out = StringBuilder()
    let rec go (out: StringBuilder) i =
      if i <= 10_000 then
        out
          .Append(i).Append(",")
          .Append(i * i).Append("\n")
          |> ignore
        go out (i + 1)
    go out 1
    out.ToString()

  [<Benchmark>]
  member __.StringListConcat() =
    let rec go i acc =
      if i > 10_000 then
        acc
      else
        acc
        |> cons (string i) |> cons ","
        |> cons (string (i * i)) |> cons "\n"
        |> go (i + 1)
    [] |> go 1 |> List.rev |> String.concat ""

  [<Benchmark>]
  member __.TokenListRender() =
    let render tokens =
      let tokens = tokens |> List.toArray
      let out = StringBuilder()

      for i in tokens.Length - 1..-1..0 do
        match tokens.[i] with
        | Token.Int value ->
          out.Append(value) |> ignore

        | Token.Str value ->
          out.Append(value) |> ignore

      out.ToString()

    let rec go i acc =
      if i > 10_000 then
        acc
      else
        acc
        |> cons (Token.Int i) |> cons (Token.Str ",")
        |> cons (Token.Int (i * i)) |> cons (Token.Str "\n")
        |> go (i + 1)

    [] |> go 1 |> render

[<EntryPoint>]
let main _ =
  // 結果が正しいことを確認する。
  let expected = Benchmarks().StringBuilder()
  assert (
    Benchmarks().StringListConcat() = expected
    && Benchmarks().TokenListRender() = expected
  )

#if !DEBUG
  // ベンチマークをとる。
  let config () =
    let rough = AccuracyMode(MaxRelativeError = 0.1)
    let quickRoughJob = Job("QuickRough", rough, RunMode.Short)
    let mutable config = ManualConfig()
    config.Add(quickRoughJob)
    config <- ManualConfig.Union(DefaultConfig.Instance, config)
    config

  let _summary = Running.BenchmarkRunner.Run<Benchmarks>(config ())
#endif

  0
