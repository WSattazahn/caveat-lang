"""The caller's rule against a real `caveat serve` process.

Run from this directory with bytecode off, so nothing is written here:

    python3 -B -m unittest -v test_caller

CAVEAT_COMMAND chooses how `caveat` is started; see caller.default_command.
The tests of a server that breaks the protocol start a stand-in written to a
temporary directory instead.
"""
import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

import caller
from caller import (
    Attempt,
    CaveatLoadError,
    CaveatProtocolError,
    CaveatRequestError,
    CaveatServer,
    CaveatSessionFailed,
    Dispatch,
    assess_answer,
    assessment_permits,
)

HERE = os.path.dirname(os.path.abspath(__file__))
PROGRAM = os.path.join(HERE, "assessment.cav")


def verdict(snapshot):
    return snapshot["bindings"]["assessment"]["verdict"]


def always(snapshot, attempt):
    """A test that permits anything, so only the required operations decide."""
    return True


class ResponseLevels(unittest.TestCase):
    # A valid response that must not be read as the operation's success: the
    # request was handled, and the event was refused.
    def test_a_handled_request_can_carry_a_rejected_event(self):
        response = {"id": 2, "ok": True, "outcome": "rejected", "origin": "input", "code": "bound_exceeded",
                    "message": "confidence must be finite and in 0..100", "sequence": 1}
        result = Dispatch.from_response("observe", response)
        self.assertFalse(result.accepted)
        self.assertEqual((result.origin, result.code, result.sequence), ("input", "bound_exceeded", 1))

    # Anything the outcome contract does not name fails closed, each for its own reason.
    def test_an_unknown_outcome_or_origin_is_not_a_rejection(self):
        for response, reason in (
            ({"id": 1, "ok": True, "outcome": "maybe", "sequence": 1}, "unknown dispatch outcome"),
            ({"id": 1, "ok": True, "outcome": "rejected", "origin": "elsewhere", "code": "x", "sequence": 1},
             "unknown origin or no code"),
            ({"id": 1, "ok": True, "outcome": "rejected", "origin": "input", "sequence": 1},
             "unknown origin or no code"),
            ({"id": 1, "ok": True, "outcome": "accepted"}, "no sequence"),
            ({"id": 1, "ok": False, "outcome": "accepted", "sequence": 1}, "not a handled dispatch"),
        ):
            with self.subTest(response=response), self.assertRaisesRegex(CaveatProtocolError, reason):
                Dispatch.from_response("observe", response)


class Attempts(unittest.TestCase):
    def setUp(self):
        self.server = CaveatServer(PROGRAM, cwd=HERE)
        self.addCleanup(self.server.close)

    def approve(self, confidence=85):
        return assess_answer(self.server, confidence)

    # A required operation is rejected after an earlier approval: this attempt
    # did not succeed, even if the old approval remains visible.
    def test_a_rejected_required_operation_fails_the_attempt_despite_an_old_approval(self):
        first = self.approve()
        self.assertTrue(first.succeeded)
        before = self.server.save()

        attempt = Attempt(self.server)
        refused = attempt.require("observe", {"confidence": 250})
        self.assertIsNone(attempt.require("assess"), "nothing more is sent once a required operation fails")
        result = attempt.finish(assessment_permits)

        self.assertEqual((refused.outcome, refused.origin, refused.code), ("rejected", "input", "bound_exceeded"))
        self.assertFalse(result.succeeded)
        self.assertIs(result.failed, refused)
        self.assertEqual(result.operations, [refused])
        # The old approval is still what the session shows; the refusal changed nothing.
        self.assertEqual(verdict(result.snapshot), "approved")
        self.assertEqual(self.server.save(), before)
        # Even an assessment test that accepted the old approval could not
        # make this attempt succeed: the rejected operation decides it.
        permissive = attempt.finish(always)
        self.assertTrue(permissive.permits)
        self.assertFalse(permissive.succeeded)

    # Evidence supporting an approval is withdrawn: the program's withdrawal
    # policy applies, and the old approval is not usable.
    def test_withdrawn_evidence_reopens_the_approval_by_the_programs_policy(self):
        # One attempt commits an approval, then withdraws the evidence under it.
        attempt = Attempt(self.server)
        attempt.require("observe", {"confidence": 85})
        attempt.require("assess")
        grounds = self.server.snapshot()["commitment_grounds"]["assessment@1"]
        attempt.require("retract")
        result = attempt.finish(assessment_permits)

        self.assertIsNone(result.failed, "every required operation was accepted")
        snapshot = result.snapshot
        self.assertEqual([entry["evidence"] for entry in snapshot["withdrawals"]], ["observations@1"])
        # The program reopens the assessment; its original grounds stay as they were.
        self.assertEqual(verdict(snapshot), "reopened")
        self.assertEqual(snapshot["commitment_grounds"]["assessment@1"], grounds)
        self.assertEqual(snapshot["decision_journal"][-1]["change"], "reopened")
        # The approval was made in this attempt, and it is no longer in force.
        self.assertFalse(result.permits)
        self.assertFalse(result.succeeded)

        # Withdrawing it again, or assessing again on it, is refused.
        again = Attempt(self.server)
        refused = again.require("retract")
        self.assertEqual((refused.origin, refused.code, refused.message),
                         ("policy", "reject", "The latest observation is already withdrawn."))
        again = Attempt(self.server)
        refused = again.require("assess")
        result = again.finish(assessment_permits)
        self.assertEqual((refused.origin, refused.code, refused.message),
                         ("policy", "reject", "The latest observation is withdrawn."))
        self.assertFalse(result.succeeded)
        self.assertFalse(result.permits)

    # A test deliberately requests a forbidden action and gets the expected
    # refusal: the test passed; the requested action did not succeed.
    def test_a_forbidden_action_is_refused(self):
        self.assertTrue(self.approve().succeeded)
        before = self.server.save()

        attempt = Attempt(self.server)
        refused = attempt.require("erase")
        result = attempt.finish(always)

        self.assertEqual((refused.outcome, refused.origin, refused.code, refused.message),
                         ("rejected", "policy", "reject", "Evidence is withdrawn with a reason, never erased."))
        self.assertFalse(result.succeeded)
        self.assertIs(result.failed, refused)
        self.assertEqual(self.server.save(), before)

    # A later, separate attempt receives valid replacement evidence and an
    # accepted reassessment: that attempt is judged on its own.
    def test_a_later_attempt_with_replacement_evidence_is_judged_on_its_own(self):
        self.assertTrue(self.approve().succeeded)
        self.server.dispatch("retract")
        failed = Attempt(self.server)
        failed.require("assess")
        self.assertFalse(failed.finish(assessment_permits).succeeded)

        result = self.approve(confidence=90)
        self.assertTrue(result.succeeded, result.failed)
        snapshot = result.snapshot
        self.assertEqual(verdict(snapshot), "approved")
        # Grounds are sets; compare them as sets.
        self.assertEqual(set(snapshot["commitment_grounds"]["assessment@2"]["evidence"]), {"observations@2"})
        self.assertEqual(set(snapshot["commitment_grounds"]["assessment@1"]["evidence"]), {"observations@1"})

    # Level 3 reads the current assessment: an approval committed before this
    # attempt does not permit what this attempt asks for.
    def test_an_earlier_approval_does_not_stand_in_for_this_attempt(self):
        self.assertTrue(self.approve().succeeded)
        attempt = Attempt(self.server)
        result = attempt.finish(assessment_permits)
        self.assertIsNone(result.failed, "no required operation failed; only the assessment decides")
        self.assertEqual(verdict(result.snapshot), "approved")
        self.assertFalse(result.permits)
        self.assertFalse(result.succeeded)

    # A malformed request is refused as a request: nothing changes, and the
    # server keeps serving. A line the server cannot read, such as a payload
    # holding NaN, is answered with an id of null and is refused the same way.
    def test_a_malformed_request_changes_nothing(self):
        self.assertTrue(self.approve().succeeded)
        before = self.server.save()
        for op, fields, unread in (("dispatch", {"event": "observe", "payload": {"confidence": 50}, "extra": True}, False),
                                   ("dispatch", {"event": "observe", "payload": {"confidence": float("nan")}}, True),
                                   ("frobnicate", {}, False)):
            with self.assertRaises(CaveatRequestError) as caught:
                self.server.request(op, **fields)
            self.assertEqual(caught.exception.response["ok"], False)
            self.assertEqual(caught.exception.response["error"]["kind"], "request")
            self.assertEqual(caught.exception.response["id"] is None, unread)
        self.assertEqual(self.server.save(), before)
        self.assertTrue(self.server.dispatch("observe", {"confidence": 20}).accepted)

    # A required operation refused as a request fails the attempt, even when
    # the application catches the error and finishes the attempt anyway.
    def test_a_refused_request_fails_the_attempt(self):
        attempt = Attempt(self.server)
        attempt.require("observe", {"confidence": 85})
        sequence = self.server.snapshot()["sequence"]
        with self.assertRaises(CaveatRequestError) as caught:
            attempt.require("assess", ["not", "an", "object"])
        self.assertIsNone(attempt.require("assess"), "nothing more is sent once a required operation fails")
        self.assertEqual(self.server.snapshot()["sequence"], sequence, "the later assess was not sent")
        result = attempt.finish(always)
        self.assertIs(result.failed, caught.exception)
        self.assertTrue(result.permits)
        self.assertFalse(result.succeeded)

    # The program keeps at most 16 observations. After that, observe is
    # refused by the limit, and every attempt in this session fails.
    def test_a_full_history_refuses_more_evidence(self):
        for _ in range(16):
            self.assertTrue(self.server.dispatch("observe", {"confidence": 20}).accepted)
        result = self.approve(confidence=95)
        self.assertEqual((result.failed.origin, result.failed.code), ("limit", "history_limit"))
        self.assertFalse(result.succeeded)


class FailingOperation:
    """A server whose dispatch raises: an error that is not a refused request."""

    def __init__(self, error):
        self.error = error
        self.dispatched = []

    def snapshot(self):
        return {"sequence": 0, "bindings": {"assessment": {"verdict": "none"}}, "decision_journal": []}

    def dispatch(self, event, payload=None):
        self.dispatched.append(event)
        raise self.error


class RequiredOperations(unittest.TestCase):
    # Any error from a required operation fails the attempt, not only a
    # refused request, even when the application catches it and finishes.
    def test_any_error_from_a_required_operation_fails_the_attempt(self):
        server = FailingOperation(RuntimeError("the tool crashed"))
        attempt = Attempt(server)
        with self.assertRaises(RuntimeError) as caught:
            attempt.require("observe", {"confidence": 85})
        self.assertIsNone(attempt.require("assess"), "nothing more is sent once a required operation fails")
        self.assertEqual(server.dispatched, ["observe"])
        result = attempt.finish(always)
        self.assertIs(result.failed, caught.exception)
        self.assertTrue(result.permits)
        self.assertFalse(result.succeeded)


def snapshot_with(verdict_text, journal):
    return {"bindings": {"assessment": {"verdict": verdict_text}}, "decision_journal": journal}


def entry(decision, change, sequence):
    return {"decision": decision, "commitment": f"{decision}@1", "change": change, "sequence": sequence}


class Attempted:
    """An attempt that began after sequence 2."""

    start_sequence = 2


class Permits(unittest.TestCase):
    """assessment_permits reads only the snapshot. Each test isolates one of its conditions: the
    snapshot it permits differs from the one it refuses in that condition alone."""

    def test_the_verdict_must_be_approved(self):
        journal = [entry("assessment", "committed", 3)]
        self.assertTrue(assessment_permits(snapshot_with("approved", journal), Attempted()))
        self.assertFalse(assessment_permits(snapshot_with("reopened", journal), Attempted()))

    def test_the_approval_must_be_committed_during_this_attempt(self):
        self.assertTrue(assessment_permits(snapshot_with("approved", [entry("assessment", "committed", 3)]), Attempted()))
        self.assertFalse(assessment_permits(snapshot_with("approved", [entry("assessment", "committed", 2)]), Attempted()))

    # An approval is shown, and nothing was committed: a program whose
    # displayed verdict says approved by mistake permits nothing.
    def test_a_verdict_without_a_commitment_permits_nothing(self):
        self.assertFalse(assessment_permits(snapshot_with("approved", []), Attempted()))

    # Another decision committed in this attempt says nothing about the assessment.
    def test_only_the_assessment_counts(self):
        earlier = [entry("assessment", "committed", 1)]
        self.assertFalse(assessment_permits(snapshot_with("approved", earlier + [entry("other", "committed", 3)]),
                                            Attempted()))

    # A reopening in this attempt is not an approval made in it, even for a
    # program whose displayed verdict ignores reopening.
    def test_only_a_commitment_counts(self):
        earlier = [entry("assessment", "committed", 1)]
        self.assertFalse(assessment_permits(snapshot_with("approved", earlier + [entry("assessment", "reopened", 3)]),
                                            Attempted()))


# A stand-in for `caveat serve`, written to a temporary directory by the tests
# below. Its script says what to write: a ready line, then one reply per
# request ("$id" becomes the request's id; None reads the request and answers
# nothing), then whether to stay running ("hang") or exit with a status. With
# "grandchild", it first starts a process of its own that stays alive, as npx
# starts caveat under a shell. With "deaf", it stops reading requests before
# it writes the ready line. It records when it wrote that line in "ready.time".
STAND_IN = r'''
import json, os, subprocess, sys, time
with open(sys.argv[1], encoding="utf-8") as handle:
    script = json.load(handle)
if script["grandchild"]:
    pid_file = os.path.join(os.path.dirname(sys.argv[1]), "grandchild.pid")
    code = ("import os, sys, time; open(sys.argv[1] + '.tmp', 'w').write(str(os.getpid())); "
            "os.replace(sys.argv[1] + '.tmp', sys.argv[1]); time.sleep(60)")
    subprocess.Popen([sys.executable, "-B", "-I", "-S", "-c", code, pid_file],
                     stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    deadline = time.monotonic() + 10
    while not os.path.exists(pid_file) and time.monotonic() < deadline:
        time.sleep(0.02)
if script["deaf"]:
    os.close(0)  # sys.stdin.close() would leave the descriptor open.
def write(line):
    sys.stdout.write(line + "\n")
    sys.stdout.flush()
if script["ready"] is not None:
    with open(os.path.join(os.path.dirname(sys.argv[1]), "ready.time"), "w", encoding="utf-8") as handle:
        handle.write(repr(time.time()))
    write(script["ready"])
for reply in script["replies"]:
    request = sys.stdin.readline()
    if not request or reply is None:
        break
    write(reply.replace('"$id"', json.dumps(json.loads(request).get("id"))))
if script["end"] == "hang":
    time.sleep(60)
sys.exit(0 if script["end"] == "hang" else script["end"])
'''

READY = json.dumps({"schema": "caveat-serve/0.1", "ready": True, "program": "assessment.cav"})
SNAPSHOT = json.dumps({"id": "$id", "ok": True, "snapshot": {"sequence": 0}})


def alive(pid):
    """Whether process `pid` is running. Never signals it."""
    if os.name == "nt":
        # os.kill on Windows terminates the process; ask tasklist instead.
        listed = subprocess.run(["tasklist", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"],
                                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                                universal_newlines=True, errors="replace").stdout
        return f'"{pid}"' in listed
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    try:
        # A stopped process that is not yet reaped is a zombie: it is not running.
        with open(f"/proc/{pid}/stat", encoding="utf-8") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] != "Z"
    except OSError:
        return True


class GivingUp(unittest.TestCase):
    """A server that breaks the protocol, or does not end in time, is given up on,
    and nothing from its launch is left running."""

    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.directory = directory.name
        self.addCleanup(self.stop_grandchild)

    def start(self, *replies, ready=READY, end="hang", grandchild=True, deaf=False, timeout=15.0):
        """Start CaveatServer on the stand-in. Returns the server, or raises what its constructor raises.

        Only the tests of a server that does not answer, or does not exit, in time wait for the
        timeout; they shorten it. The others keep it long, so a slow start is not taken for one.
        """
        stand_in = os.path.join(self.directory, "stand_in.py")
        with open(stand_in, "w", encoding="utf-8") as handle:
            handle.write(STAND_IN)
        script = os.path.join(self.directory, "script.json")
        with open(script, "w", encoding="utf-8") as handle:
            json.dump({"ready": ready, "replies": list(replies), "end": end, "grandchild": grandchild,
                       "deaf": deaf}, handle)
        server = CaveatServer(PROGRAM, command=[sys.executable, "-B", "-I", "-S", stand_in, script], cwd=HERE, timeout=timeout)
        self.addCleanup(server.close)
        return server

    def grandchild(self):
        """The process the stand-in started. It writes its own id, so it was running."""
        with open(os.path.join(self.directory, "grandchild.pid"), encoding="utf-8") as handle:
            return int(handle.read())

    def assertStopped(self, pid):
        deadline = time.monotonic() + 5
        while alive(pid) and time.monotonic() < deadline:
            time.sleep(0.1)
        self.assertFalse(alive(pid), f"process {pid}, started under the server, is still running")

    def stop_grandchild(self):
        """Clean up after a failed test, so no process outlives the suite."""
        if not os.path.exists(os.path.join(self.directory, "grandchild.pid")):
            return
        pid = self.grandchild()
        if alive(pid):
            if os.name == "nt":
                subprocess.run(["taskkill", "/F", "/PID", str(pid)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            else:
                os.kill(pid, signal.SIGKILL)

    def test_a_server_that_writes_nothing_is_stopped_with_everything_it_started(self):
        with self.assertRaisesRegex(CaveatProtocolError, "wrote nothing for 2.0 seconds"):
            self.start(ready=None, timeout=2.0)
        self.assertStopped(self.grandchild())

    # The caller stops the launch as soon as it gives up; it does not wait out
    # the timeout first. The time is counted from the line, so a slow start
    # does not count.
    def test_a_first_line_that_is_not_json_stops_everything_it_started(self):
        with self.assertRaisesRegex(CaveatProtocolError, "a line that is not JSON"):
            self.start(ready="Starting caveat...", timeout=60.0)
        given_up = time.time()
        with open(os.path.join(self.directory, "ready.time"), encoding="utf-8") as handle:
            written = float(handle.read())
        self.assertLess(given_up - written, 30.0, "the caller waited for the server instead of stopping it")
        self.assertStopped(self.grandchild())

    def test_a_line_that_is_not_an_object_stops_everything_it_started(self):
        with self.assertRaisesRegex(CaveatProtocolError, "a line that is not an object"):
            self.start(ready="[1]")
        self.assertStopped(self.grandchild())

    def test_a_ready_line_of_another_protocol_stops_everything_it_started(self):
        with self.assertRaisesRegex(CaveatProtocolError, "no ready line"):
            self.start(ready=json.dumps({"schema": "caveat-serve/9", "ready": True}))
        self.assertStopped(self.grandchild())

    def test_a_server_that_ends_without_a_ready_line_is_reported(self):
        with self.assertRaisesRegex(CaveatProtocolError, r"no ready line from caveat serve: None \(status 0\)"):
            self.start(ready=None, end=0, grandchild=False)

    def test_a_response_to_another_request_stops_everything_it_started(self):
        server = self.start(json.dumps({"id": 99, "ok": True, "outcome": "accepted", "sequence": 1}))
        with self.assertRaisesRegex(CaveatProtocolError, "does not echo id 1"):
            server.dispatch("observe", {"confidence": 85})
        self.assertStopped(self.grandchild())

    # Only a request error may carry an id of null, and only a request error
    # for this line may carry another id.
    def test_a_refusal_for_another_request_breaks_the_protocol(self):
        server = self.start(json.dumps({"id": 99, "ok": False, "error": {"kind": "request", "message": "x"}}))
        with self.assertRaisesRegex(CaveatProtocolError, "does not echo id 1"):
            server.request("snapshot")
        self.assertStopped(self.grandchild())

    def test_an_id_of_null_without_ok_false_breaks_the_protocol(self):
        server = self.start(json.dumps({"id": None, "error": {"kind": "request", "message": "x"}}))
        with self.assertRaisesRegex(CaveatProtocolError, "does not echo id 1"):
            server.request("snapshot")
        self.assertStopped(self.grandchild())

    def test_an_id_of_null_on_a_session_failure_breaks_the_protocol(self):
        server = self.start(json.dumps({"id": None, "ok": False, "error": {"kind": "fatal", "message": "x"}}))
        with self.assertRaisesRegex(CaveatProtocolError, "does not echo id 1"):
            server.request("snapshot")
        self.assertStopped(self.grandchild())

    # The same null-id refusal, well formed, is a refused request, and the server keeps serving.
    def test_an_id_of_null_on_a_request_error_is_a_refused_request(self):
        server = self.start(json.dumps({"id": None, "ok": False, "error": {"kind": "request", "message": "unreadable"}}),
                            SNAPSHOT, json.dumps({"id": "$id", "ok": True}), end=0, grandchild=False)
        with self.assertRaisesRegex(CaveatRequestError, "unreadable"):
            server.request("snapshot")
        self.assertEqual(server.snapshot(), {"sequence": 0})
        self.assertEqual(server.close(), 0)

    def test_a_server_that_stops_reading_requests_stops_everything_it_started(self):
        server = self.start(deaf=True)
        with self.assertRaisesRegex(CaveatProtocolError, "stopped reading requests"):
            server.request("snapshot")
        self.assertStopped(self.grandchild())

    # When the launch cannot be stopped as a whole, the caller stops its first
    # process and says so; it does not report a clean stop.
    def test_a_launch_that_cannot_be_stopped_whole_is_reported(self):
        if os.name == "nt":
            refusal, problem = mock.patch.object(caller.subprocess, "run", side_effect=OSError("no taskkill")),                 "taskkill could not run: no taskkill"
        else:
            refusal, problem = mock.patch.object(caller.os, "killpg", side_effect=PermissionError("not permitted")),                 "stopping its process group failed: not permitted"
        with refusal, self.assertRaises(CaveatProtocolError) as caught:
            self.start(ready="Starting caveat...", grandchild=False, timeout=30.0)
        self.assertIn(f"({problem}; processes started under it may still be running)", str(caught.exception))

    def test_a_server_that_ends_without_answering_is_reported(self):
        server = self.start(None, end=0, grandchild=False)
        with self.assertRaisesRegex(CaveatProtocolError, "ended without answering"):
            server.request("snapshot")

    # An outcome the contract does not name is a protocol failure, not a rejection.
    def test_an_unknown_outcome_stops_everything_it_started(self):
        server = self.start(json.dumps({"id": "$id", "ok": True, "outcome": "maybe", "sequence": 1}))
        with self.assertRaisesRegex(CaveatProtocolError, "unknown dispatch outcome"):
            server.dispatch("observe", {"confidence": 85})
        self.assertStopped(self.grandchild())

    # A failed session is not a refused request: it fails the attempt, the
    # server has ended, and nothing more can be sent.
    def test_a_failed_session_fails_the_attempt_and_ends_the_server(self):
        server = self.start(SNAPSHOT, json.dumps({"id": "$id", "ok": False, "error": {"kind": "fatal", "message": "boom"}}),
                            end=1, grandchild=False)
        attempt = Attempt(server)
        with self.assertRaises(CaveatSessionFailed) as caught:
            attempt.require("observe", {"confidence": 85})
        self.assertEqual((caught.exception.kind, caught.exception.status), ("fatal", 1))
        self.assertIs(attempt.failed, caught.exception)
        self.assertIsNone(attempt.require("assess"), "nothing more is sent once a required operation fails")
        with self.assertRaisesRegex(CaveatProtocolError, "the server has ended"):
            attempt.finish(always)

    # A failed session whose server does not exit is stopped, with everything it started.
    def test_a_failed_session_that_does_not_exit_is_stopped(self):
        server = self.start(json.dumps({"id": "$id", "ok": False, "error": {"kind": "fatal", "message": "boom"}}),
                            timeout=10.0)
        with self.assertRaisesRegex(CaveatSessionFailed, "fatal: boom"):
            server.request("snapshot")
        self.assertStopped(self.grandchild())

    # On POSIX the launch's process group outlives its first process, so what a
    # server that exits by itself leaves running is stopped too.
    @unittest.skipIf(os.name == "nt", "taskkill finds a process tree only while its first process runs")
    def test_a_failed_session_leaves_nothing_running_on_posix(self):
        server = self.start(json.dumps({"id": "$id", "ok": False, "error": {"kind": "fatal", "message": "boom"}}), end=1)
        with self.assertRaises(CaveatSessionFailed) as caught:
            server.request("snapshot")
        self.assertEqual(caught.exception.status, 1)
        self.assertStopped(self.grandchild())

    # A server that answers close and then keeps running is stopped, and close says so.
    def test_a_close_that_does_not_finish_in_time_stops_everything_it_started(self):
        server = self.start(json.dumps({"id": "$id", "ok": True}), timeout=10.0)
        with self.assertRaisesRegex(CaveatProtocolError, "did not exit within 10.0 seconds of close"):
            server.close()
        self.assertStopped(self.grandchild())

    # A server that closes correctly is not stopped: it exits by itself, with status 0.
    def test_a_server_that_closes_correctly_exits_by_itself(self):
        server = self.start(json.dumps({"id": "$id", "ok": True}), end=0, grandchild=False)
        self.assertEqual(server.close(), 0)


class Loading(unittest.TestCase):
    # A program that does not load is announced by the ready line, and the
    # server exits with 2.
    def test_a_program_that_does_not_load_is_reported(self):
        with tempfile.TemporaryDirectory() as directory:
            broken = os.path.join(directory, "broken.cav")
            with open(broken, "w", encoding="utf-8") as handle:
                handle.write("this is not caveat;\n")
            with self.assertRaises(CaveatLoadError) as caught:
                CaveatServer(broken, cwd=HERE)
        self.assertEqual(caught.exception.status, 2)
        self.assertTrue(str(caught.exception))


if __name__ == "__main__":
    unittest.main()
