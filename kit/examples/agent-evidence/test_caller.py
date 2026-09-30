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
    assessment_permits,
)

HERE = os.path.dirname(os.path.abspath(__file__))
PROGRAM = os.path.join(HERE, "assessment.cav")


def verdict(snapshot):
    return snapshot["bindings"]["assessment"]["verdict"]


class ResponseLevels(unittest.TestCase):
    # A valid response that must not be read as the operation's success: the
    # request was handled, and the event was refused.
    def test_a_handled_request_can_carry_a_rejected_event(self):
        response = {"id": 2, "ok": True, "outcome": "rejected", "origin": "input",
                    "code": "bound_exceeded", "sequence": 1}
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
        attempt = Attempt(self.server)
        attempt.require("observe", {"confidence": confidence})
        attempt.require("assess")
        return attempt.finish(assessment_permits)

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

    # Evidence supporting an approval is withdrawn: the program's withdrawal
    # policy applies, and the old approval is not usable.
    def test_withdrawn_evidence_reopens_the_approval_by_the_programs_policy(self):
        self.assertTrue(self.approve().succeeded)
        grounds = self.server.snapshot()["commitment_grounds"]["assessment@1"]

        retracted = self.server.dispatch("retract")
        self.assertTrue(retracted.accepted)
        snapshot = self.server.snapshot()
        self.assertEqual([entry["evidence"] for entry in snapshot["withdrawals"]], ["observations@1"])
        # The program reopens the assessment; its original grounds stay as they were.
        self.assertEqual(verdict(snapshot), "reopened")
        self.assertEqual(snapshot["commitment_grounds"]["assessment@1"], grounds)
        self.assertEqual(snapshot["decision_journal"][-1]["change"], "reopened")

        # Assessing again on the withdrawn observation is refused, so the attempt fails.
        attempt = Attempt(self.server)
        refused = attempt.require("assess")
        result = attempt.finish(assessment_permits)
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
        result = attempt.finish(lambda snapshot, attempt: True)

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
        self.assertEqual(snapshot["commitment_grounds"]["assessment@2"]["evidence"], ["observations@2"])
        self.assertEqual(snapshot["commitment_grounds"]["assessment@1"]["evidence"], ["observations@1"])

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
    # server keeps serving.
    def test_a_malformed_request_changes_nothing(self):
        self.assertTrue(self.approve().succeeded)
        before = self.server.save()
        for op, fields in (("dispatch", {"event": "observe", "payload": {"confidence": 50}, "extra": True}),
                           ("frobnicate", {})):
            with self.assertRaises(CaveatRequestError) as caught:
                self.server.request(op, **fields)
            self.assertEqual(caught.exception.response["ok"], False)
            self.assertEqual(caught.exception.response["error"]["kind"], "request")
        self.assertEqual(self.server.save(), before)
        self.assertTrue(self.server.dispatch("observe", {"confidence": 20}).accepted)


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
