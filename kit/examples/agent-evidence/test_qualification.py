"""Late qualification regression: exact identities, not a freshness slogan."""
from pathlib import Path
import unittest
from caller import CaveatServer

HERE = Path(__file__).resolve().parent


class Qualification(unittest.TestCase):
    def test_template_current_values_archives_and_future_samples(self):
        with CaveatServer(str(HERE / "qualification.cav")) as server:
            self.assertTrue(server.dispatch("remember", {"confidence": 85}).accepted)
            self.assertTrue(server.dispatch("assess").accepted)
            before = server.snapshot()
            archived = before["reading_streams"]["freshness"]["occurrences"][0]
            grounds = before["commitment_grounds"]["strategy@1"]
            self.assertEqual(set(grounds["evidence"]), {"freshness@1"})
            self.assertEqual(grounds["caveats"], [])
            self.assertTrue(server.dispatch("learn_stale").accepted)
            after = server.snapshot()
            self.assertIn("stale", after["value_grounds"]["direct"]["caveats"])
            self.assertTrue(any(relation["from"] == "stale" and relation["relation"] == "qualifies"
                                and relation["to"] == "memory" for relation in after["relations"]))
            self.assertIn("stale", after["qualified_values"]["direct"]["provenance"]["caveats"])
            self.assertEqual(set(after["value_grounds"]["direct"]["evidence"]), {"memory"})
            self.assertEqual(after["reading_streams"]["freshness"]["occurrences"][0], archived)
            self.assertEqual(after["commitment_grounds"]["strategy@1"], grounds)
            self.assertEqual(after["decision_journal"], before["decision_journal"])
            # Q1: reopening and making another decision does not take a sample.
            self.assertTrue(server.dispatch("reconsider").accepted)
            self.assertTrue(server.dispatch("assess").accepted)
            reused = server.snapshot()
            self.assertEqual(len(reused["reading_streams"]["freshness"]["occurrences"]), 1)
            self.assertEqual(reused["commitment_grounds"]["strategy@2"], grounds)
            # Save/restore retains these identity distinctions before continuing.
            saved = server.save()
            server.restore(saved)
            self.assertEqual(server.save(), saved)
            self.assertTrue(server.dispatch("remember", {"confidence": 90}).accepted)
            self.assertTrue(server.dispatch("reconsider").accepted)
            self.assertTrue(server.dispatch("assess").accepted)
            final = server.snapshot()
            self.assertEqual(final["reading_streams"]["freshness"]["occurrences"][0], archived)
            self.assertIn("stale", final["reading_streams"]["freshness"]["occurrences"][1]["provenance"]["caveats"])
            self.assertEqual(set(final["commitment_grounds"]["strategy@3"]["evidence"]), {"freshness@2"})
            self.assertIn("stale", final["commitment_grounds"]["strategy@3"]["caveats"])
            self.assertEqual(final["commitment_grounds"]["strategy@1"], grounds)
            self.assertEqual(final["commitment_grounds"]["strategy@2"], grounds)

    def test_sampling_does_not_observe_the_template(self):
        # Independent source removes the authored template observation, so the
        # test cannot confuse a sample with permission to qualify its template.
        import tempfile
        from caller import CaveatLoadError
        source = (HERE / "qualification.cav").read_text(encoding="utf-8")
        source = source.replace("on remember when not observed(memory) reveal memory supports memory_available;", "")
        source = source.replace("on remember set direct = qualified(confidence, memory);", "")
        with tempfile.TemporaryDirectory() as directory:
            program = Path(directory) / "sample_only.cav"
            program.write_text(source, encoding="utf-8")
            with self.assertRaisesRegex(CaveatLoadError, "qualify memory needs memory observed"):
                CaveatServer(str(program))
