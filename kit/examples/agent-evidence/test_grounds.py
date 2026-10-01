"""Custom application tests, separate from the 53 original starter tests.

The negative source variants deliberately commit plausible but wrong grounds.
Operation success alone must not turn them into a successful application run.
"""
from pathlib import Path
import tempfile
import unittest
from caller import Attempt, CaveatServer, assessment_permits
from grounds import grounded_permits

HERE = Path(__file__).resolve().parent
SOURCE = (HERE / "grounded_assessment.cav").read_text(encoding="utf-8")
EVENTS = {"clarity": "observe_clarity", "freshness": "observe_memory", "tools": "observe_tool"}


class Grounds(unittest.TestCase):
    def open(self, source=SOURCE):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        program = Path(directory.name) / "assessment.cav"
        program.write_text(source, encoding="utf-8")
        server = CaveatServer(str(program))
        self.addCleanup(server.close)
        return server

    def readings(self, server, attempt, streams=EVENTS):
        expected = {}
        for stream in streams:
            result = attempt.require(EVENTS[stream], {"confidence": 85})
            self.assertTrue(result.accepted)
            expected[stream] = server.snapshot()["reading_streams"][stream]["current"]
        return expected

    def finish(self, server, attempt, expected, **policy):
        attempt.require("assess")
        return attempt.finish(lambda snapshot, active: grounded_permits(snapshot, active, expected, **policy))

    def test_exact_current_attempt_grounds_permit(self):
        server = self.open()
        attempt = Attempt(server)
        result = self.finish(server, attempt, self.readings(server, attempt))
        self.assertTrue(result.succeeded)

    def test_missing_each_required_source_is_rejected(self):
        for missing in EVENTS:
            with self.subTest(missing=missing):
                server = self.open()
                attempt = Attempt(server)
                expected = self.readings(server, attempt, [stream for stream in EVENTS if stream != missing])
                result = self.finish(server, attempt, expected)
                self.assertFalse(result.succeeded)
                self.assertEqual((result.failed.origin, result.failed.code), ("policy", "reject"))
                self.assertEqual(result.snapshot["decision_series"]["assessment"]["revisions"], [])

    def test_old_readings_cannot_silently_satisfy_a_new_attempt(self):
        server = self.open()
        first = Attempt(server)
        expected = self.readings(server, first)
        self.assertTrue(self.finish(server, first, expected).succeeded)
        later = Attempt(server)
        expected.update(self.readings(server, later, ["tools"]))
        result = self.finish(server, later, expected)
        self.assertTrue(assessment_permits(result.snapshot, later))
        self.assertFalse(result.succeeded)

    def test_only_explicit_memory_carry_over_is_allowed(self):
        server = self.open()
        first = Attempt(server)
        expected = self.readings(server, first)
        self.assertTrue(self.finish(server, first, expected).succeeded)
        later = Attempt(server)
        expected.update(self.readings(server, later, ["clarity", "tools"]))
        later.require("assess")
        snapshot = server.snapshot()
        self.assertFalse(grounded_permits(snapshot, later, expected))
        self.assertFalse(grounded_permits(snapshot, later, expected, carry_memory="freshness@999"))
        result = later.finish(lambda snap, active: grounded_permits(snap, active, expected, carry_memory=expected["freshness"]))
        self.assertTrue(result.succeeded)

    def test_wrong_reading_is_rejected_even_with_a_new_commitment(self):
        bad = SOURCE.replace("using (latest(clarity)", "using (history_at(clarity, 0)")
        server = self.open(bad)
        attempt = Attempt(server)
        self.readings(server, attempt, ["clarity"])
        expected = self.readings(server, attempt)
        result = self.finish(server, attempt, expected)
        self.assertTrue(assessment_permits(result.snapshot, attempt))
        self.assertIn("clarity@1", result.snapshot["commitment_grounds"]["assessment@1"]["evidence"])
        self.assertFalse(result.succeeded)

    def test_wrong_sources_are_rejected_even_with_successful_operations(self):
        bad = SOURCE.replace("(latest(clarity) + latest(freshness) + latest(tools)) / 3", "(latest(clarity) + latest(freshness)) / 2")
        server = self.open(bad)
        attempt = Attempt(server)
        result = self.finish(server, attempt, self.readings(server, attempt))
        self.assertTrue(assessment_permits(result.snapshot, attempt))
        self.assertTrue(all(op.status == "accepted" for op in result.operations))
        self.assertFalse(result.succeeded)

    def test_withdrawn_clarity_requires_replacement_Q2(self):
        server = self.open()
        attempt = Attempt(server)
        expected = self.readings(server, attempt)
        first = self.finish(server, attempt, expected)
        self.assertTrue(first.succeeded)
        original = first.snapshot["commitment_grounds"]["assessment@1"]
        self.assertTrue(server.dispatch("retract_clarity").accepted)
        saved = server.save()
        later = Attempt(server)
        refused = self.finish(server, later, expected)
        self.assertFalse(refused.succeeded)
        self.assertEqual(refused.failed.message, "Clarity was withdrawn.")
        self.assertEqual(server.save(), saved)
        self.assertEqual(refused.snapshot["commitment_grounds"]["assessment@1"], original)
        self.assertEqual(len(refused.snapshot["decision_series"]["assessment"]["revisions"]), 1)
        retry = Attempt(server)
        self.assertTrue(self.finish(server, retry, self.readings(server, retry)).succeeded)

    def test_extra_evidence_is_not_an_exact_match(self):
        bad = SOURCE.replace('claim misreport;', 'claim misreport; evidence extra from "an unrelated source";')
        bad = bad.replace('on assess commit assessment',
                          'on assess reveal extra supports answer_supported; on assess commit assessment')
        bad = bad.replace('latest(tools)) / 3;', 'latest(tools)) / 3 + qualified(0, extra);')
        server = self.open(bad)
        attempt = Attempt(server)
        result = self.finish(server, attempt, self.readings(server, attempt))
        self.assertTrue(assessment_permits(result.snapshot, attempt))
        self.assertTrue(all(op.status == "accepted" for op in result.operations))
        self.assertIn("extra", result.snapshot["commitment_grounds"]["assessment@1"]["evidence"])
        self.assertFalse(result.succeeded)

    def test_documented_grounding_example(self):
        text = (HERE / "QUALIFICATION.md").read_text(encoding="utf-8")
        code = text.split("<!-- grounding-example -->\n```python\n", 1)[1].split("\n```", 1)[0]
        filename = str(HERE / "grounding_example.py")
        exec(compile(code, filename, "exec"), {"__file__": filename})
