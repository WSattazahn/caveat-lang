"""Lifecycle L1-L8 and task isolation against the real Caveat CLI.

Run with python3 -B -m unittest -v test_lifecycle. CAVEAT_COMMAND has the
same JSON-array meaning as in caller.py. The original caller suite is separate.
"""
from pathlib import Path
import tempfile
import unittest

from caller import Attempt, AttemptFinished, CaveatServer, assess_answer, assessment_permits

HERE = Path(__file__).resolve().parent
PROGRAM = HERE / "assessment.cav"
SHIPPED_DECLARATION = "decisions assessment limit 8 reopened by observations opposing answer_supported;"
EVERY_OBSERVATION_DECLARATION = "decisions assessment limit 8 reopened by observations;"


def verdict(snapshot):
    return snapshot["bindings"]["assessment"]["verdict"]


class Lifecycle(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        source = PROGRAM.read_text(encoding="utf-8")
        self.assertEqual(source.count(SHIPPED_DECLARATION), 1)
        self.every_observation = Path(temporary.name) / "every_observation.cav"
        self.every_observation.write_text(
            source.replace(SHIPPED_DECLARATION, EVERY_OBSERVATION_DECLARATION), encoding="utf-8")

    def server(self, program=PROGRAM):
        server = CaveatServer(str(program), cwd=str(HERE), timeout=300.0)
        self.addCleanup(server.close)
        return server

    def assert_refused_unchanged(self, server, event, message):
        snapshot, saved = server.snapshot(), server.save()
        result = server.dispatch(event)
        self.assertEqual((result.outcome, result.origin, result.code, result.message),
                         ("rejected", "policy", "reject", message))
        self.assertEqual(result.sequence, snapshot["sequence"])
        self.assertEqual(server.snapshot(), snapshot)
        self.assertEqual(server.save(), saved, "rejection changed the complete serialized session")
        return result

    def assert_frozen(self, before, after, revision="assessment@1"):
        self.assertEqual(after["commitment_bases"][revision], before["commitment_bases"][revision])
        self.assertEqual(after["commitment_grounds"][revision], before["commitment_grounds"][revision])
        self.assertEqual(after["decision_series"]["assessment"]["revisions"][0],
                         before["decision_series"]["assessment"]["revisions"][0])

    def run_readme_example(self, name):
        """Run the documented Python exactly, resolving its example directory."""
        readme = (HERE / "README.md").read_text(encoding="utf-8")
        marker = "<!-- lifecycle-example: " + name + " -->\n```python\n"
        self.assertEqual(readme.count(marker), 1)
        code = readme.split(marker, 1)[1].split("\n```", 1)[0]
        filename = str(HERE / ("readme_" + name + ".py"))
        exec(compile(code, filename, "exec"), {"__file__": filename})

    def test_l1_duplicate_assess_rejects_without_changing_the_complete_save(self):
        server = self.server()
        self.assertTrue(assess_answer(server, 85).succeeded)
        self.assert_refused_unchanged(server, "assess", "Already assessed.")
        self.assertEqual(len(server.snapshot()["decision_series"]["assessment"]["revisions"]), 1)

    def test_l2_supporting_observation_keeps_the_shipped_revision_in_force(self):
        server = self.server()
        first = assess_answer(server, 85)
        self.assertTrue(first.succeeded)
        self.assertTrue(server.dispatch("observe", {"confidence": 90}).accepted)
        current = server.snapshot()
        self.assertEqual(current["sequence"], 3)
        readings = current["reading_streams"]["observations"]["occurrences"]
        self.assertEqual([(entry["id"], entry["value"], entry["relation"]) for entry in readings],
                         [("observations@1", 85, "supports"), ("observations@2", 90, "supports")])
        self.assertEqual(verdict(current), "approved")
        self.assertEqual(current["decision_series"]["assessment"], first.snapshot["decision_series"]["assessment"])
        self.assertEqual(current["decision_journal"], first.snapshot["decision_journal"])
        self.assert_frozen(first.snapshot, current)
        self.assert_refused_unchanged(server, "assess", "Already assessed.")
        later = Attempt(server).finish(assessment_permits)
        self.assertFalse(later.permits, "an old approval cannot satisfy a new attempt")
        self.assertFalse(later.succeeded)
        self.run_readme_example("shipped")

    def test_l3_every_observation_policy_reopens_then_commits_a_new_revision(self):
        server = self.server(self.every_observation)
        first = assess_answer(server, 85)
        self.assertTrue(first.succeeded)
        attempt = Attempt(server)
        self.assertTrue(attempt.require("observe", {"confidence": 90}).accepted)
        reopened = server.snapshot()
        self.assertEqual(verdict(reopened), "reopened")
        self.assertEqual(reopened["decision_series"]["assessment"]["current"], "assessment@1")
        self.assertEqual(reopened["decision_journal"][-1]["change"], "reopened")
        self.assertEqual(reopened["decision_journal"][-1]["because"], ["observations@2"])
        self.assert_frozen(first.snapshot, reopened)
        self.assertTrue(attempt.require("assess").accepted)
        second = attempt.finish(assessment_permits)
        self.assertTrue(second.succeeded)
        current = second.snapshot
        revisions = current["decision_series"]["assessment"]["revisions"]
        self.assertEqual([(entry["id"], entry["previous"]) for entry in revisions],
                         [("assessment@1", None), ("assessment@2", "assessment@1")])
        self.assertEqual(current["commitment_bases"]["assessment@2"]["value"], 90)
        self.assertEqual(set(current["commitment_grounds"]["assessment@2"]["evidence"]), {"observations@2"})
        self.assert_frozen(first.snapshot, current)
        self.run_readme_example("every-observation")

    def test_l4_opposition_reopens_the_shipped_approval_and_cannot_approve(self):
        server = self.server()
        first = assess_answer(server, 85)
        self.assertTrue(first.succeeded)
        self.assertTrue(server.dispatch("observe", {"confidence": 20}).accepted)
        current = server.snapshot()
        self.assertEqual(verdict(current), "reopened")
        self.assertEqual(current["reading_streams"]["observations"]["occurrences"][-1]["relation"], "opposes")
        self.assertEqual(current["decision_journal"][-1]["because"], ["observations@2"])
        self.assert_frozen(first.snapshot, current)
        self.assert_refused_unchanged(server, "assess", "The evidence does not support the answer.")
        self.assertEqual(len(server.snapshot()["decision_series"]["assessment"]["revisions"]), 1)

    def test_l5_old_approval_does_not_cover_a_rejected_required_operation(self):
        server = self.server()
        self.assertTrue(assess_answer(server, 85).succeeded)
        saved = server.save()
        failed = assess_answer(server, 250)
        self.assertFalse(failed.succeeded)
        self.assertFalse(failed.permits)
        self.assertEqual((failed.failed.origin, failed.failed.code), ("input", "bound_exceeded"))
        self.assertEqual([operation.status for operation in failed.operations], ["rejected", "not sent"])
        self.assertEqual(verdict(failed.snapshot), "approved")
        self.assertEqual(server.save(), saved)

    def test_l6_failed_attempt_does_not_poison_a_later_successful_attempt(self):
        server = self.server(self.every_observation)
        first = assess_answer(server, 85)
        self.assertTrue(first.succeeded)
        failed = assess_answer(server, 250)
        self.assertFalse(failed.succeeded)
        later = assess_answer(server, 90)
        self.assertTrue(later.succeeded)
        self.assertEqual([operation.status for operation in later.operations], ["accepted", "accepted"])
        self.assertEqual(later.snapshot["decision_series"]["assessment"]["current"], "assessment@2")
        self.assertEqual(set(later.snapshot["commitment_grounds"]["assessment@2"]["evidence"]), {"observations@2"})
        self.assert_frozen(first.snapshot, later.snapshot)
        self.assertFalse(failed.succeeded, "a later success changed the earlier attempt's result")
        self.run_readme_example("correction")

    def test_l7_unfinished_required_operation_cannot_borrow_an_approval(self):
        server = self.server()
        self.assertTrue(assess_answer(server, 85).succeeded)
        saved = server.save()
        attempt = Attempt(server)
        pending = attempt.begin("observe", {"confidence": 90})
        result = attempt.finish(assessment_permits)
        self.assertFalse(result.succeeded)
        self.assertFalse(result.permits)
        self.assertIsNone(result.snapshot)
        self.assertEqual([operation.status for operation in result.operations], ["not sent"])
        with self.assertRaises(AttemptFinished):
            pending.send()
        self.assertEqual(server.save(), saved)
        self.assertIs(attempt.finish(assessment_permits), result)

    def test_l8_restore_preserves_lifecycle_state_and_continued_execution(self):
        for program in (PROGRAM, self.every_observation):
            with self.subTest(policy=program.name):
                original = self.server(program)
                first = assess_answer(original, 85)
                self.assertTrue(first.succeeded)
                self.assertTrue(original.dispatch("observe", {"confidence": 20}).accepted)
                saved = original.save()
                resumed = self.server(program)
                self.assertEqual(resumed.restore(saved), original.snapshot()["sequence"])
                self.assertEqual(resumed.snapshot(), original.snapshot())
                self.assertEqual(resumed.save(), saved)
                for event, payload, accepted in (
                    ("assess", None, False),
                    ("observe", {"confidence": 92}, True),
                    ("assess", None, True),
                    ("retract", None, True),
                    ("observe", {"confidence": 94}, True),
                    ("assess", None, True),
                    ("assess", None, False),
                ):
                    before = original.save()
                    outcome = original.dispatch(event, payload)
                    restored_outcome = resumed.dispatch(event, payload)
                    self.assertEqual(outcome, restored_outcome)
                    self.assertEqual(outcome.accepted, accepted)
                    self.assertEqual(resumed.snapshot(), original.snapshot())
                    self.assertEqual(resumed.save(), original.save())
                    if not accepted:
                        self.assertEqual(original.save(), before)
                    self.assert_frozen(first.snapshot, original.snapshot())
                self.assertEqual(original.snapshot()["decision_series"]["assessment"]["current"], "assessment@3")
                self.assertEqual(set(original.snapshot()["commitment_grounds"]["assessment@3"]["evidence"]),
                                 {"observations@4"})

    def test_separate_tasks_use_separate_sessions(self):
        first_task = self.server()
        self.assertTrue(assess_answer(first_task, 85).succeeded)
        first_saved = first_task.save()
        other_task = self.server()
        initial = other_task.snapshot()
        self.assertEqual(verdict(initial), "none")
        self.assertEqual(initial["reading_streams"]["observations"]["occurrences"], [])
        self.assertEqual(initial["decision_series"]["assessment"]["revisions"], [])
        self.assert_refused_unchanged(other_task, "assess", "No observation to assess.")
        failed = assess_answer(other_task, 20)
        self.assertFalse(failed.succeeded)
        self.assertEqual(verdict(failed.snapshot), "none")
        self.assertEqual(other_task.snapshot()["reading_streams"]["observations"]["current"], "observations@1")
        self.assertEqual(first_task.save(), first_saved)


if __name__ == "__main__":
    unittest.main()
