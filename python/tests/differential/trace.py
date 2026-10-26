"""Replays scenario files through the Python package's native runtime.

Prints the same JSON document trace.mjs prints from the npm kit's WebAssembly
runtime; test_differential.py requires the two to be identical.

    python trace.py FILE.scenarios.json...
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Callable, Dict, List

from caveat_lang import payload_text
from caveat_lang._runtime import NativeSession, RuntimeFailure, Trap

Result = Dict[str, Any]


def attempt(read: Callable[[], Any]) -> Result:
    """The runtime's text, or what it raised."""
    try:
        return {"text": read()}
    except (RuntimeFailure, Trap) as error:
        return {"error": str(error)}


def program(source: str) -> Result:
    return {
        "check": attempt(lambda: NativeSession.check(source)),
        "check_unused": attempt(lambda: NativeSession.check_with(source, True)),
        "check_records": attempt(
            lambda: NativeSession.check_options(source, json.dumps({"unused": False, "records": True}))),
        "interface": attempt(lambda: NativeSession.interface(source)),
    }


def ending(session: NativeSession, source: str) -> Result:
    end = {
        "snapshot": attempt(session.snapshot),
        "view": attempt(session.view),
        "view_v2": attempt(session.view_v2),
        "save": attempt(session.save),
        "save_compact": attempt(session.save_compact),
        "save_positional": attempt(session.save_compact_positional),
        "undrained": attempt(lambda: str(session.undrained())),
        "archive": attempt(session.drain_archive),
    }
    for kind in ("save", "save_compact", "save_positional"):
        if "text" in end[kind]:
            saved = end[kind]["text"]
            end[f"{kind}_restored"] = attempt(lambda: NativeSession.restore(source, saved).snapshot())
    return end


def scenario(item: Result, source: str) -> Result:
    steps: List[Result] = []
    session = NativeSession(source)
    checkpoint = None
    variant = 0
    for step in item["steps"]:
        if "send" in step:
            text = payload_text(step.get("payload", {}))
            for _ in range(step.get("repeat", 1)):
                # Turn through the three dispatch forms so each is compared.
                variant = (variant + 1) % 3
                if "agent" in step:
                    sent = (session.dispatch_outcome_as, session.dispatch_view_outcome_as,
                            session.dispatch_view_delta_outcome_as)[variant]
                    outcome = attempt(lambda: sent(step["send"], text, step["agent"]))
                else:
                    read = (session.dispatch_outcome, session.dispatch_view_outcome,
                            session.dispatch_view_delta_outcome)[variant]
                    outcome = attempt(lambda: read(step["send"], text))
                steps.append({"payload": text, "outcome": outcome})
                if "error" in outcome:
                    return {"steps": steps, "fatal": True}
        elif "checkpoint" in step:
            checkpoint = attempt(session.save)
            steps.append({"checkpoint": checkpoint})
        elif "resume" in step and checkpoint is not None and "text" in checkpoint:
            saved = checkpoint["text"]
            resumed = attempt(lambda: NativeSession.restore(source, saved))
            steps.append({"resume": resumed.get("error", "ok")})
            if "error" not in resumed:
                session = resumed["text"]
    return {"steps": steps, "end": ending(session, source)}


def main(files: List[str]) -> None:
    out = []
    for file in files:
        doc = json.loads(Path(file).read_bytes())

        def source_of(name: str) -> str:
            return (Path(file).parent / name).read_bytes().decode("utf-8")

        file_source = source_of(doc["source"])
        entry: Result = {"file": file, "program": program(file_source), "scenarios": []}
        for item in doc["scenarios"]:
            source = source_of(item["source"]) if "source" in item else file_source
            try:
                result = scenario(item, source)
            except (RuntimeFailure, Trap) as error:
                result = {"load": str(error)}
            entry["scenarios"].append({"id": item["id"], **result})
        out.append(entry)
    sys.stdout.write(json.dumps(out, separators=(",", ":"), ensure_ascii=False))


if __name__ == "__main__":
    main(sys.argv[1:])
