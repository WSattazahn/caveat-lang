"""The caller's rule against a real `caveat serve` process.

Run from this directory with bytecode off, so nothing is written here:

    python3 -B -m unittest -v test_caller

CAVEAT_COMMAND chooses how `caveat` is started; see caller.default_command.
The tests of a server that breaks the protocol start a stand-in written to a
temporary directory instead.
"""
import asyncio
import dataclasses
import json
import os
import signal
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest import mock

import caller
from caller import (
    Attempt,
    AttemptFinished,
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
# How long the tests of a real server wait for any one line. npx can take more
# than the caller's default, 60 seconds, to start `caveat` on a busy machine.
SERVER_TIMEOUT = 300.0


def verdict(snapshot):
    return snapshot["bindings"]["assessment"]["verdict"]


def always(snapshot, attempt):
    """A test that permits anything, so only the required operations decide."""
    return True


def statuses(result):
    return [(operation.event, operation.status) for operation in result.operations]


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
        self.server = CaveatServer(PROGRAM, cwd=HERE, timeout=SERVER_TIMEOUT)
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
        # Even an assessment test that accepts the old approval cannot make
        # this attempt succeed: the rejected operation decides it.
        result = attempt.finish(always)

        self.assertEqual((refused.outcome, refused.origin, refused.code), ("rejected", "input", "bound_exceeded"))
        self.assertTrue(result.permits)
        self.assertFalse(result.succeeded)
        self.assertIs(result.failed, refused)
        self.assertEqual(statuses(result), [("observe", "rejected"), ("assess", "not sent")])
        self.assertIs(result.operations[0].dispatch, refused)
        # The old approval is still what the session shows; the refusal changed nothing.
        self.assertEqual(verdict(result.snapshot), "approved")
        self.assertEqual(self.server.save(), before)
        # The verdict is final: finishing again returns it, whatever the test,
        # and nothing can change it.
        self.assertIs(attempt.finish(assessment_permits), result)
        self.assertIsInstance(result.operations, tuple)
        with self.assertRaises(dataclasses.FrozenInstanceError):
            result.succeeded = True

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
        self.assertEqual(statuses(result), [("erase", "rejected")])
        self.assertTrue(result.permits)
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
        self.assertEqual(statuses(result), [("observe", "accepted"), ("assess", "unconfirmed"), ("assess", "not sent")])
        # After an unconfirmed operation, finish does not read the session.
        self.assertIsNone(result.snapshot)
        self.assertFalse(result.permits)
        self.assertFalse(result.succeeded)

    # A required operation that raises, here after the server committed the
    # approval, fails the attempt: a cancelled task is not a success. The
    # operation is unconfirmed, not rejected: here the server did apply it.
    def test_an_interrupted_required_operation_fails_the_attempt(self):
        dispatch = self.server.dispatch

        def interrupted(event, payload=None):
            result = dispatch(event, payload)
            if event == "assess":
                raise Cancelled("cancelled while reading the response")
            return result

        attempt = Attempt(self.server)
        attempt.require("observe", {"confidence": 85})
        with mock.patch.object(self.server, "dispatch", interrupted), self.assertRaises(Cancelled) as caught:
            attempt.require("assess")
        result = attempt.finish(assessment_permits)
        self.assertEqual(statuses(result), [("observe", "accepted"), ("assess", "unconfirmed")])
        self.assertIs(result.failed, caught.exception)
        self.assertIs(result.operations[1].error, caught.exception)
        self.assertIsNone(result.snapshot)
        self.assertFalse(result.permits)
        self.assertFalse(result.succeeded)
        # The approval was committed in this attempt, and still does not make it succeed.
        snapshot = self.server.snapshot()
        self.assertEqual(verdict(snapshot), "approved")
        self.assertTrue(assessment_permits(snapshot, attempt))

    # assess_answer judges by the assessment test: when it does not permit
    # the result, the attempt fails although every operation was accepted.
    def test_assess_answer_applies_the_assessment_test(self):
        with mock.patch.object(caller, "assessment_permits", return_value=False):
            result = self.approve()
        self.assertEqual([operation.event for operation in result.operations], ["observe", "assess"])
        self.assertIsNone(result.failed)
        self.assertFalse(result.permits)
        self.assertFalse(result.succeeded)

    # The program keeps at most 16 observations. After that, observe is
    # refused by the limit, and every attempt in this session fails.
    def test_a_full_history_refuses_more_evidence(self):
        for _ in range(16):
            self.assertTrue(self.server.dispatch("observe", {"confidence": 20}).accepted)
        result = self.approve(confidence=95)
        self.assertEqual((result.failed.origin, result.failed.code), ("limit", "history_limit"))
        self.assertFalse(result.succeeded)

    # The application finishes the attempt while a worker is still sending a
    # required operation. Its acceptance is not recorded, so the attempt did
    # not succeed, and finish leaves the server alone: the worker owns the pipe.
    def test_an_operation_still_in_flight_at_finish_leaves_the_attempt_unsuccessful(self):
        for confidence, late in ((250, "rejected"), (85, "accepted")):
            with self.subTest(late=late):
                attempt = Attempt(self.server)
                operation = attempt.begin("observe", {"confidence": confidence})
                held = HeldRequests(self.server)
                self.addCleanup(held.gate.set)
                with mock.patch.object(self.server, "request", held):
                    worker, outcome = in_worker(operation.send)
                    self.assertTrue(held.arrived.wait(SERVER_TIMEOUT), "the worker never sent the operation")
                    result = attempt.finish(always)
                    self.assertEqual(held.made, ["dispatch"],
                                     "finish talked to the server while an operation was in flight")
                    held.gate.set()
                    worker.join(SERVER_TIMEOUT)
                self.assertFalse(worker.is_alive())
                self.assertFalse(result.succeeded)
                self.assertFalse(result.permits)
                self.assertIsNone(result.snapshot)
                self.assertIsNone(result.failed)
                self.assertEqual(statuses(result), [("observe", "unconfirmed")])

                # The response arrives after the verdict: it is recorded on
                # the operation, and neither the verdict nor the attempt changes.
                self.assertEqual(operation.status, late)
                self.assertIs(operation.dispatch, outcome["returned"])
                self.assertIsNone(attempt.failed, "a late outcome changed the finished attempt")
                with self.assertRaises(RuntimeError):
                    operation.send()
                self.assertEqual(operation.status, late, "a second send() changed the recorded outcome")
                self.assertIs(attempt.finish(always), result)
                self.assertFalse(result.succeeded)
                self.assertEqual(statuses(result), [("observe", "unconfirmed")])
        # A retry is a new attempt, once nothing of the finished one is still
        # sending. The gate held each request before it was written, so the
        # pipe is in step, and the same server serves it.
        self.assertNotEqual(operation.status, "sending")
        self.assertTrue(self.approve(confidence=90).succeeded)

    # Two callers finish the same attempt while a worker is still sending. The
    # first decides under the lock: a response that arrives before the second
    # call, while the first is still on its way out, cannot make it a success.
    def test_a_concurrent_finish_does_not_change_the_verdict(self):
        attempt = Attempt(self.server)
        attempt.require("observe", {"confidence": 85})
        operation = attempt.begin("assess")
        paused = attempt._lock = PausedAfterRelease(attempt, "finish")
        held = HeldRequests(self.server)
        self.addCleanup(held.gate.set)
        self.addCleanup(paused.gate.set)
        with mock.patch.object(self.server, "request", held):
            worker, _ = in_worker(operation.send)
            self.assertTrue(held.arrived.wait(SERVER_TIMEOUT), "the worker never sent the operation")
            first, outcome = in_worker(lambda: attempt.finish(assessment_permits), name="finish")
            self.assertTrue(paused.arrived.wait(SERVER_TIMEOUT), "the first finish never released the lock")
            held.gate.set()
            worker.join(SERVER_TIMEOUT)
            self.assertEqual(operation.status, "accepted")
            second = attempt.finish(assessment_permits)
            paused.gate.set()
            first.join(SERVER_TIMEOUT)
        self.assertIs(outcome["returned"], second)
        self.assertFalse(second.succeeded)
        self.assertFalse(second.permits)
        self.assertIsNone(second.snapshot)
        self.assertEqual(statuses(second), [("observe", "accepted"), ("assess", "unconfirmed")])

    # The application finishes the attempt after registering an operation for
    # a worker, and before the worker sends it: it is never sent.
    def test_an_operation_not_yet_sent_at_finish_is_never_sent(self):
        attempt = Attempt(self.server)
        operation = attempt.begin("observe", {"confidence": 85})
        sequence = self.server.snapshot()["sequence"]
        held = HeldRequests(self.server)
        held.gate.set()
        with mock.patch.object(self.server, "request", held):
            result = attempt.finish(always)
            self.assertEqual(held.made, [], "finish talked to the server while an operation was registered")
            worker, outcome = in_worker(operation.send)
            worker.join(SERVER_TIMEOUT)
        self.assertFalse(worker.is_alive())
        self.assertFalse(result.succeeded)
        self.assertFalse(result.permits)
        self.assertEqual(statuses(result), [("observe", "not sent")])
        self.assertIsInstance(outcome.get("raised"), AttemptFinished)
        self.assertEqual(operation.status, "not sent")
        self.assertEqual(held.made, [], "the operation reached the server after finish")
        self.assertEqual(self.server.snapshot()["sequence"], sequence)
        self.assertIs(attempt.finish(always), result)
        with self.assertRaises(AttemptFinished):
            attempt.begin("assess")


class PausedAfterRelease:
    """Stands in for an attempt's lock. The thread named `name` is held at `gate` once it has
    released the lock with the attempt finished."""

    def __init__(self, attempt, name):
        self.lock, self.attempt, self.name = attempt._lock, attempt, name
        self.arrived = threading.Event()
        self.gate = threading.Event()

    def __enter__(self):
        return self.lock.__enter__()

    def __exit__(self, *exc):
        self.lock.__exit__(*exc)
        if threading.current_thread().name == self.name and self.attempt._finished and not self.arrived.is_set():
            self.arrived.set()
            self.gate.wait(SERVER_TIMEOUT)
        return False


class HeldRequests:
    """Stands in for a server's `request`: records each request, and holds a dispatch at `gate` before it is written."""

    def __init__(self, server):
        self.request = server.request
        self.made = []
        self.arrived = threading.Event()
        self.gate = threading.Event()

    def __call__(self, op, **fields):
        self.made.append(op)
        if op == "dispatch":
            self.arrived.set()
            if not self.gate.wait(SERVER_TIMEOUT):
                raise RuntimeError("the gate was never opened")
        return self.request(op, **fields)


def in_worker(action, name=None):
    """Run `action` on a worker thread. Returns the thread, and what it returned or raised once it ends."""
    outcome = {}

    def run():
        try:
            outcome["returned"] = action()
        except BaseException as error:
            outcome["raised"] = error

    worker = threading.Thread(target=run, daemon=True, name=name)
    worker.start()
    return worker, outcome


class Interruptions(unittest.TestCase):
    # An earlier attempt's approval is in force when this attempt's required
    # operation is interrupted after its request was written, before its
    # response was read. The application catches the interruption and
    # finishes: the attempt did not succeed, and the old approval, which an
    # assessment test that permits anything would accept, does not cover it.
    def test_an_interrupted_operation_is_unconfirmed_and_no_earlier_approval_covers_it(self):
        for interruption in (KeyboardInterrupt, asyncio.CancelledError):
            with self.subTest(interruption=interruption.__name__), \
                    CaveatServer(PROGRAM, cwd=HERE, timeout=SERVER_TIMEOUT) as server:
                self.assertTrue(assess_answer(server, 85).succeeded)
                attempt = Attempt(server)
                before = server.snapshot()
                with mock.patch.object(server, "_read_line", side_effect=interruption()), \
                        self.assertRaises(interruption) as caught:
                    attempt.require("observe", {"confidence": 90})
                result = attempt.finish(lambda *_: True)

                self.assertFalse(result.succeeded)
                self.assertEqual(statuses(result), [("observe", "unconfirmed")])
                self.assertIs(result.failed, caught.exception)
                self.assertIs(result.operations[0].error, caught.exception)
                self.assertIsNone(result.operations[0].dispatch)
                self.assertIsNone(result.snapshot)
                self.assertFalse(result.permits)
                # The stale-approval trap: before the attempt, an approval
                # committed by the earlier attempt was in force.
                self.assertEqual(verdict(before), "approved")
                committed = [entry["sequence"] for entry in before["decision_journal"]
                             if entry["decision"] == "assessment" and entry["change"] == "committed"]
                self.assertTrue(committed and committed[-1] <= attempt.start_sequence)
                # The pipe is out of step, so the server is not used again.
                with self.assertRaisesRegex(CaveatProtocolError, "out of step"):
                    server.snapshot()


class Cancelled(BaseException):
    """Not an Exception, like asyncio.CancelledError and KeyboardInterrupt."""


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
    # Anything a required operation raises fails the attempt, not only a
    # refused request: an error, an interruption or a cancelled task, even
    # when the application catches it and finishes.
    def test_any_error_from_a_required_operation_fails_the_attempt(self):
        for error in (RuntimeError("the tool crashed"), KeyboardInterrupt(), Cancelled()):
            with self.subTest(error=type(error).__name__):
                server = FailingOperation(error)
                attempt = Attempt(server)
                with self.assertRaises(type(error)) as caught:
                    attempt.require("observe", {"confidence": 85})
                self.assertIs(attempt.failed, caught.exception)
                self.assertIsNone(attempt.require("assess"), "nothing more is sent once a required operation fails")
                self.assertEqual(server.dispatched, ["observe"])
                result = attempt.finish(always)
                self.assertIs(result.failed, caught.exception)
                self.assertEqual(statuses(result), [("observe", "unconfirmed"), ("assess", "not sent")])
                self.assertFalse(result.permits)
                self.assertFalse(result.succeeded)

    # Registration, outcomes and finish take the attempt's one lock: while it
    # is held, none of them goes ahead.
    def test_registration_outcomes_and_finish_take_one_lock(self):
        server = FailingOperation(RuntimeError("the tool crashed"))
        attempt = Attempt(server)
        operation = attempt.begin("observe", {"confidence": 85})
        with attempt._lock:
            workers = [in_worker(action) for action in (lambda: attempt.begin("assess"), operation.send,
                                                        lambda: attempt.finish(always))]
            time.sleep(0.5)
            self.assertEqual([outcome for _, outcome in workers], [{}, {}, {}],
                             "an attempt went ahead without its lock")
        for worker, outcome in workers:
            worker.join(10)
            self.assertTrue(outcome, "an attempt did not go ahead once its lock was released")


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
# nothing), then whether to stay running ("hang") or exit with a status. A
# stand-in that hangs, and the process it starts with "grandchild", sleep for
# an hour: only being stopped ends them in time. The grandchild stays alive
# as caveat does under the shell npx starts. With "deaf", the stand-in stops
# reading requests before it writes the ready line. It records its own id in
# "stand_in.pid", when it wrote the ready line in "ready.time", and the
# requests it read in "requests.json".
STAND_IN = r'''
import json, os, subprocess, sys, time
here = os.path.dirname(sys.argv[1])
with open(sys.argv[1], encoding="utf-8") as handle:
    script = json.load(handle)
def record(name, text):
    # Written whole, then renamed, so a reader never sees half of it.
    with open(os.path.join(here, name + ".tmp"), "w", encoding="utf-8") as handle:
        handle.write(text)
    os.replace(os.path.join(here, name + ".tmp"), os.path.join(here, name))
record("stand_in.pid", str(os.getpid()))
if script["grandchild"]:
    pid_file = os.path.join(here, "grandchild.pid")
    code = ("import os, sys, time; open(sys.argv[1] + '.tmp', 'w').write(str(os.getpid())); "
            "os.replace(sys.argv[1] + '.tmp', sys.argv[1]); time.sleep(3600)")
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
    record("ready.time", repr(time.time()))
    write(script["ready"])
requests = []
for reply in script["replies"]:
    request = sys.stdin.readline()
    if request:
        requests.append(json.loads(request))
        record("requests.json", json.dumps(requests))
    if not request or reply is None:
        break
    write(reply.replace('"$id"', json.dumps(requests[-1].get("id"))))
if script["end"] == "hang":
    time.sleep(3600)
sys.exit(0 if script["end"] == "hang" else script["end"])
'''

READY = json.dumps({"schema": "caveat-serve/0.1", "ready": True, "program": "assessment.cav"})
SNAPSHOT = json.dumps({"id": "$id", "ok": True, "snapshot": {"sequence": 0}})
FATAL = json.dumps({"id": "$id", "ok": False, "error": {"kind": "fatal", "message": "boom"}})
# What a stop that may leave processes running adds to the error.
UNSURE = "; processes started under it may still be running)"

# The tests' own process checks call subprocess.run as it was when they were
# imported, so a test that makes the caller's taskkill fail does not change them.
_run = subprocess.run


def alive(pid):
    """Whether process `pid` is running. Never signals it."""
    if os.name == "nt":
        # os.kill on Windows terminates the process; ask tasklist instead.
        listed = _run(["tasklist", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"],
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


def kill(pid):
    if os.name == "nt":
        _run(["taskkill", "/F", "/PID", str(pid)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    else:
        os.kill(pid, signal.SIGKILL)


def unstoppable():
    """Ways the launch cannot be stopped whole on this system, each with what the caller says about it."""
    if os.name == "nt":
        refused = subprocess.CompletedProcess(["taskkill"], 1, stderr="ERROR: refused\n")
        return [(lambda: mock.patch.object(caller.subprocess, "run", side_effect=OSError("no taskkill")),
                 "taskkill could not run: no taskkill"),
                (lambda: mock.patch.object(caller.subprocess, "run", return_value=refused),
                 "taskkill failed with status 1: ERROR: refused")]
    return [(lambda: mock.patch.object(caller.os, "killpg", side_effect=PermissionError("not permitted")),
             "stopping its process group failed: not permitted")]


class GivingUp(unittest.TestCase):
    """A server that breaks the protocol, or does not end in time, is given up on,
    and nothing from its launch is left running."""

    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.directory = directory.name
        self.launches = []

    def start(self, *replies, ready=READY, end="hang", grandchild=True, deaf=False, timeout=15.0):
        """Start CaveatServer on a new stand-in. Returns the server, or raises what its constructor raises.

        Only the tests of a server that does not answer, or does not exit, in time wait for the
        timeout; they shorten it. The others keep it long, so a slow start is not taken for one.
        """
        launch = os.path.join(self.directory, str(len(self.launches)))
        os.mkdir(launch)
        self.launches.append(launch)
        # A failed test may leave the stand-in running; stop it, so nothing outlives the suite.
        self.addCleanup(self.stop_launch, launch)
        stand_in = os.path.join(launch, "stand_in.py")
        with open(stand_in, "w", encoding="utf-8") as handle:
            handle.write(STAND_IN)
        script = os.path.join(launch, "script.json")
        with open(script, "w", encoding="utf-8") as handle:
            json.dump({"ready": ready, "replies": list(replies), "end": end, "grandchild": grandchild,
                       "deaf": deaf}, handle)
        server = CaveatServer(PROGRAM, command=[sys.executable, "-B", "-I", "-S", stand_in, script], cwd=HERE,
                              timeout=timeout)

        def close():
            self.stop_launch(launch)
            server.close()
        self.addCleanup(close)
        return server

    def recorded(self, name):
        """What the latest stand-in recorded in `name`."""
        with open(os.path.join(self.launches[-1], name), encoding="utf-8") as handle:
            return handle.read()

    def grandchild(self):
        """The process the stand-in started. It writes its own id, so it was running."""
        return int(self.recorded("grandchild.pid"))

    def stand_in(self):
        return int(self.recorded("stand_in.pid"))

    def assertStopped(self, pid):
        deadline = time.monotonic() + 5
        while alive(pid) and time.monotonic() < deadline:
            time.sleep(0.1)
        self.assertFalse(alive(pid), f"process {pid}, from the server's launch, is still running")

    def stop_launch(self, launch):
        for name in ("stand_in.pid", "grandchild.pid"):
            path = os.path.join(launch, name)
            if os.path.exists(path):
                with open(path, encoding="utf-8") as handle:
                    pid = int(handle.read())
                if alive(pid):
                    kill(pid)

    def within(self, seconds, action):
        """Run `action`, which gives up on a stand-in, and return what it raised.

        A stand-in that hangs never exits by itself, so a caller that waits for it instead of
        stopping it would wait for an hour. After `seconds`, the test stops it and fails.
        """
        outcome = {}

        def run():
            try:
                action()
            except BaseException as error:
                outcome["raised"] = error

        worker = threading.Thread(target=run, daemon=True)
        worker.start()
        worker.join(seconds)
        if worker.is_alive():
            for launch in self.launches:
                self.stop_launch(launch)
            worker.join(30)
            self.fail(f"the caller did not give up within {seconds} seconds")
        return outcome.get("raised")

    def test_a_server_that_writes_nothing_is_stopped_with_everything_it_started(self):
        with self.assertRaisesRegex(CaveatProtocolError, "wrote nothing for 2.0 seconds"):
            self.start(ready=None, timeout=2.0)
        self.assertStopped(self.grandchild())
        self.assertStopped(self.stand_in())

    # The caller stops the launch as soon as it gives up; it does not wait out
    # the timeout first. The time is counted from the line, so a slow start
    # does not count.
    def test_a_first_line_that_is_not_json_stops_everything_it_started(self):
        with self.assertRaisesRegex(CaveatProtocolError, "a line that is not JSON"):
            self.start(ready="Starting caveat...", timeout=60.0)
        given_up = time.time()
        self.assertLess(given_up - float(self.recorded("ready.time")), 30.0,
                        "the caller waited for the server instead of stopping it")
        self.assertStopped(self.grandchild())

    def test_a_line_that_is_not_an_object_stops_everything_it_started(self):
        with self.assertRaisesRegex(CaveatProtocolError, "a line that is not an object"):
            self.start(ready="[1]")
        self.assertStopped(self.grandchild())

    # A ready line says ready true or false, in Serve 0.1. Anything else is
    # not one, not even a ready line without `ready`, or with `ready: 1`.
    def test_a_line_that_is_not_a_ready_line_stops_everything_it_started(self):
        for ready in ({"schema": "caveat-serve/9", "ready": True},
                      {"schema": "caveat-serve/0.1", "program": "assessment.cav"},
                      {"schema": "caveat-serve/0.1", "ready": 1, "program": "assessment.cav"}):
            with self.subTest(ready=ready):
                with self.assertRaisesRegex(CaveatProtocolError, "no ready line"):
                    self.start(ready=json.dumps(ready))
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

    # `ok` is exactly true, or exactly false with an error that names its
    # kind. Anything else is neither a result nor a failed session.
    def test_a_response_serve_does_not_allow_stops_everything_it_started(self):
        for response in ({"id": "$id", "ok": 1, "snapshot": {"sequence": 0}},
                         {"id": "$id", "ok": 1, "error": {"kind": "fatal", "message": "x"}},
                         {"id": "$id", "ok": False, "error": {"message": "x"}},
                         {"id": "$id", "ok": False, "error": "x"}):
            with self.subTest(response=response):
                server = self.start(json.dumps(response))
                with self.assertRaisesRegex(CaveatProtocolError, "not a Serve 0.1 response"):
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

    # Requests from two threads take turns on the pipe: the second is not
    # written while the first waits for its response.
    def test_requests_from_two_threads_take_turns_on_the_pipe(self):
        server = self.start(SNAPSHOT, SNAPSHOT, json.dumps({"id": "$id", "ok": True}), end=0, grandchild=False)
        read_line, reads, gate = server._read_line, [], threading.Event()
        self.addCleanup(gate.set)

        def held_read_line():
            reads.append(threading.current_thread().name)
            if len(reads) == 1 and not gate.wait(30):
                raise RuntimeError("the gate was never opened")
            return read_line()

        requests = os.path.join(self.launches[-1], "requests.json")
        with mock.patch.object(server, "_read_line", held_read_line):
            first, first_outcome = in_worker(server.snapshot)
            deadline = time.monotonic() + 30
            while not os.path.exists(requests) and time.monotonic() < deadline:
                time.sleep(0.02)
            second, second_outcome = in_worker(server.snapshot)
            # Long enough for a second request, written without waiting, to reach the stand-in.
            time.sleep(1.0)
            self.assertEqual(len(json.loads(self.recorded("requests.json"))), 1,
                             "a second request was written while the first waited for its response")
            self.assertEqual(len(reads), 1)
            gate.set()
            first.join(30)
            second.join(30)
        self.assertEqual([first_outcome, second_outcome], [{"returned": {"sequence": 0}}] * 2)
        self.assertEqual([request["id"] for request in json.loads(self.recorded("requests.json"))], [1, 2])
        self.assertEqual(server.close(), 0)

    def test_a_server_that_stops_reading_requests_stops_everything_it_started(self):
        server = self.start(deaf=True)
        with self.assertRaisesRegex(CaveatProtocolError, "stopped reading requests"):
            server.request("snapshot")
        self.assertStopped(self.grandchild())

    # When the launch cannot be stopped as a whole, the caller stops its first
    # process at once, and says so; it does not report a clean stop.
    def test_a_launch_that_cannot_be_stopped_whole_is_reported(self):
        for refusal, problem in unstoppable():
            with self.subTest(problem=problem):
                with refusal():
                    raised = self.within(50.0, lambda: self.start(ready="Starting caveat...", grandchild=False,
                                                                  timeout=20.0))
                self.assertIsInstance(raised, CaveatProtocolError)
                self.assertIn(f"({problem}{UNSURE}", str(raised))
                self.assertStopped(self.stand_in())

    # So does an error the server reported, when what is left of its launch
    # could not be stopped whole.
    def test_a_load_error_says_when_the_launch_could_not_be_stopped_whole(self):
        failed = json.dumps({"schema": "caveat-serve/0.1", "ready": False, "error": {"kind": "load", "message": "bad"}})
        for refusal, problem in unstoppable():
            with self.subTest(problem=problem):
                with refusal():
                    raised = self.within(40.0, lambda: self.start(ready=failed, grandchild=False, timeout=8.0))
                self.assertIsInstance(raised, CaveatLoadError)
                self.assertEqual(str(raised), f"bad ({problem}{UNSURE}")
                self.assertStopped(self.stand_in())

    def test_a_failed_session_says_when_the_launch_could_not_be_stopped_whole(self):
        for refusal, problem in unstoppable():
            with self.subTest(problem=problem):
                server = self.start(FATAL, grandchild=False, timeout=8.0)
                with refusal():
                    raised = self.within(40.0, lambda: server.request("snapshot"))
                self.assertIsInstance(raised, CaveatSessionFailed)
                self.assertEqual(str(raised), f"fatal: boom ({problem}{UNSURE}")
                self.assertStopped(self.stand_in())

    def test_a_server_that_ends_without_answering_is_reported(self):
        server = self.start(None, end=0, grandchild=False)
        with self.assertRaisesRegex(CaveatProtocolError, "ended without answering"):
            server.request("snapshot")

    # A request interrupted after it was written, before its response was
    # read, leaves the pipe out of step: the next line read could answer it.
    # The caller lets the interruption go on, gives up on the server and
    # stops everything it started; the server is not used again.
    def test_an_interrupted_request_gives_up_on_the_server(self):
        for interruption in (KeyboardInterrupt, asyncio.CancelledError):
            with self.subTest(interruption=interruption.__name__):
                server = self.start(SNAPSHOT, SNAPSHOT)
                with mock.patch.object(server, "_read_line", side_effect=interruption()), \
                        self.assertRaises(interruption):
                    server.request("snapshot")
                self.assertStopped(self.grandchild())
                self.assertStopped(self.stand_in())
                with self.assertRaisesRegex(CaveatProtocolError, "out of step"):
                    server.request("snapshot")

    # An outcome the contract does not name is a protocol failure, not a rejection.
    def test_an_unknown_outcome_stops_everything_it_started(self):
        server = self.start(json.dumps({"id": "$id", "ok": True, "outcome": "maybe", "sequence": 1}))
        with self.assertRaisesRegex(CaveatProtocolError, "unknown dispatch outcome"):
            server.dispatch("observe", {"confidence": 85})
        self.assertStopped(self.grandchild())

    # A failed session is not a refused request: it fails the attempt, the
    # server has ended, and nothing more can be sent. A server that exits by
    # itself leaves nothing to report about stopping it.
    def test_a_failed_session_fails_the_attempt_and_ends_the_server(self):
        server = self.start(SNAPSHOT, FATAL, end=1, grandchild=False)
        attempt = Attempt(server)
        with self.assertRaises(CaveatSessionFailed) as caught:
            attempt.require("observe", {"confidence": 85})
        self.assertEqual((caught.exception.kind, caught.exception.status), ("fatal", 1))
        self.assertEqual(str(caught.exception), "fatal: boom")
        self.assertIs(attempt.failed, caught.exception)
        self.assertIsNone(attempt.require("assess"), "nothing more is sent once a required operation fails")
        result = attempt.finish(always)
        self.assertEqual(statuses(result), [("observe", "unconfirmed"), ("assess", "not sent")])
        self.assertIs(result.failed, caught.exception)
        self.assertIsNone(result.snapshot)
        self.assertFalse(result.succeeded)
        with self.assertRaisesRegex(CaveatProtocolError, "the server has ended"):
            server.snapshot()

    # A failed session whose server does not exit is stopped, with everything
    # it started, once the timeout has passed.
    def test_a_failed_session_that_does_not_exit_is_stopped(self):
        server = self.start(FATAL, timeout=10.0)
        raised = self.within(40.0, lambda: server.request("snapshot"))
        self.assertIsInstance(raised, CaveatSessionFailed)
        self.assertEqual(str(raised), "fatal: boom")
        self.assertStopped(self.grandchild())
        self.assertStopped(self.stand_in())

    # On POSIX the launch's process group outlives its first process, so what a
    # server that exits by itself leaves running is stopped too.
    @unittest.skipIf(os.name == "nt", "taskkill finds a process tree only while its first process runs")
    def test_a_failed_session_leaves_nothing_running_on_posix(self):
        server = self.start(FATAL, end=1)
        with self.assertRaises(CaveatSessionFailed) as caught:
            server.request("snapshot")
        self.assertEqual(caught.exception.status, 1)
        self.assertStopped(self.grandchild())

    # A server that answers close and then keeps running is stopped, and close says so.
    def test_a_close_that_does_not_finish_in_time_stops_everything_it_started(self):
        server = self.start(json.dumps({"id": "$id", "ok": True}), timeout=10.0)
        raised = self.within(40.0, server.close)
        self.assertIsInstance(raised, CaveatProtocolError)
        self.assertIn("did not exit within 10.0 seconds of close", str(raised))
        self.assertStopped(self.grandchild())

    # A server that closes correctly is asked to, and is not stopped: it exits
    # by itself, with status 0.
    def test_a_server_that_closes_correctly_exits_by_itself(self):
        server = self.start(json.dumps({"id": "$id", "ok": True}), end=0, grandchild=False)
        self.assertEqual(server.close(), 0)
        self.assertEqual(json.loads(self.recorded("requests.json")), [{"id": 1, "op": "close"}])


class Loading(unittest.TestCase):
    # A program that does not load is announced by the ready line, and the
    # server exits with 2.
    def test_a_program_that_does_not_load_is_reported(self):
        with tempfile.TemporaryDirectory() as directory:
            broken = os.path.join(directory, "broken.cav")
            with open(broken, "w", encoding="utf-8") as handle:
                handle.write("this is not caveat;\n")
            with self.assertRaises(CaveatLoadError) as caught:
                CaveatServer(broken, cwd=HERE, timeout=SERVER_TIMEOUT)
        self.assertEqual(caught.exception.status, 2)
        self.assertTrue(str(caught.exception))


if __name__ == "__main__":
    unittest.main()
