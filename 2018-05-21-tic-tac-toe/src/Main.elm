module Main exposing (main)

import Browser
import Dict exposing (Dict)
import Html exposing (..)
import Html.Attributes as Attr
import Html.Events exposing (onClick)


type Player
    = CirclePlayer
    | CrossPlayer


type Cell
    = EmptyCell
    | CircleCell
    | CrossCell


type alias Position =
    ( Int, Int )


type alias Board =
    Dict Position Cell


type alias Game =
    { activePlayer : Player
    , board : Board
    }


main =
    Browser.sandbox
        { init = init
        , update = update
        , view = view
        }


emptyBoard : Board
emptyBoard =
    let
        s =
            List.range 0 2
    in
    let
        assocs =
            List.concatMap (\y -> List.map (\x -> ( ( y, x ), EmptyCell )) s) s
    in
    Dict.fromList assocs


yourCell : Player -> Cell
yourCell player =
    case player of
        CirclePlayer ->
            CircleCell

        CrossPlayer ->
            CrossCell


opponent : Player -> Player
opponent player =
    case player of
        CirclePlayer ->
            CrossPlayer

        CrossPlayer ->
            CirclePlayer


put : Player -> Position -> Board -> ( Board, Player )
put player pos board =
    let
        next () =
            let
                cell =
                    yourCell player
            in
            let
                newBoard =
                    Dict.update pos (\_ -> Just cell) board
            in
            ( newBoard, opponent player )
    in
    let
        ignore () =
            ( board, player )
    in
    case Maybe.withDefault EmptyCell (Dict.get pos board) of
        EmptyCell ->
            next ()

        CircleCell ->
            ignore ()

        CrossCell ->
            ignore ()


type alias Model =
    { game : Game
    }


type Msg
    = Put Position


initGame : () -> Game
initGame () =
    { activePlayer = CirclePlayer
    , board = emptyBoard
    }


update : Msg -> ( Model, Cmd Msg ) -> ( Model, Cmd Msg )
update msg ( model, _ ) =
    case msg of
        Put pos ->
            let
                ( board, player ) =
                    put model.game.activePlayer pos model.game.board
            in
            ( { game = { board = board, activePlayer = player } }, Cmd.none )


cellButton : Cell -> Position -> Html Msg
cellButton cell pos =
    let
        b attrs children =
            button
                (Attr.type_ "button"
                    :: Attr.style "height" "80px"
                    :: Attr.style "width" "80px"
                    :: Attr.style "display" "flex"
                    :: Attr.style "justify-content" "center"
                    :: Attr.style "align-items" "center"
                    :: attrs
                )
                children
    in
    case cell of
        EmptyCell ->
            b [ onClick (Put pos) ] []

        CircleCell ->
            b [ Attr.disabled True ] [ text "○" ]

        CrossCell ->
            b [ Attr.disabled True ] [ text "×" ]


view : ( Model, Cmd Msg ) -> Html Msg
view ( model, _ ) =
    div []
        [ h1 [] [ text "Tic Tac Toe" ]
        , text "Hello, world!"
        , form
            [ Attr.style "width" "260px"
            , Attr.style "display" "flex"
            , Attr.style "flexFlow" "row wrap"
            ]
            (List.map
                (\( pos, cell ) -> cellButton cell pos)
                (Dict.toList model.game.board)
            )
        ]


init : ( Model, Cmd Msg )
init =
    ( { game = initGame () }
    , Cmd.none
    )
