"""Drive one `caveat serve` session from Python, and judge workflow attempts.

Standard library only; Python 3.9 or later. The protocol is Serve 0.1
(docs/reference/spec/caveat-serve-0.1.md in the installed package), and the
outcomes follow Dispatch outcomes 0.1.

A response is read at three levels, and each answers a different question:

1. Was the request handled?  `ok` true, or `ok` false with an error kind.
   Kind "request" means the line was refused and nothing changed; any other
   kind means the session failed and the server has ended.
2. Was the event accepted?  `outcome` "accepted" or "rejected". A rejected
   event is a successful request that changed nothing.
3. Does the application's decision permit the intended result?  Only the
   current snapshot says, read after the attempt.

The caller's rule, which `Attempt` carries out:

    A workflow attempt succeeds only when its required operations succeed and
    its relevant, current CAVEAT assessment permits the intended result. A
    stored earlier assessment cannot substitute for a rejected required
    operation.
"""
from __future__ import annotations

import json
import os
import queue
import shlex
import shutil
import subprocess
import tempfile
import threading
from dataclasses import dataclass, field
from typing import Any, Callable, Dict, List, Optional

SERVE_SCHEMA = "caveat-serve/0.1"
# Origins in Dispatch outcomes 0.1. An unknown origin is a protocol failure,
# never a rejection to accept.
ORIGINS = ("policy", "input", "evaluation", "limit", "host")


class CaveatProtocolError(Exception):
    """The server said something Serve 0.1 does not allow."""


class CaveatLoadError(Exception):
    """The program did not load: the ready line was false, and the server exits with 2."""

    def __init__(self, message: str, status: Optional[int]):
        super().__init__(message)
        self.status = status


class CaveatRequestError(Exception):
    """Level 1, kind "request": the request was refused. Nothing changed, and the server keeps serving."""

    def __init__(self, message: str, response: Dict[str, Any]):
        super().__init__(message)
        self.response = response


class CaveatSessionFailed(Exception):
    """Level 1, any other kind (usually "fatal"): the session cannot be used, and the server exits with 1."""

    def __init__(self, kind: str, message: str, status: Optional[int]):
        super().__init__(f"{kind}: {message}")
        self.kind = kind
        self.status = status


@dataclass(frozen=True)
class Dispatch:
    """Level 2: what one handled dispatch did to the session."""

    event: str
    outcome: str
    sequence: int
    origin: Optional[str] = None
    code: Optional[str] = None
    message: Optional[str] = None

    @property
    def accepted(self) -> bool:
        return self.outcome == "accepted"

    @classmethod
    def from_response(cls, event: str, response: Dict[str, Any]) -> "Dispatch":
        """Read a dispatch response that has `ok: true`. Anything else fails closed."""
        if response.get("ok") is not True:
            raise CaveatProtocolError(f"not a handled dispatch: {response!r}")
        outcome = response.get("outcome")
        sequence = response.get("sequence")
        if not isinstance(sequence, int):
            raise CaveatProtocolError(f"dispatch response has no sequence: {response!r}")
        if outcome == "accepted":
            return cls(event, outcome, sequence)
        if outcome == "rejected":
            origin, code = response.get("origin"), response.get("code")
            if origin not in ORIGINS or not isinstance(code, str):
                raise CaveatProtocolError(f"rejection with an unknown origin or no code: {response!r}")
            return cls(event, outcome, sequence, origin, code, response.get("message"))
        raise CaveatProtocolError(f"unknown dispatch outcome: {response!r}")


def default_command() -> List[str]:
    """The command that runs `caveat`, without `serve PROGRAM`.

    CAVEAT_COMMAND overrides it, as a JSON array of strings such as
    ["node", "/path/to/caveat.mjs"], or as words split like a shell would.
    Otherwise the installed package's command runs through npx, which is
    npx.cmd on Windows.
    """
    configured = os.environ.get("CAVEAT_COMMAND")
    if configured:
        if configured.lstrip().startswith("["):
            words = json.loads(configured)
            if not words or not all(isinstance(word, str) for word in words):
                raise ValueError("CAVEAT_COMMAND must be a JSON array of strings")
            return words
        return shlex.split(configured)
    npx = shutil.which("npx")
    if npx is None:
        raise FileNotFoundError("npx was not found; install Node, or set CAVEAT_COMMAND")
    return [npx, "--no-install", "caveat"]


class CaveatServer:
    """One `caveat serve PROGRAM` process, holding one session."""

    def __init__(self, program: str, command: Optional[List[str]] = None, cwd: Optional[str] = None,
                 timeout: float = 60.0):
        self._next_id = 0
        # How long to wait for any one line before giving up on the server.
        self._timeout = timeout
        self._stderr = tempfile.TemporaryFile()
        self._process = subprocess.Popen(
            [*(command or default_command()), "serve", program],
            cwd=cwd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self._stderr,
            encoding="utf-8",
            bufsize=1,
        )
        # Lines are read on a thread, so a server that never answers ends in
        # an error after the timeout instead of a hang.
        self._lines: "queue.Queue[str]" = queue.Queue()
        self._stderr_text: Optional[str] = None
        self._reader = threading.Thread(target=self._pump, daemon=True)
        self._reader.start()
        ready = self._read_line()
        if ready is None or ready.get("schema") != SERVE_SCHEMA:
            self._finish()
            raise CaveatProtocolError(f"no ready line from caveat serve: {ready!r}{self._errors()}")
        if ready.get("ready") is not True:
            status = self._finish()
            message = (ready.get("error") or {}).get("message", "")
            raise CaveatLoadError(message, status)
        self.ready = ready

    def __enter__(self) -> "CaveatServer":
        return self

    def __exit__(self, *exc: Any) -> None:
        self.close()

    # Level 1: send one request and read its response.
    def request(self, op: str, **fields: Any) -> Dict[str, Any]:
        """Send one request. Returns a response with `ok: true`, or raises."""
        if self._process.poll() is not None:
            raise CaveatProtocolError("the server has ended")
        self._next_id += 1
        request_id = self._next_id
        self._process.stdin.write(json.dumps({"id": request_id, "op": op, **fields}) + "\n")
        self._process.stdin.flush()
        response = self._read_line()
        if response is None:
            status = self._finish()
            raise CaveatProtocolError(f"the server ended without answering (status {status}){self._errors()}")
        if response.get("id") != request_id:
            raise CaveatProtocolError(f"response {response!r} does not echo id {request_id}")
        if response.get("ok") is True:
            return response
        error = response.get("error") or {}
        if error.get("kind") == "request":
            raise CaveatRequestError(error.get("message", ""), response)
        status = self._finish()
        raise CaveatSessionFailed(error.get("kind", "unknown"), error.get("message", ""), status)

    # Level 2: an event is accepted or rejected; both are handled requests.
    def dispatch(self, event: str, payload: Optional[Dict[str, Any]] = None) -> Dispatch:
        fields: Dict[str, Any] = {"event": event}
        if payload is not None:
            fields["payload"] = payload
        return Dispatch.from_response(event, self.request("dispatch", **fields))

    # Level 3 reads the current session.
    def snapshot(self) -> Dict[str, Any]:
        return self.request("snapshot")["snapshot"]

    def explain(self) -> Dict[str, Any]:
        return self.request("explain")["report"]

    def dependents(self, of: str) -> Dict[str, Any]:
        return self.request("dependents", of=of)["report"]

    def save(self) -> str:
        return self.request("save")["save"]

    def restore(self, save: str) -> int:
        return self.request("restore", save=save)["sequence"]

    def close(self) -> Optional[int]:
        """Close the session and wait for the server. Returns its exit status."""
        if self._process.poll() is None:
            try:
                self.request("close")
            except (CaveatProtocolError, OSError):
                pass
        return self._finish()

    def _pump(self) -> None:
        try:
            for line in self._process.stdout:
                self._lines.put(line)
        finally:
            self._process.stdout.close()
            self._lines.put("")

    def _read_line(self) -> Optional[Dict[str, Any]]:
        try:
            line = self._lines.get(timeout=self._timeout)
        except queue.Empty:
            self._process.kill()
            self._finish()
            raise CaveatProtocolError(f"caveat serve wrote nothing for {self._timeout} seconds{self._errors()}")
        if not line:
            return None
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise CaveatProtocolError(f"caveat serve wrote a line that is not JSON: {line!r}") from error
        if not isinstance(value, dict):
            raise CaveatProtocolError(f"caveat serve wrote a line that is not an object: {line!r}")
        return value

    def _finish(self) -> Optional[int]:
        if self._process.stdin and not self._process.stdin.closed:
            try:
                self._process.stdin.close()
            except OSError:
                pass
        try:
            status = self._process.wait(timeout=30)
        except subprocess.TimeoutExpired:
            self._process.kill()
            status = self._process.wait()
        self._reader.join(timeout=5)
        if self._stderr_text is None:
            self._stderr.seek(0)
            self._stderr_text = self._stderr.read().decode("utf-8", "replace").strip()
            self._stderr.close()
        return status

    def _errors(self) -> str:
        """What the server wrote to standard error, once it has ended."""
        return f"\n{self._stderr_text}" if self._stderr_text else ""


@dataclass
class AttemptResult:
    """The verdict on one attempt, and why."""

    succeeded: bool
    operations: List[Dispatch]
    failed: Optional[Dispatch]
    permits: bool
    snapshot: Dict[str, Any]


@dataclass
class Attempt:
    """One workflow attempt: its required operations, then the current assessment.

    `require` sends a required operation. Once one is rejected, the attempt
    has failed; later required operations are not sent, because they would
    run on a session the attempt did not bring about. `finish` reads the
    current snapshot and applies `permits` to it.
    """

    server: CaveatServer
    operations: List[Dispatch] = field(default_factory=list)
    failed: Optional[Dispatch] = None

    def __post_init__(self) -> None:
        # The sequence before this attempt, so `permits` can tell this
        # attempt's assessment from an earlier one.
        self.start_sequence = self.server.snapshot()["sequence"]

    def require(self, event: str, payload: Optional[Dict[str, Any]] = None) -> Optional[Dispatch]:
        if self.failed is not None:
            return None
        result = self.server.dispatch(event, payload)
        self.operations.append(result)
        if not result.accepted:
            self.failed = result
        return result

    def finish(self, permits: Callable[[Dict[str, Any], "Attempt"], bool]) -> AttemptResult:
        snapshot = self.server.snapshot()
        allowed = bool(permits(snapshot, self))
        return AttemptResult(
            succeeded=self.failed is None and allowed,
            operations=list(self.operations),
            failed=self.failed,
            permits=allowed,
            snapshot=snapshot,
        )


# This example's assessment: what permits the application to use the answer.
def assessment_permits(snapshot: Dict[str, Any], attempt: Attempt) -> bool:
    """The answer may be used when the assessment in force is an approval made in this attempt.

    The verdict is the program's own displayed value. Requiring that the
    approval in force was committed during this attempt keeps an earlier
    approval from standing in for this attempt's evidence.
    """
    if snapshot["bindings"]["assessment"]["verdict"] != "approved":
        return False
    committed = [entry for entry in snapshot["decision_journal"]
                 if entry["decision"] == "assessment" and entry["change"] == "committed"]
    return bool(committed) and committed[-1]["sequence"] > attempt.start_sequence
