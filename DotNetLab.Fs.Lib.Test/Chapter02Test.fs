﻿namespace DotNetLab.Fs.Lib.PFDS

open Persimmon
open Persimmon.Syntax.UseTestNameByReflection
open DotNetLab.Fs.Lib.PFDS.Chapter02

module Chapter02Test =
  let suffixTest =
    test {
      let actual = [1; 2; 3; 4] |> suffix
      let expected =
        [
          [1; 2; 3; 4]
          [2; 3; 4]
          [3; 4]
          [4]
          []
        ]
      do! actual |> assertEquals expected
    }

  let testISet (s: ISet<int, 's>) =
    let empty = s.Empty :> ISet<int, 's>
    [
      yield
        test {
          do! empty.Contains(0) |> assertEquals false
        }
      yield
        test {
          let set = empty.Insert(0).Insert(1)
          do! set.Contains(0) |> assertEquals true
          do! set.Contains(1) |> assertEquals true
          do! set.Contains(2) |> assertEquals false
        }
      yield
        test {
          let set = empty.Insert(0).Insert(0)
          do! set.Contains(0) |> assertEquals true
        }
      yield!
        seq {
          let xs = [4; 6; 5; 7; 2]
          let set =
             xs |> List.fold (fun (set: 's) x -> set.Insert(x)) s.Empty
          let cases =
            seq {
              let ys = [0; 1; 3; 8]
              yield! xs |> Seq.map (fun x -> (x, true))
              yield! ys |> Seq.map (fun y -> (y, false))
            }
          for (x, expected) in cases do
            yield
              test {
                do! set.Contains(x) |> assertEquals expected
              }
        }
    ]

  let testBinarySearchTree =
    testISet BinarySearchTree<int>.Empty

  let testEfficientBinarySearchTree =
    let empty = EfficientBinarySearchTree<int>.Empty
    let insert x (s: ISet<_, _>) = s.Insert(x)
    [
      yield!
        testISet empty
      yield
        test {
          let xs = [4; 6; 5; 7; 2]
          let set = xs |> List.fold (fun set x -> set |> insert x) empty
          let set' = set.Insert(2)
          do! obj.ReferenceEquals(set, set') |> assertEquals true
        }
    ]

  let testComplete =
    [
      test {
        let x = 0
        do! complete 0 x |> assertEquals Empty
      }
      test {
        let x = 0
        do! complete 1 x |> assertEquals (Node (Empty, x, Empty))
      }
      test {
        let x = 0
        do!
          complete 2 x
          |> assertEquals
            (Node (Node (Empty, x, Empty), x, Node (Empty, x, Empty)))
      }
    ]
