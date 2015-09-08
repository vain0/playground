module LispVal where

data LispVal
	= Atom String
	| List [LispVal]
	| DottedList [LispVal] LispVal
	| Number Integer
	| String String
	| Char Char
	| Bool Bool
	deriving (Show)
