"""Execute the two-process branching recipe shipped with the starter."""
from pathlib import Path
import unittest

HERE = Path(__file__).resolve().parent


class Branching(unittest.TestCase):
    def test_documented_two_process_branch(self):
        text = (HERE / "BRANCHING.md").read_text(encoding="utf-8")
        code = text.split("<!-- branching-example -->\n```python\n", 1)[1].split("\n```", 1)[0]
        filename = str(HERE / "branching_example.py")
        exec(compile(code, filename, "exec"), {"__file__": filename})
