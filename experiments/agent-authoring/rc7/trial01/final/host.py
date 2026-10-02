"""One session per task; use only a successful, freshly assessed AttemptResult."""
from pathlib import Path
import hashlib
import json
import unittest
from caller import Attempt, CaveatServer, assessment_permits

PROGRAM = Path(__file__).with_name("task.cav")


def permits(snapshot, attempt):
    if not assessment_permits(snapshot, attempt):
        return False
    current = snapshot["decision_series"]["assessment"]["current"]
    latest = snapshot["reading_streams"]["observations"]["current"]
    grounds = snapshot["commitment_grounds"][current]
    return set(grounds["evidence"]) == {latest, "reference"} and "withdrawn" not in grounds["caveats"]


class Task:
    def __init__(self, task_id):
        self.task_id = task_id
        self.server = CaveatServer(str(PROGRAM))

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.server.close()

    def assess(self, confidence, correct_latest=False):
        attempt = Attempt(self.server)
        if correct_latest:
            attempt.require("retract")
        attempt.require("observe", {"confidence": confidence})
        attempt.require("consult")
        attempt.require("assess")
        return attempt.finish(permits)

    def reassess(self):
        attempt = Attempt(self.server)
        attempt.require("assess")
        return attempt.finish(permits)

    def save(self):
        return {"task_id": self.task_id,
                "source_sha256": hashlib.sha256(PROGRAM.read_bytes()).hexdigest(),
                "save": self.server.save()}

    def restore(self, envelope):
        if envelope["task_id"] != self.task_id:
            raise ValueError("A save belongs to a different task.")
        if envelope["source_sha256"] != hashlib.sha256(PROGRAM.read_bytes()).hexdigest():
            raise ValueError("A save belongs to different source.")
        return self.server.restore(envelope["save"])


class IntegrationTests(unittest.TestCase):
    def grounds(self, snapshot, revision, evidence, caveats=()):
        ground = snapshot["commitment_grounds"][revision]
        self.assertEqual(set(ground["evidence"]), set(evidence))
        self.assertEqual(set(ground["caveats"]), set(caveats))

    def test_second_supporting_observation_reassesses(self):
        with Task("task-A") as task:
            first = task.assess(85)
            second = task.assess(90)
            self.assertTrue(first.succeeded and second.succeeded)
            self.grounds(first.snapshot, "assessment@1", ["observations@1", "reference"])
            self.grounds(second.snapshot, "assessment@2", ["observations@2", "reference"])
            self.assertEqual(second.snapshot["commitment_grounds"]["assessment@1"],
                             first.snapshot["commitment_grounds"]["assessment@1"])
            self.assertEqual(second.snapshot["decision_journal"][1]["because"], ["observations@2"])

    def test_correction_withdraws_then_replaces(self):
        with Task("task-A") as task:
            first = task.assess(85)
            corrected = task.assess(88, correct_latest=True)
            self.assertTrue(corrected.succeeded)
            self.assertEqual(corrected.snapshot["withdrawals"][0]["evidence"], "observations@1")
            self.assertEqual(corrected.snapshot["decision_journal"][1]["because"], ["recheck"])
            self.assertEqual(corrected.snapshot["commitment_grounds"]["assessment@1"],
                             first.snapshot["commitment_grounds"]["assessment@1"])
            self.grounds(corrected.snapshot, "assessment@2", ["observations@2", "reference"])

    def test_rejected_required_observation_and_retry(self):
        with Task("task-A") as task:
            self.assertTrue(task.assess(85).succeeded)
            before = task.server.save()
            failed = task.assess(250)
            self.assertFalse(failed.succeeded)
            self.assertEqual([op.status for op in failed.operations], ["rejected", "not sent", "not sent"])
            self.assertEqual((failed.failed.origin, failed.failed.code), ("input", "bound_exceeded"))
            self.assertEqual(failed.snapshot["bindings"]["assessment"]["verdict"], "approved")
            self.assertEqual(task.server.save(), before)
            retry = task.assess(92)
            self.assertTrue(retry.succeeded)
            self.assertFalse(failed.succeeded)
            self.grounds(retry.snapshot, "assessment@2", ["observations@2", "reference"])

    def test_save_restore_continue_same_task(self):
        with Task("task-A") as task:
            task.assess(85)
            task.assess(90)
            envelope, before = task.save(), task.server.snapshot()
        with Task("task-A") as resumed:
            resumed.restore(envelope)
            self.assertEqual(resumed.server.snapshot(), before)
            self.assertEqual(resumed.save(), envelope)
            self.assertFalse(resumed.reassess().succeeded)
            result = resumed.assess(93)
            self.assertTrue(result.succeeded)
            self.grounds(result.snapshot, "assessment@3", ["observations@3", "reference"])

    def test_late_source_qualification_freezes_old_grounds(self):
        with Task("task-A") as task:
            first = task.assess(85)
            self.assertTrue(task.server.dispatch("qualify_reference").accepted)
            reopened = task.server.snapshot()
            self.assertEqual(reopened["bindings"]["assessment"]["verdict"], "reopened")
            self.assertEqual(reopened["decision_journal"][-1]["because"], ["reference"])
            self.assertEqual(set(reopened["decision_journal"][-1]["caveats"]), {"outdated"})
            self.assertEqual(reopened["commitment_grounds"]["assessment@1"],
                             first.snapshot["commitment_grounds"]["assessment@1"])
            result = task.reassess()
            self.assertTrue(result.succeeded)
            self.grounds(result.snapshot, "assessment@1", ["observations@1", "reference"])
            self.grounds(result.snapshot, "assessment@2", ["observations@1", "reference"], ["outdated"])
            self.assertEqual(result.snapshot["commitment_bases"]["assessment@2"]["value"], 75)
            folder = Path(__file__).with_name("receipts")
            folder.mkdir(exist_ok=True)
            for name, data in [("before-qualification", first.snapshot), ("after-qualification", reopened),
                               ("after-reassessment", result.snapshot), ("explain", task.server.explain()),
                               ("dependents-reference", task.server.dependents("reference"))]:
                (folder / (name + ".json")).write_text(json.dumps(data, indent=2), encoding="utf-8")

    def test_qualified_source_can_make_score_insufficient(self):
        with Task("task-A") as task:
            self.assertTrue(task.assess(75).succeeded)
            self.assertTrue(task.server.dispatch("qualify_reference").accepted)
            insufficient = task.reassess()
            self.assertFalse(insufficient.succeeded)
            self.assertEqual(insufficient.failed.message, "The effective confidence is below 70.")
            self.assertTrue(task.assess(85).succeeded)

    def test_new_task_does_not_inherit_and_cannot_restore_old_save(self):
        with Task("task-A") as old:
            self.assertTrue(old.assess(90).succeeded)
            saved = old.save()
        with Task("task-B") as new:
            self.assertEqual(new.server.snapshot()["bindings"]["assessment"]["verdict"], "none")
            with self.assertRaises(ValueError):
                new.restore(saved)
            self.assertFalse(new.reassess().succeeded)
            self.assertTrue(new.assess(80).succeeded)


if __name__ == "__main__":
    unittest.main(verbosity=2)
