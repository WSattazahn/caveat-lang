"""An agent that checks a claim against a file and records its decision in Caveat.

The agent reads a file with a tool, judges how strongly the file supports a
claim, and approves the claim when the evidence is strong. Then the file
changes. The agent reads it again, finds the claim no longer holds, and
withdraws its earlier observation. Caveat keeps the approval's original
grounds and reopens it, so the decision log says what the approval rested on
and why it no longer stands.

    python claim_checker.py            # a keyword judge; runs anywhere
    python claim_checker.py --claude   # Claude judges, through the claude CLI

The program is kit/examples/agent-evidence/assessment.cav.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import tempfile
from pathlib import Path
from typing import Callable

import caveat_lang as caveat

PROGRAM = Path(__file__).resolve().parents[2] / "kit" / "examples" / "agent-evidence" / "assessment.cav"
CLAIM = "The package has no runtime dependencies."

Judge = Callable[[str, str], int]


def read_file(path: Path) -> str:
    """The agent's tool."""
    return path.read_text(encoding="utf-8")


def keyword_judge(claim: str, text: str) -> int:
    """How strongly `text` supports `claim`, 0 to 100, by keywords alone."""
    if "depends on" in text.lower():
        return 10
    return 90 if "no runtime dependencies" in text.lower() else 50


def claude_judge(claim: str, text: str) -> int:
    """How strongly `text` supports `claim`, 0 to 100, as Claude judges it."""
    prompt = (f"Claim: {claim}\n\nDocument:\n{text}\n\n"
              "How strongly does the document support the claim, from 0 to 100? "
              "Reply with the number only.")
    reply = subprocess.run(["claude", "-p", prompt], capture_output=True, text=True, timeout=180, check=True).stdout
    found = re.search(r"\d+", reply)
    if found is None:
        raise ValueError(f"the judge did not reply with a number: {reply!r}")
    return min(100, int(found.group()))


def log(step: str, outcome: caveat.Accepted | caveat.Rejected) -> None:
    if isinstance(outcome, caveat.Rejected):
        print(f"  {step}: refused ({outcome.code}): {outcome.message}")
    else:
        print(f"  {step}: accepted")


def main(judge: Judge) -> None:
    with tempfile.TemporaryDirectory() as folder, caveat.open(PROGRAM.read_text(encoding="utf-8")) as session:
        notes = Path(folder) / "RELEASE_NOTES.md"
        notes.write_text("# 0.1.0\n\nThe package has no runtime dependencies.\n", encoding="utf-8")

        confidence = judge(CLAIM, read_file(notes))
        print(f"First reading: the notes support the claim at {confidence}.")
        log("observe", session.dispatch("observe", {"confidence": confidence}))
        log("assess", session.dispatch("assess"))
        print(f"  verdict: {session.snapshot()['bindings']['assessment']['verdict']}")

        notes.write_text("# 0.1.1\n\nThe package now depends on numpy at run time.\n", encoding="utf-8")
        confidence = judge(CLAIM, read_file(notes))
        print(f"Second reading: the notes support the claim at {confidence}.")
        if confidence < 70:
            log("retract", session.dispatch("retract"))
        print(f"  verdict: {session.snapshot()['bindings']['assessment']['verdict']}")

        print("Decision log:")
        for revision in caveat.decision_history(session.snapshot(), "assessment"):
            committed, reopened = revision["committed"], revision["reopened"]
            print(f"  {revision['revision']} is {revision['status']}: committed at event "
                  f"{committed['sequence']} on {', '.join(committed['because'])}", end="")
            if reopened:
                print(f"; reopened at event {reopened['sequence']} because of {', '.join(reopened['because'])}", end="")
            print()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="An agent that checks a claim and records its decision in Caveat.")
    parser.add_argument("--claude", action="store_true", help="judge with Claude through the claude CLI")
    main(claude_judge if parser.parse_args().claude else keyword_judge)
