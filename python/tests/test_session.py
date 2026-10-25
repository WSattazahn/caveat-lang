"""The caveat_lang API, against the agent-evidence example program.

    python -m unittest discover -s python/tests -v
"""

from __future__ import annotations

import json
import threading
import types
import unittest
from pathlib import Path
from typing import List

import caveat_lang as caveat
from caveat_lang import _runtime

# A copy of kit/examples/agent-evidence/assessment.cav, so the tests run from
# the source distribution too. FixtureTests checks the copy in a clone.
FIXTURE = Path(__file__).resolve().parent / "fixtures" / "assessment.cav"
SOURCE = FIXTURE.read_bytes().decode("utf-8")
KIT_COPY = Path(__file__).resolve().parents[2] / "kit" / "examples" / "agent-evidence" / "assessment.cav"


class FixtureTests(unittest.TestCase):
    def test_the_fixture_is_the_kit_example(self) -> None:
        if not KIT_COPY.exists():
            self.skipTest("not in a clone of the repository")
        self.assertEqual(FIXTURE.read_bytes(), KIT_COPY.read_bytes())


class DispatchTests(unittest.TestCase):
    def setUp(self) -> None:
        self.session = caveat.open(SOURCE)
        self.addCleanup(self.session.close)

    def test_accepted_event_returns_the_snapshot(self) -> None:
        outcome = self.session.dispatch("observe", {"confidence": 85})
        assert isinstance(outcome, caveat.Accepted)
        self.assertTrue(outcome.accepted)
        self.assertEqual(outcome.snapshot["sequence"], 1)
        self.assertEqual(repr(outcome), "<Accepted 'observe', event 1>")

    def test_rejected_event_is_a_value_and_changes_nothing(self) -> None:
        outcome = self.session.dispatch("assess")
        self.assertEqual(outcome, caveat.Rejected("policy", "reject", "No observation to assess."))
        self.assertFalse(outcome.accepted)
        self.assertEqual(self.session.snapshot()["sequence"], 0)
        self.assertEqual(self.session.state, "open")

    def test_input_bound_is_refused_as_input(self) -> None:
        outcome = self.session.dispatch("observe", {"confidence": 101})
        assert isinstance(outcome, caveat.Rejected)
        self.assertEqual((outcome.origin, outcome.code), ("input", "bound_exceeded"))

    def test_unknown_event(self) -> None:
        outcome = self.session.dispatch("no_such_event")
        assert isinstance(outcome, caveat.Rejected)
        self.assertEqual(outcome.code, "unknown_event")

    def test_empty_event_name_is_a_value_error(self) -> None:
        with self.assertRaises(ValueError):
            self.session.dispatch("")

    def test_withdrawal_reopens_and_keeps_the_grounds(self) -> None:
        self.session.dispatch("observe", {"confidence": 85})
        self.session.dispatch("assess")
        self.session.dispatch("retract")
        snapshot = self.session.snapshot()
        self.assertEqual(snapshot["bindings"]["assessment"]["verdict"], "reopened")
        [revision] = caveat.decision_history(snapshot, "assessment")
        self.assertEqual(revision["status"], "reopened")
        self.assertEqual(revision["committed"]["because"], ["observations@1"])
        self.assertEqual(revision["reopened"]["because"], ["recheck"])
        self.assertEqual(caveat.decision_history(self.session.view(), "assessment"), [revision])
        html = self.session._repr_html_()
        self.assertIn("observations@1", html)
        self.assertIn("reopened", html)

    def test_outcomes_compare_but_do_not_hash(self) -> None:
        first = self.session.dispatch("assess")
        self.assertEqual(hash(first), hash(caveat.Rejected("policy", "reject", "No observation to assess.")))
        accepted = self.session.dispatch("observe", {"confidence": 85})
        with self.assertRaises(TypeError):
            hash(accepted)

    def test_another_thread_is_refused(self) -> None:
        caught: List[BaseException] = []

        def use() -> None:
            try:
                self.session.snapshot()
            except BaseException as error:  # noqa: BLE001 - the test reports whatever arrived
                caught.append(error)

        worker = threading.Thread(target=use)
        worker.start()
        worker.join()
        self.assertEqual([type(error) for error in caught], [caveat.ThreadError])
        self.assertEqual(self.session.state, "open")
        self.assertEqual(self.session.snapshot()["sequence"], 0)

    def test_decision_history_names_an_unknown_series(self) -> None:
        with self.assertRaises(KeyError):
            caveat.decision_history(self.session.snapshot(), "nothing")

    def test_dispatch_view_and_fields(self) -> None:
        outcome = self.session.dispatch_view("observe", {"confidence": 85})
        assert isinstance(outcome, caveat.AcceptedView)
        self.assertEqual(outcome.view, self.session.view())
        selected = self.session.dispatch_view("assess", fields=["bindings"])
        assert isinstance(selected, caveat.AcceptedView)
        self.assertEqual(set(selected.view), {"schema", "sequence", "last_event", "bindings"})
        with self.assertRaises(ValueError):
            self.session.dispatch_view("assess", fields=["bindings", "bindings"])
        with self.assertRaises(ValueError):
            self.session.dispatch_view("assess", fields=["nonsense"])  # type: ignore[list-item]
        self.assertEqual(self.session.snapshot()["sequence"], 2)

    def test_dispatch_view_delta(self) -> None:
        outcome = self.session.dispatch_view_delta("observe", {"confidence": 85})
        assert isinstance(outcome, caveat.AcceptedDelta)
        self.assertIsInstance(outcome.delta, dict)

    def test_views(self) -> None:
        self.assertEqual(self.session.view()["schema"], json.loads(self.session.view_text())["schema"])
        self.assertIn("schema", self.session.view(schema="0.2"))
        with self.assertRaises(ValueError):
            self.session.view(schema="9")  # type: ignore[arg-type]
        self.assertEqual(json.loads(self.session.snapshot_text()), self.session.snapshot())


class SaveTests(unittest.TestCase):
    def test_every_save_restores_to_the_same_snapshot(self) -> None:
        with caveat.open(SOURCE) as session:
            session.dispatch("observe", {"confidence": 85})
            session.dispatch("assess")
            saves = [session.save(), session.save(compact=True), session.save(compact=True, positional=True)]
            expected = session.snapshot_text()
        for save in saves:
            with caveat.restore(SOURCE, save) as restored:
                self.assertEqual(restored.snapshot_text(), expected)

    def test_positional_needs_compact(self) -> None:
        with caveat.open(SOURCE) as session, self.assertRaises(ValueError):
            session.save(positional=True)

    def test_a_save_restores_only_with_its_source(self) -> None:
        with caveat.open(SOURCE) as session:
            save = session.save()
        with self.assertRaises(caveat.RestoreError) as caught:
            caveat.restore(SOURCE + "\n# changed\n", save)
        self.assertEqual(caught.exception.kind, "restore")

    def test_archive(self) -> None:
        with caveat.open(SOURCE) as session:
            self.assertEqual(session.drain_archive(), [])
            self.assertEqual(session.undrained, 0)


class LifecycleTests(unittest.TestCase):
    def test_a_program_that_does_not_load(self) -> None:
        with self.assertRaises(caveat.LoadError) as caught:
            caveat.open("this is not caveat")
        self.assertEqual(caught.exception.kind, "load")
        with self.assertRaises(TypeError):
            caveat.open(b"bytes")  # type: ignore[arg-type]

    def test_closed_session_refuses_calls(self) -> None:
        session = caveat.open(SOURCE)
        session.close()
        session.close()
        self.assertEqual(session.state, "closed")
        with self.assertRaises(caveat.ClosedError):
            session.snapshot()

    def test_a_fatal_report_ends_the_session(self) -> None:
        session = caveat.open(SOURCE)

        class Failing:
            def dispatch_outcome(self, event: str, payload: str) -> str:
                report = {"schema": caveat.DISPATCH_SCHEMA, "outcome": "fatal", "code": "invariant",
                          "message": "broken"}
                raise _runtime.RuntimeFailure(json.dumps(report))

        session._native = Failing()  # type: ignore[assignment]  # noqa: SLF001
        with self.assertRaises(caveat.FatalError) as caught:
            session.dispatch("observe", {"confidence": 1})
        assert caught.exception.report is not None
        self.assertEqual(caught.exception.report["code"], "invariant")
        self.assertEqual(str(caught.exception), "invariant: broken")
        self.assertEqual(session.state, "fatal")
        with self.assertRaises(caveat.FatalError):
            session.snapshot()

    def test_an_unrecognised_outcome_is_fatal(self) -> None:
        session = caveat.open(SOURCE)

        class Odd:
            def dispatch_outcome(self, event: str, payload: str) -> str:
                return json.dumps({"schema": caveat.DISPATCH_SCHEMA, "outcome": "rejected", "origin": "host",
                                   "code": "x", "message": "y"})

        session._native = Odd()  # type: ignore[assignment]
        with self.assertRaises(caveat.FatalError):
            session.dispatch("observe", {"confidence": 1})
        self.assertEqual(session.state, "fatal")


class ProgramTests(unittest.TestCase):
    def test_check(self) -> None:
        report = caveat.check(SOURCE)
        self.assertIsInstance(report, dict)
        self.assertIsInstance(caveat.check(SOURCE, unused=True), dict)
        self.assertIsInstance(caveat.check(SOURCE, records=True), dict)
        with caveat.open(SOURCE) as session:
            save = session.save()
        self.assertIsInstance(caveat.check(SOURCE, records=True, save=save), dict)
        with self.assertRaises(ValueError):
            caveat.check(SOURCE, save=save)
        with self.assertRaises(caveat.LoadError):
            caveat.check("not caveat")

    def test_interface(self) -> None:
        names = json.dumps(caveat.interface(SOURCE))
        for event in ("observe", "assess", "retract", "erase"):
            self.assertIn(event, names)


class PayloadTests(unittest.TestCase):
    def test_numbers_are_written_as_javascript_writes_them(self) -> None:
        cases = {
            1.0: "1", -0.0: "0", 2.5: "2.5", 1e21: "1e+21", 1e20: "100000000000000000000",
            1e-7: "1e-7", 1e-6: "0.000001", 0.001: "0.001", 123.456: "123.456",
            5e-324: "5e-324", 1.7976931348623157e308: "1.7976931348623157e+308",
            2**53: "9007199254740992", -42: "-42", 1.5e-10: "1.5e-10",
        }
        for value, text in cases.items():
            self.assertEqual(caveat.payload_text(value), text, value)

    def test_payload_shapes(self) -> None:
        self.assertEqual(caveat.payload_text({"a": [1, True, None, "é\n"], "b": (2,)}),
                         '{"a":[1,true,null,"é\\n"],"b":[2]}')
        self.assertEqual(caveat.payload_text(types.MappingProxyType({"a": 1})), '{"a":1}')

    def test_integer_types_are_integers(self) -> None:
        class Int64:
            """Stands in for numpy.int64: an integer by __index__ alone."""

            def __index__(self) -> int:
                return 7

        self.assertEqual(caveat.payload_text({"n": Int64()}), '{"n":7}')

    def test_refused_payloads(self) -> None:
        for payload in (float("nan"), float("inf"), 2**53 + 1, {1: "key"}, {"a": object()}, {1, 2}, "\ud800"):
            with self.assertRaises(caveat.PayloadError):
                caveat.payload_text(payload)

    def test_a_refused_payload_leaves_the_session_open(self) -> None:
        with caveat.open(SOURCE) as session:
            with self.assertRaises(caveat.PayloadError):
                session.dispatch("observe", {"confidence": float("nan")})
            with self.assertRaises(caveat.PayloadError):
                session.dispatch("\ud800")
            self.assertEqual(session.state, "open")


if __name__ == "__main__":
    unittest.main()
