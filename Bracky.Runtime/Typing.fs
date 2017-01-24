﻿namespace Bracky.Runtime.Typing

open System
open Bracky.Runtime.Parsing

// 参考
// https://sites.google.com/site/scalajp/home/documentation/scala-by-example/chapter16
// http://uehaj.hatenablog.com/entry/2014/02/01/183039
// http://www.fos.kuis.kyoto-u.ac.jp/~igarashi/class/isle4-06w/text/miniml011.html

[<RequireQualifiedAccess>]
type Kind =
  | Unit
  | Int
  | Bool

type TypeVariable =
  | TypeVariable of int64
with
  override this.ToString() =
    let (TypeVariable identifier) = this
    sprintf "'%d" identifier

[<CompilationRepresentation(CompilationRepresentationFlags.ModuleSuffix)>]
module private TypeVariable =
  let count = ref 0L
  let fresh () =
    let value = !count
    count := value + 1L
    value |> TypeVariable

type TypeExpression =
  | RefTypeExpression
    of TypeVariable
  | FunTypeExpression
    of TypeExpression * TypeExpression
  | AppTypeExpression
    of Kind * array<TypeExpression>
with
  override this.ToString() =
    match this with
    | RefTypeExpression tv ->
      string tv
    | FunTypeExpression (sourceType, targetType) ->
      sprintf "(%s -> %s)" (string sourceType) (string targetType)
    | AppTypeExpression (kind, arguments) ->
      if arguments |> Array.isEmpty then
        string kind
      else
        let argumentList = arguments |> Array.map string |> String.concat " "
        sprintf "(%s %s)" (string kind) argumentList

  member this.TypeVariableSet =
    match this with
    | RefTypeExpression tv ->
      Set.singleton tv
    | FunTypeExpression (s, t) ->
      Set.union s.TypeVariableSet t.TypeVariableSet
    | AppTypeExpression (_, arguments) ->
      arguments |> Seq.map (fun t -> t.TypeVariableSet) |> Set.unionMany

[<CompilationRepresentation(CompilationRepresentationFlags.ModuleSuffix)>]
module TypeExpression =
  let unit =
    AppTypeExpression (Kind.Unit, [||])

  let int =
    AppTypeExpression (Kind.Int, [||])

  let bool =
    AppTypeExpression (Kind.Bool, [||])

[<AbstractClass>]
type Substitution() =
  abstract Item: TypeVariable -> TypeExpression with get

  member this.Apply(t: TypeExpression) =
    match t with
    | RefTypeExpression tv ->
      let t' = this.[tv]
      if t = t' then t else this.Apply(t')
    | FunTypeExpression (s, t) ->
      FunTypeExpression (this.Apply(s), this.Apply(t))
    | AppTypeExpression (kind, arguments) ->
      AppTypeExpression (kind, arguments |> Array.map this.Apply)

  member this.Extend(tu: TypeVariable, t: TypeExpression) =
    { new Substitution() with
        override this.Item
          with get tv =
            if tv = tu then t else this.[tv]
    }

  member this.ExtendMany(bindings: Map<TypeVariable, TypeExpression>) =
    { new Substitution() with
        override this.Item
          with get tv =
            match bindings |> Map.tryFind tv with
            | Some t -> t
            | None -> this.[tv]
    }

  static member val Empty =
    { new Substitution() with
        override this.Item
          with get tv = RefTypeExpression tv
    }

type ForallTypeScheme =
  | ForallTypeScheme
    of array<TypeVariable> * TypeExpression
with
  member this.Instantiate() =
    let (ForallTypeScheme (tvs, t)) = this
    let bindings =
      tvs
      |> Array.map (fun tv -> (tv, TypeVariable.fresh () |> RefTypeExpression))
      |> Map.ofArray
    let substitution =
      Substitution.Empty.ExtendMany(bindings)
    substitution.Apply(t)

  member this.FreeTypeVariableSet =
    let (ForallTypeScheme (tvs, t)) = this
    Set.difference t.TypeVariableSet (tvs |> Set.ofArray)

/// From variables to type schemas.
type Environment =
  Map<string, ForallTypeScheme>

[<CompilationRepresentation(CompilationRepresentationFlags.ModuleSuffix)>]
module Environment =
  let freeTypeVariableSet (this: Environment) =
    this
    |> Seq.map (fun (KeyValue (_, ts)) -> ts.FreeTypeVariableSet)
    |> Set.unionMany

  let variableSet (this: Environment) =
    this |> Seq.map (fun (KeyValue (k, v)) -> k) |> Set.ofSeq

  /// Converts a type expression to a type scheme by binding all free variables with ∀.
  let generalize t (this: Environment) =
    let tvs = (t: TypeExpression).TypeVariableSet |> Set.toArray
    ForallTypeScheme (tvs, t)

[<CompilationRepresentation(CompilationRepresentationFlags.ModuleSuffix)>]
module TypeInference =
  open Bracky.Runtime.Parsing

  let rec unify t t' (substitution: Substitution) =
    match (substitution.Apply(t), substitution.Apply(t')) with
    | (RefTypeExpression tv, RefTypeExpression tv') when tv = tv' ->
      substitution.Extend(tv, t')
    | (RefTypeExpression tv, _)
      when (t': TypeExpression).TypeVariableSet |> Set.contains tv |> not ->
      substitution.Extend(tv, t')
    | (_, RefTypeExpression tv) ->
      unify t' t substitution
    | (FunTypeExpression (s, u), FunTypeExpression (s', u')) ->
      let substitution =
        unify u u' substitution
      unify s s' substitution
    | (AppTypeExpression (kind, arguments), AppTypeExpression (kind', arguments'))
      when kind = kind' ->
      Array.zip arguments arguments' |> Array.fold
        (fun substitution (a, a') ->
          unify a a' substitution
        ) substitution
    | (_, _) ->
      failwith "TODO: error handling"

  let infer =
    let rec loop previous expression t (substitution, environment) =
      let loop = loop (Some expression)
      match expression with
      | IntExpression _
      | BoolExpression _
        -> (substitution, environment)
      | RefExpression (_, identifier) ->
        match environment |> Map.tryFind identifier with
        | Some (typeScheme: ForallTypeScheme) ->
          let t' = typeScheme.Instantiate()
          let substitution = unify t t' substitution
          (substitution, environment)
        | None ->
          failwith "TODO: variable not defined"
      | FunExpression (_, pattern, expresssion) ->
        let (IdentifierPattern (_, identifier)) = pattern
        let tv = TypeVariable.fresh () |> RefTypeExpression
        let tu = TypeVariable.fresh () |> RefTypeExpression
        let t' = FunTypeExpression (tv, tu)
        let substitution = substitution |> unify t t'
        let environment = environment |> Map.add identifier (ForallTypeScheme ([||], tv))
        loop expression tu (substitution, environment)
      | IfExpression _ ->
        NotImplementedException() |> raise
      | BinaryOperationExpression (operator, left, right) ->
        match operator with
        | ThenOperator ->
          (substitution, environment)
          |> loop left TypeExpression.unit
          |> loop right t
        | _ ->
          NotImplementedException() |> raise
      | ValExpression ((IdentifierPattern (_, identifier)), expression) ->
        let tv = TypeVariable.fresh () |> RefTypeExpression
        let (substitution, environment) =
          (substitution, environment) |> loop expression tv
        let variableType = environment |> Environment.generalize (substitution.Apply(tv))
        let environment = environment |> Map.add identifier variableType
        (substitution, environment)
    fun expression t environment substitution ->
      loop None expression t (substitution, environment)
