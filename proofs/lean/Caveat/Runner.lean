import Caveat.Bridge
import Lean.Data.Json

open Lean

namespace Caveat.Runner
open Bridge

def schema : String := "caveat-guard-citation/0.2"

structure Request where
  id : String
  seed : Seed
  steps : List (List Action)

def exactFields (json : Json) (required optional : List String) : Except String Unit := do
  let fields ← json.getObj?
  for (key, _) in fields.toList do
    unless (required ++ optional).contains key do
      throw s!"unknown field: {key}"
  for key in required do
    unless fields.contains key do
      throw s!"missing field: {key}"

def arrayBetween (json : Json) (minimum maximum : Nat) (label : String) :
    Except String (List Json) := do
  let entries ← json.getArr?
  unless minimum ≤ entries.size && entries.size ≤ maximum do
    throw s!"{label} requires {minimum}..{maximum} entries"
  pure entries.toList

def parseName (json : Json) : Except String StateName := do
  match ← json.getStr? with
  | "a" => pure .a | "b" => pure .b | "g" => pure .g | "x" => pure .x | "y" => pure .y
  | name => throw s!"unknown state reference: {name}"

def parseNames (json : Json) (label : String) : Except String (List StateName) := do
  (← arrayBetween json 0 4 label).mapM parseName

def parseBodyBranch (json : Json) (label : String) : Except String (List StateName) := do
  match json with
  | .arr _ =>
      let entries ← arrayBetween json 0 4 s!"conditional {label}"
      entries.mapM fun entry => do
        match entry with
        | .str _ => parseName entry
        | _ => throw s!"conditional {label} requires state references"
  | _ => throw s!"conditional {label} requires an array"

/-- Only the exact object shape is admitted; branch entries are never recursive bodies. -/
def parseBody (json : Json) : Except String Body := do
  match json with
  | .arr _ => .sum <$> parseNames json "body"
  | .obj _ =>
      exactFields json ["condition", "then", "else"] []
      let conditionJson ← json.getObjVal? "condition"
      let condition ← match conditionJson with
        | .str _ => parseName conditionJson
        | _ => throw "conditional condition requires a state reference"
      let yes ← parseBodyBranch (← json.getObjVal? "then") "then"
      let no ← parseBodyBranch (← json.getObjVal? "else") "else"
      pure (.select condition yes no)
  | _ => throw "body requires an array or a conditional object"

def optional (json : Json) (key : String) : Option Json :=
  match json.getObjVal? key with
  | .error _ => none
  | .ok .null => none
  | .ok value => some value

def parseAction (json : Json) : Except String Action := do
  exactFields json ["target", "body", "guard", "citations"] []
  let target ← parseName (← json.getObjVal? "target")
  let body ← parseBody (← json.getObjVal? "body")
  let guard ← match optional json "guard" with
    | none => pure none
    | some value => some <$> parseName value
  let citations ← match optional json "citations" with
    | none => pure none
    | some value => some <$> parseNames value "citations"
  pure ⟨target, body, guard, citations⟩

def parseStep (json : Json) : Except String (List Action) := do
  exactFields json ["actions"] []
  (← arrayBetween (← json.getObjVal? "actions") 1 4 "actions").mapM parseAction

def parseSeed (json : Json) : Except String Seed := do
  exactFields json ["a", "b", "g"] []
  let a ← (← json.getObjVal? "a").getInt?
  let b ← (← json.getObjVal? "b").getInt?
  let g ← (← json.getObjVal? "g").getInt?
  unless a.natAbs ≤ 1000 && b.natAbs ≤ 1000 && (g = 0 || g = 1) do
    throw "seed requires a,b in [-1000,1000] and g in {0,1}"
  pure ⟨a, b, g⟩

def parseRequest (json : Json) : Except String Request := do
  exactFields json ["schema", "id", "seed", "steps"] []
  unless (← (← json.getObjVal? "schema").getStr?) == schema do
    throw "unsupported schema"
  let id ← (← json.getObjVal? "id").getStr?
  let idChars := id.toList
  unless 1 ≤ idChars.length && idChars.length ≤ 64 &&
      ('a' ≤ idChars.head! && idChars.head! ≤ 'z') &&
      idChars.all (fun c => ('a' ≤ c && c ≤ 'z') || ('0' ≤ c && c ≤ '9') || c == '-') do
    throw "id requires lowercase ASCII letter followed by up to 63 lowercase letters, digits or hyphens"
  let seed ← parseSeed (← json.getObjVal? "seed")
  let steps ← (← arrayBetween (← json.getObjVal? "steps") 1 8 "steps").mapM parseStep
  let actions := steps.flatten
  unless actions.length ≤ 8 do throw "at most eight total actions"
  let mut magnitude := max 1 (max seed.a.natAbs (max seed.b.natAbs seed.g.natAbs))
  for action in actions do
    magnitude := magnitude * max 1 action.body.width
    unless magnitude ≤ 1000000000 do throw "conservative intermediate numeric bound exceeded"
  pure ⟨id, seed, steps⟩

/-- Integers use canonical JSON integer tokens: fractional/exponent spellings and -0 are excluded. -/
def checkIntegerToken (token : String) : Except String Unit :=
  if token.isEmpty then pure ()
  else if token == "-0" then throw "signed zero is outside the comparison domain"
  else match token.toInt? with
    | some _ => pure ()
    | none => throw "non-integer numeric token is outside the comparison domain"

def checkNumericTokens (input : String) : Except String Unit := do
  let mut inString := false
  let mut escaped := false
  let mut token := ""
  for char in input.toList do
    if inString then
      if escaped then escaped := false
      else if char == '\\' then escaped := true
      else if char == '"' then inString := false
    else if char.isDigit || char == '-' || char == '+' || char == '.' ||
        char == 'e' || char == 'E' then
      if !token.isEmpty || char.isDigit || char == '-' then
        token := token.push char
    else
      checkIntegerToken token
      token := ""
      if char == '"' then inString := true
  checkIntegerToken token

/-- Called only after Json.parse checks syntax. Each object has its own decoded-key set. -/
def checkDuplicateFields (input : String) : Except String Unit := do
  let mut scopes : List (Option (List String)) := []
  let mut inString := false
  let mut escaped := false
  let mut token := ""
  let mut pendingString : Option String := none
  for char in input.toList do
    if inString then
      token := token.push char
      if escaped then escaped := false
      else if char == '\\' then escaped := true
      else if char == '"' then
        inString := false
        pendingString := some token
    else if char == '"' then
      inString := true
      token := "\""
      pendingString := none
    else if char == '{' then
      scopes := some [] :: scopes
      pendingString := none
    else if char == '[' then
      scopes := none :: scopes
      pendingString := none
    else if char == '}' || char == ']' then
      scopes := scopes.tail
      pendingString := none
    else if char == ':' then
      match scopes, pendingString with
      | some names :: outer, some rawKey =>
          let key ← (← Json.parse rawKey).getStr?
          if names.contains key then throw s!"duplicate field: {key}"
          scopes := some (key :: names) :: outer
          pendingString := none
      | _, _ => throw "invalid object key context"
    else if char != ' ' && char != '\n' && char != '\r' && char != '\t' then
      pendingString := none

/-- Read at most the byte limit plus one sentinel byte, including short stream reads. -/
def readRequest (stream : IO.FS.Stream) : IO (Except String String) := do
  let mut bytes := ByteArray.empty
  for _ in [:65537] do
    let chunk ← stream.read (65537 - bytes.size).toUSize
    bytes := bytes ++ chunk
    if bytes.size > 65536 then
      return .error "request exceeds 65536 UTF-8 bytes"
    if chunk.isEmpty then
      return match String.fromUTF8? bytes with
        | some input => .ok input
        | none => .error "request is not valid UTF-8"
  return .error "request exceeds 65536 UTF-8 bytes"

def provenanceJson (value : Provenance) : Json :=
  Json.mkObj [
    ("evidence", toJson value.evidence.eraseDups.mergeSort),
    ("caveats", toJson value.caveats.eraseDups.mergeSort)]

def trackedJson (value : Tracked) : Json :=
  Json.mkObj [
    ("value", toJson value.value),
    ("lineage", provenanceJson value.lineage),
    ("grounds", provenanceJson value.grounds)]

def statesJson (state : State) : Json :=
  Json.mkObj (stateNames.map (fun name => (name.text, trackedJson (state name))))

def outcomeJson (outcome : Outcome Session) : Json :=
  match outcome with
  | .accepted _ => Json.mkObj [("outcome", toJson ("accepted" : String))]
  | .rejected reason =>
      let result : Outcome Session := .rejected reason
      Json.mkObj [("outcome", toJson ("rejected" : String)),
        ("origin", toJson result.origin), ("code", toJson result.code)]
  | .fatal reason =>
      let result : Outcome Session := .fatal reason
      Json.mkObj [("outcome", toJson ("fatal" : String)), ("code", toJson result.code)]

def frameJson (outcome : Json) (session : Session) : Json :=
  Json.mkObj [
    ("outcome", outcome),
    ("states", statesJson session.values),
    ("observations", toJson session.observations),
    ("effects", toJson (session.revealEffects.map (fun name =>
      Json.mkObj [("kind", toJson ("reveal" : String)), ("evidence", toJson name)]))),
    ("decision_journal", Json.arr #[])]

def execute (request : Request) : Except String Json := do
  let seed := seedSession request.seed
  let mut state := seed
  let mut frames := [
    frameJson (Json.mkObj [("outcome", toJson ("initial" : String))]) initialSession,
    frameJson (outcomeJson (.accepted seed)) seed]
  for actions in request.steps do
    let (outcome, next) := runSessionStep state actions
    match next with
    | none => throw "unexpected fatal result in the admitted comparison fragment"
    | some after =>
        state := after
        frames := frames ++ [frameJson (outcomeJson outcome) after]
  pure (Json.mkObj [("schema", toJson schema), ("id", toJson request.id), ("frames", toJson frames)])

def process (input : String) : Except String Json := do
  unless input.utf8ByteSize ≤ 65536 do throw "request exceeds 65536 UTF-8 bytes"
  checkNumericTokens input
  let json ← Json.parse input
  checkDuplicateFields input
  let request ← parseRequest json
  execute request

end Caveat.Runner

def main : IO UInt32 := do
  let input ← Caveat.Runner.readRequest (← IO.getStdin)
  match input.bind Caveat.Runner.process with
  | .ok output =>
      (← IO.getStdout).putStrLn output.compress
      pure 0
  | .error error =>
      (← IO.getStderr).putStrLn s!"caveat_compare: {error}"
      pure 1
