"""The caller's rule against a real `caveat serve` process.

Run from this directory with bytecode off, so nothing is written here:

    python3 -B -m unittest -v test_caller

CAVEAT_COMMAND chooses how `caveat` is started; see caller.default_command.
"""
import os
import tempfile
import unittest

from caller import (
    Attempt,
    CaveatLoadError,
    CaveatProtocolError,
    CaveatRequestError,
    CaveatServer,
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

    # Anything the outcome contract does not name fails closed.
    def test_an_unknown_outcome_or_origin_is_not_a_rejection(self):
        for response in (
            {"id": 1, "ok": True, "outcome": "maybe", "sequence": 1},
            {"id": 1, "ok": True, "outcome": "rejected", "origin": "elsewhere", "code": "x", "sequence": 1},
            {"id": 1, "ok": True, "outcome": "accepted"},
        ):
            with self.assertRaises(CaveatProtocolError):
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
        # The old approval is still what the session shows; the refusal changed nothing.
        self.assertEqual(verdict(result.snapshot), "approved")
        self.assertEqual(self.server.save(), before)
        # Even an assessment test that accepted the old approval could not
        # make this attempt succeed: the rejected operation decides it.
        self.assertFalse(attempt.finish(always).succeeded)

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
        self.assertEqual(verdict(result.snapshot), "approved")
        self.assertFalse(result.permits)
        self.assertFalse(result.succeeded)

    # A malformed request is refused as a request: nothing changes, and the
    # server keeps serving. A line the server cannot read, such as a payload
    # holding NaN, is answered with an id of null and is refused the same way.
    def test_a_malformed_request_changes_nothing(self):
        self.assertTrue(self.approve().succeeded)
        before = self.server.save()
        for op, fields in (("dispatch", {"event": "observe", "payload": {"confidence": 50}, "extra": True}),
                           ("dispatch", {"event": "observe", "payload": {"confidence": float("nan")}}),
                           ("frobnicate", {})):
            with self.assertRaises(CaveatRequestError) as caught:
                self.server.request(op, **fields)
            self.assertEqual(caught.exception.response["ok"], False)
            self.assertEqual(caught.exception.response["error"]["kind"], "request")
        self.assertEqual(self.server.save(), before)
        self.assertTrue(self.server.dispatch("observe", {"confidence": 20}).accepted)

    # A required operation refused as a request fails the attempt, even when
    # the application catches the error and finishes the attempt anyway.
    def test_a_refused_request_fails_the_attempt(self):
        attempt = Attempt(self.server)
        attempt.require("observe", {"confidence": 85})
        with self.assertRaises(CaveatRequestError) as caught:
            attempt.require("assess", ["not", "an", "object"])
        self.assertIsNone(attempt.require("assess"), "nothing more is sent once a required operation fails")
        result = attempt.finish(always)
        self.assertIs(result.failed, caught.exception)
        self.assertFalse(result.succeeded)

    # The program keeps at most 16 observations. After that, observe is
    # refused by the limit, and every attempt in this session fails.
    def test_a_full_history_refuses_more_evidence(self):
        for _ in range(16):
            self.assertTrue(self.server.dispatch("observe", {"confidence": 20}).accepted)
        result = self.approve(confidence=95)
        self.assertEqual((result.failed.origin, result.failed.code), ("limit", "history_limit"))
        self.assertFalse(result.succeeded)


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
