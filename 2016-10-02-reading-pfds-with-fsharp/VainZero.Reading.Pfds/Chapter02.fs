﻿namespace VainZero.Reading.Pfds

module Chapter02 =
  // Ex2.1
  let rec suffix =
    function
    | x :: xs ->
      (x :: xs) :: suffix xs
    | [] ->
      [[]]

  (*
    時間計算量 O(n):
        長さ n のリスト xs について、time(suffix xs) = T(n) とおく。
        任意の xs: 'x list をとる。n = length xs とおく。
        xs = [] のとき、
            T(0) = time(suffix []) = 1
        xs = x :: xs' のとき、
            T(n) = time(suffix xs)
                = time((x :: xs') :: suffix xs')
                = time(suffix xs') + 2
                = T(n - 1) + 2
        従って、T(n) = 2n + 1 = O(n)
    空間計算量 O(n):
        長さ n のリスト xs について、space(suffix xs) = S(n) とおく。
        任意の xs: 'x list をとる。n = length xs とおく。
        xs = [] のとき
            S(0) = space([[]]) = 1
        xs = x :: xs' のとき、xs' は xs の部分リストなので空間 O(1) で表現できて、
            S(n) = space(suffix xs)
                = space(x :: xs') + space(suffix xs')
                = space(suffix xs') + O(1)
        従って、S(n) = O(n)
  *)

  type ISet<'e, 's when 's :> ISet<'e, 's>> =
    abstract member Empty: 's
    abstract member Insert: 'e -> 's
    abstract member Contains: 'e -> bool

  /// Unbalanced binary search tree representing set.
  type BinarySearchTree<'e when 'e: comparison> =
    | Empty
    | Node of BinarySearchTree<'e> * 'e * BinarySearchTree<'e>
  with
    member this.Insert(x) =
      match this with
      | Empty -> Node (Empty, x, Empty)
      | Node (left, y, right) ->
        if x < y then
          Node (left.Insert(x), y, right)
        elif x > y then
          Node (left, y, right.Insert(x))
        else
          this

    member this.Contains(x) =
      match this with
      | Empty -> false
      | Node (left, y, right) ->
        if x < y then
          left.Contains(x)
        elif x > y then
          right.Contains(x)
        else true

    interface ISet<'e, BinarySearchTree<'e>> with
      override this.Empty = Empty
      override this.Insert(x) = this.Insert(x)
      override this.Contains(x) = this.Contains(x)

  type EfficientBinarySearchTree<'e when 'e: comparison> =
    | Empty
    | Node of EfficientBinarySearchTree<'e> * 'e * EfficientBinarySearchTree<'e>
  with
    // Ex2.3
    // Ex2.4
    member this.Insert(x) =
      // Returns None if this contains x.
      let rec loop candidate this =
        match this with
        | Empty when candidate = Some x ->
          None
        | Empty ->
          Node (Empty, x, Empty) |> Some
        | Node (left, y, right) when x < y ->
          left
          |> loop candidate
          |> Option.map (fun left -> Node (left, y, right))
        | Node (left, y, right) ->
          right
          |> loop (Some y)
          |> Option.map (fun right -> Node (left, y, right))
      match this |> loop None with
      | Some result -> result
      | None -> this

    // Ex2.2
    member this.Contains(x) =
      let rec walk q this =
        match this with
        | Empty ->
          q = Some x
        | Node (left, y, _) when x < y ->
          left |> walk q
        | Node (_, y, right) ->
          right |> walk (Some y)
      in
        walk None this

    interface ISet<'e, EfficientBinarySearchTree<'e>> with
      override this.Empty = Empty
      override this.Insert(x) = this.Insert(x)
      override this.Contains(x) = this.Contains(x)

  // Ex2.5 (a)
  let rec complete d x =
    if d < 0 then
      invalidArg "d" "d must be nonnegative."
    elif d = 0 then
      Empty
    else
      let subtree = complete (d - 1) x
      Node (subtree, x, subtree)

  // Ex2.5 (b)
  let balanced n x =
    if n < 0 then
      invalidArg "n" "n must be nonnegative."
    elif n = 0 then
      Empty
    else
      let rec create n =
        if n = 0 then
          (Empty, Node (Empty, x, Empty))
        else
          let (s, t) = create ((n - 1) / 2)
          if (n - 1) % 2 = 0 then
            (Node (s, x, s), Node (s, x, t))
          else
            (Node (s, x, t), Node (t, x, t))
      in
        create n |> fst

  let rec count =
    function
    | Empty -> 0
    | Node (left, _, right) ->
      count left + count right + 1

  let rec isBalanced =
    function
    | Empty -> true
    | Node (left, _, right) ->
      (left |> isBalanced)
      && (right |> isBalanced)
      && abs (count left - count right) <= 1

  // Ex2.6
  module Exercise06 =
    type Map<'k, 'v, 'm> =
      {
        Empty           : 'm
        Insert          : 'k -> 'v -> 'm -> 'm
        TryFind         : 'k -> 'm -> option<'v>
      }

    type BinarySearchTree<'k, 'v when 'k: comparison> =
      | Empty
      | Node of BinarySearchTree<'k, 'v> * 'k * 'v * BinarySearchTree<'k, 'v>
    with
      member this.Insert(k, v) =
        match this with
        | Empty -> Node (Empty, k, v, Empty)
        | Node (left, k', v', right) ->
          if k < k' then
            Node (left.Insert(k, v), k', v', right)
          elif k > k' then
            Node (left, k', v', right.Insert(k, v))
          else
            this

      member this.TryFind(k) =
        match this with
        | Empty ->
          None
        | Node (left, k', v', right) ->
          if k < k' then
            left.TryFind(k)
          elif k > k' then
            right.TryFind(k)
          else
            Some v'

      static member AsMap: Map<'k, 'v, BinarySearchTree<'k, 'v>> =
        {
          Empty         = Empty
          Insert        = fun k v this -> this.Insert(k, v)
          TryFind       = fun k this -> this.TryFind(k)
        }
