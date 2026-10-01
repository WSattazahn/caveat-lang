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

In full: an attempt succeeds only when every required operation has
explicitly recorded accepted completion, no required operation has failed,
and the relevant assessment permits success. An earlier approval cannot cover
an incomplete operation.
"""
from __future__ import annotations

import json
import os
import queue
import shutil
import signal
import subprocess
import tempfile
import threading
from dataclasses import dataclass
from typing import Any, Callable, Dict, List, Optional, Tuple, Union

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
    ["node", "/path/to/caveat.mjs"]. Otherwise the installed package's command
    runs through npx, which is npx.cmd on Windows.
    """
    configured = os.environ.get("CAVEAT_COMMAND")
    if configured:
        try:
            words = json.loads(configured)
        except json.JSONDecodeError:
            words = None
        if not isinstance(words, list) or not words or not all(isinstance(word, str) for word in words):
            raise ValueError('CAVEAT_COMMAND must be a JSON array of strings, such as ["node", "caveat.mjs"]')
        return words
    npx = shutil.which("npx")
    if npx is None:
        raise FileNotFoundError("npx was not found; install Node, or set CAVEAT_COMMAND")
    return [npx, "--no-install", "caveat"]


class CaveatServer:
    """One `caveat serve PROGRAM` process, holding one session.

    When the caller gives up on the server, it stops every process the launch
    started, not only the one it started itself: `npx` runs `caveat` under a
    shell (`cmd.exe` on Windows), and stopping only the first process would
    leave the server running.
    """

    def __init__(self, program: str, command: Optional[List[str]] = None, cwd: Optional[str] = None,
                 timeout: float = 60.0):
        self._next_id = 0
        # One request at a time: two threads never interleave on the pipe.
        self._lock = threading.Lock()
        # Set when a request was interrupted before its response was read.
        self._out_of_step = False
        # How long to wait for any one line, or for the server to exit, before
        # giving up on it.
        self._timeout = timeout
        # Set when stopping the server may have left a process running.
        self._stop_problem: Optional[str] = None
        self._stderr = tempfile.TemporaryFile()
        self._process = subprocess.Popen(
            [*(command or default_command()), "serve", program],
            cwd=cwd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self._stderr,
            encoding="utf-8",
            bufsize=1,
            # On POSIX the launch gets a process group of its own, so giving up
            # can stop all of it. Windows stops the tree with taskkill instead.
            start_new_session=os.name != "nt",
        )
        # Lines are read on a thread, so a server that never answers ends in
        # an error after the timeout instead of a hang.
        self._lines: "queue.Queue[str]" = queue.Queue()
        self._stderr_text: Optional[str] = None
        self._reader = threading.Thread(target=self._pump, daemon=True)
        self._reader.start()
        ready = self._read_line()
        # `ready` is true or false; anything else is not a ready line.
        if ready is None or ready.get("schema") != SERVE_SCHEMA or not isinstance(ready.get("ready"), bool):
            raise self._fail(f"no ready line from caveat serve: {ready!r}")
        if not ready["ready"]:
            status = self._end()
            message = (ready.get("error") or {}).get("message", "")
            raise CaveatLoadError(message + self._stop_note(), status)
        self.ready = ready

    def __enter__(self) -> "CaveatServer":
        return self

    def __exit__(self, *exc: Any) -> None:
        self.close()

    # Level 1: send one request and read its response.
    def request(self, op: str, **fields: Any) -> Dict[str, Any]:
        """Send one request. Returns a response with `ok: true`, or raises."""
        with self._lock:
            return self._request(op, fields)

    def _request(self, op: str, fields: Dict[str, Any]) -> Dict[str, Any]:
        if self._out_of_step:
            raise CaveatProtocolError("a request was interrupted before its response was read, so the pipe is "
                                      "out of step; the caller gave up on this server and stopped it. "
                                      "Start a new server.")
        if self._process.poll() is not None:
            raise self._fail("the server has ended")
        self._next_id += 1
        request_id = self._next_id
        line = json.dumps({"id": request_id, "op": op, **fields}) + "\n"
        try:
            self._process.stdin.write(line)
            self._process.stdin.flush()
            response = self._read_line()
        except CaveatProtocolError:
            raise  # The caller has given up on the server already.
        except OSError as error:
            raise self._fail(f"the server stopped reading requests: {error}") from None
        except BaseException:
            # Interrupted, for example by KeyboardInterrupt, between writing
            # the request and reading its response: the next line on the pipe
            # may answer this request. Give up on the server, and let the
            # interruption go on.
            self._out_of_step = True
            self._stop()
            self._finish()
            raise
        if response is None:
            raise self._fail("the server ended without answering")
        error = response.get("error")
        if not isinstance(error, dict):
            error = {}
        # Requests are answered in order, so an id of null answers this line:
        # the server could not read it, for example a payload holding NaN.
        unread = response.get("id") is None and response.get("ok") is False and error.get("kind") == "request"
        if response.get("id") != request_id and not unread:
            raise self._fail(f"response {response!r} does not echo id {request_id}")
        if response.get("ok") is True:
            return response
        # Otherwise `ok` is false and the error names its kind.
        if response.get("ok") is not False or not isinstance(error.get("kind"), str):
            raise self._fail(f"not a Serve 0.1 response: {response!r}")
        if error.get("kind") == "request":
            raise CaveatRequestError(error.get("message", ""), response)
        # The session failed. The server exits with 1 by itself; `_end` stops
        # what is left of the launch.
        status = self._end()
        raise CaveatSessionFailed(error["kind"], error.get("message", "") + self._stop_note(), status)

    # Level 2: an event is accepted or rejected; both are handled requests.
    def dispatch(self, event: str, payload: Optional[Dict[str, Any]] = None) -> Dispatch:
        fields: Dict[str, Any] = {"event": event}
        if payload is not None:
            fields["payload"] = payload
        response = self.request("dispatch", **fields)
        try:
            return Dispatch.from_response(event, response)
        except CaveatProtocolError as error:
            # An outcome the contract does not name: the server cannot be trusted.
            raise self._fail(str(error)) from None

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
        """Close the session and wait for the server. Returns its exit status.

        A server that does not answer `close`, or does not exit in time
        afterwards, is stopped, and CaveatProtocolError is raised.
        """
        if self._process.poll() is None:
            self.request("close")
            self._close_input()
            try:
                self._process.wait(timeout=self._timeout)
            except subprocess.TimeoutExpired:
                raise self._fail(f"caveat serve did not exit within {self._timeout} seconds of close") from None
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
            raise self._fail(f"caveat serve wrote nothing for {self._timeout} seconds")
        if not line:
            return None
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            raise self._fail(f"caveat serve wrote a line that is not JSON: {line!r}")
        if not isinstance(value, dict):
            raise self._fail(f"caveat serve wrote a line that is not an object: {line!r}")
        return value

    def _fail(self, message: str) -> CaveatProtocolError:
        """The caller gives up on the server: stop the launch, and say what happened."""
        self._stop()
        status = self._finish()
        return CaveatProtocolError(f"{message} (status {status}){self._stop_note()}{self._errors()}")

    def _end(self) -> Optional[int]:
        """The session is over, and the server exits by itself: wait for it, then stop what is left."""
        status = self._finish()
        self._stop()
        return status

    def _stop(self) -> None:
        """Stop every process the launch started, including those under the first one."""
        if os.name != "nt":
            # The launch's own process group, which outlives its first process.
            try:
                os.killpg(self._process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass  # Nothing of it is left.
            except PermissionError as error:
                self._stop_directly(f"stopping its process group failed: {error}")
            return
        if self._process.poll() is not None:
            # taskkill finds the tree through a running first process only.
            # One that has exited ended by itself, after what it started: npx
            # and the shell under it each wait for the process they start.
            return
        # The command may be a .cmd file, such as npx.cmd, run by cmd.exe;
        # stopping the whole tree also stops caveat under it.
        try:
            done = subprocess.run(["taskkill", "/T", "/F", "/PID", str(self._process.pid)],
                                  stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
                                  universal_newlines=True, errors="replace")
        except OSError as error:
            self._stop_directly(f"taskkill could not run: {error}")
            return
        # A failure is reported even when the first process has exited since:
        # taskkill may have stopped it and failed on a process under it.
        if done.returncode != 0:
            self._stop_directly(f"taskkill failed with status {done.returncode}: {done.stderr.strip()}")

    def _stop_directly(self, problem: str) -> None:
        """The tree could not be stopped: stop the first process, and remember why."""
        self._process.kill()  # Does nothing to a process that has exited.
        self._stop_problem = f"{problem}; processes started under it may still be running"

    def _stop_note(self) -> str:
        return f" ({self._stop_problem})" if self._stop_problem else ""

    def _close_input(self) -> None:
        if self._process.stdin and not self._process.stdin.closed:
            try:
                self._process.stdin.close()
            except OSError:
                pass

    def _finish(self) -> Optional[int]:
        """Wait for the server to exit, stopping it if it does not in time, and release what it held."""
        self._close_input()
        try:
            status = self._process.wait(timeout=self._timeout)
        except subprocess.TimeoutExpired:
            self._stop()
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


# What finish() reports for each required operation.
ACCEPTED = "accepted"  # Its acceptance is recorded.
REJECTED = "rejected"  # The server handled it and refused it; the session is unchanged.
# Sending raised, or was still under way when the attempt finished. This is
# NOT a rejection: the server may or may not have applied the operation.
UNCONFIRMED = "unconfirmed"
NOT_SENT = "not sent"  # An earlier operation failed, or the attempt finished first.
# Before it has an outcome, an operation is registered, then sending.
REGISTERED = "registered"
SENDING = "sending"
# How finish() reports an operation that has no outcome yet.
FINAL_STATUS = {REGISTERED: NOT_SENT, SENDING: UNCONFIRMED}


class AttemptFinished(Exception):
    """The attempt is finished: nothing more is registered or sent in it. A retry is a new Attempt."""


class Operation:
    """A required operation, registered by `Attempt.begin`. `send` dispatches it once, in any thread.

    `status`, `dispatch` and `error` record what happened to it, even after
    the attempt is finished; the finished attempt's result does not change.
    """

    def __init__(self, attempt: "Attempt", event: str, payload: Optional[Dict[str, Any]]):
        self.attempt = attempt
        self.event = event
        self.payload = payload
        self.status = REGISTERED
        self.dispatch: Optional[Dispatch] = None
        self.error: Optional[BaseException] = None

    def send(self) -> Optional[Dispatch]:
        """Dispatch the operation and record its outcome.

        Sends nothing, and returns None, when an earlier operation of the
        attempt has failed. Raises AttemptFinished, without contacting the
        server, when the attempt is finished.
        """
        attempt = self.attempt
        with attempt._lock:
            if self.status != REGISTERED:
                raise RuntimeError(f"send() was called already for {self.event}")
            if attempt._finished:
                self.status = NOT_SENT
                raise AttemptFinished(f"{self.event} was not sent: the attempt is finished")
            if attempt._failed is not None:
                self.status = NOT_SENT
                return None
            self.status = SENDING
        try:
            result = attempt.server.dispatch(self.event, self.payload)
        except BaseException as error:
            # Any error, and an interruption such as KeyboardInterrupt or a
            # cancelled task, fails the attempt, and goes on to the caller.
            attempt._record(self, UNCONFIRMED, None, error)
            raise
        attempt._record(self, ACCEPTED if result.accepted else REJECTED, result, None)
        return result


@dataclass(frozen=True)
class OperationResult:
    """One required operation, as it stood when the attempt finished.

    `status` is ACCEPTED, REJECTED, UNCONFIRMED or NOT_SENT. Unconfirmed is
    not a rejection: the server may or may not have applied the operation.
    """

    event: str
    status: str
    dispatch: Optional[Dispatch] = None
    error: Optional[BaseException] = None


@dataclass(frozen=True)
class AttemptResult:
    """The final verdict on one attempt, and why. Nothing that happens later changes it."""

    succeeded: bool
    operations: Tuple[OperationResult, ...]
    # The first failure: the rejected dispatch, or what sending raised.
    failed: Optional[Union[Dispatch, BaseException]]
    permits: bool
    # The snapshot `permits` was applied to, or None when the server was not read.
    snapshot: Optional[Dict[str, Any]]


class Attempt:
    """One workflow attempt: its required operations, then the current assessment.

    `begin` registers a required operation, and `send`, on what it returns,
    dispatches it; `require` does both. Once one is rejected, or raises, the
    attempt has failed, and later required operations are not sent: they
    would run on a session the attempt did not bring about. `finish` decides,
    once. Registration, outcomes and `finish` take one lock, so their order
    is never in doubt.
    """

    def __init__(self, server: CaveatServer):
        self.server = server
        self._lock = threading.Lock()
        self._operations: List[Operation] = []
        self._failed: Optional[Union[Dispatch, BaseException]] = None
        self._finished = False
        self._result: Optional[AttemptResult] = None
        # The sequence before this attempt, so `permits` can tell this
        # attempt's assessment from an earlier one.
        self.start_sequence = server.snapshot()["sequence"]

    @property
    def failed(self) -> Optional[Union[Dispatch, BaseException]]:
        with self._lock:
            return self._failed

    def begin(self, event: str, payload: Optional[Dict[str, Any]] = None) -> Operation:
        """Register a required operation, in this thread, before anything sends it."""
        with self._lock:
            if self._finished:
                raise AttemptFinished(f"{event} was not registered: the attempt is finished")
            operation = Operation(self, event, payload)
            self._operations.append(operation)
            return operation

    def require(self, event: str, payload: Optional[Dict[str, Any]] = None) -> Optional[Dispatch]:
        """Register a required operation and send it, in this thread."""
        return self.begin(event, payload).send()

    def _record(self, operation: Operation, status: str, dispatch: Optional[Dispatch],
                error: Optional[BaseException]) -> None:
        with self._lock:
            operation.status, operation.dispatch, operation.error = status, dispatch, error
            # The first failure fails the attempt. After finish, the outcome
            # is kept on the operation only.
            if status != ACCEPTED and self._failed is None and not self._finished:
                self._failed = dispatch if error is None else error

    def finish(self, permits: Callable[[Dict[str, Any], "Attempt"], bool]) -> AttemptResult:
        """Finish the attempt and return its verdict. Called again, it returns the same result."""
        with self._lock:
            if self._result is not None:
                return self._result
            # From here on, nothing more is registered or sent in this attempt.
            self._finished = True
            # An operation still registered or sending has no outcome, and a
            # worker may hold the pipe. After an unconfirmed one, what the
            # session holds is not known. Either way, the server is not read.
            unanswered = any(operation.status in (REGISTERED, SENDING, UNCONFIRMED)
                             for operation in self._operations)
            operations = tuple(OperationResult(operation.event, FINAL_STATUS.get(operation.status, operation.status),
                                               operation.dispatch, operation.error)
                               for operation in self._operations)
            failed = self._failed
        snapshot, allowed = None, False
        if not unanswered:
            # Outside the lock: no operation can be sent any more.
            snapshot = self.server.snapshot()
            allowed = bool(permits(snapshot, self))
        # A failed operation is never accepted, so this also requires that none failed.
        succeeded = allowed and all(operation.status == ACCEPTED for operation in operations)
        result = AttemptResult(succeeded, operations, failed, allowed, snapshot)
        with self._lock:
            if self._result is None:
                self._result = result
            return self._result


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


# This example's workflow. Its required operations are named here, once. A
# workflow that needs more evidence adds its `require` calls here, and makes
# `assessment_permits` check that the approval rests on that evidence.
def assess_answer(server: CaveatServer, confidence: float) -> AttemptResult:
    """One attempt: send an observation, ask for the assessment, and judge the result."""
    attempt = Attempt(server)
    attempt.require("observe", {"confidence": confidence})
    attempt.require("assess")
    return attempt.finish(assessment_permits)
