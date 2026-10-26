"""The Python package returns the same bytes as the npm kit.

Replays every scenario file in the repository through both runtimes
(differential/trace.mjs on the kit's WebAssembly build, differential/trace.py
on this package's native build) and requires identical traces: every
dispatch outcome, snapshot, view, save, restore, check and interface text.

Needs Node and the WebAssembly build (npm run build, at dist/pkg-reactive).
Without them the test is skipped, unless CAVEAT_DIFFERENTIAL=1, which makes a
missing piece a failure.
"""

from __future__ import annotations

import json
import os
import random
import shutil
import struct
import subprocess
import sys
import unittest
from pathlib import Path
from typing import List

import caveat_lang as caveat

ROOT = Path(__file__).resolve().parents[2]
HERE = Path(__file__).resolve().parent / "differential"
RUNTIME = Path(os.environ.get("CAVEAT_WASM_DIR", ROOT / "dist" / "pkg-reactive"))
REQUIRED = os.environ.get("CAVEAT_DIFFERENTIAL") == "1"


def scenario_files() -> List[str]:
    """Every tracked scenario file whose programs are in the repository."""
    listed = subprocess.run(["git", "ls-files", "*.scenarios.json"], cwd=ROOT, capture_output=True, text=True,
                            check=True).stdout.split()
    files = []
    for name in listed:
        doc = json.loads((ROOT / name).read_bytes())
        sources = {doc["source"], *(item["source"] for item in doc["scenarios"] if "source" in item)}
        if all((ROOT / name).parent.joinpath(source).exists() for source in sources):
            files.append(name)
    return files


def node_missing() -> str:
    if shutil.which("node") is None:
        return "node is not installed"
    if not (RUNTIME / "caveat_runtime_bg.wasm").exists():
        return f"no WebAssembly build at {RUNTIME}; run npm run build"
    return ""


class DifferentialTests(unittest.TestCase):
    def setUp(self) -> None:
        missing = node_missing()
        if missing:
            if REQUIRED:
                self.fail(missing)
            self.skipTest(missing)

    def test_corpus_traces_are_identical(self) -> None:
        files = scenario_files()
        self.assertGreater(len(files), 60)
        script = subprocess.run(["node", str(HERE / "trace.mjs"), str(RUNTIME), *files], cwd=ROOT,
                                capture_output=True, check=True).stdout
        native = subprocess.run([sys.executable, str(HERE / "trace.py"), *files], cwd=ROOT,
                                capture_output=True, check=True).stdout
        if script != native:
            at = next(index for index, (a, b) in enumerate(zip(script, native)) if a != b) if script[:1] else 0
            self.fail(f"traces differ at byte {at}: kit {script[max(0, at - 120):at + 120]!r} "
                      f"python {native[max(0, at - 120):at + 120]!r}")

    def test_payload_numbers_match_javascript(self) -> None:
        generator = random.Random(20261025)
        values: List[float] = [0.1, 1 / 3, 1e21, 1e-7, 2.0**53, 2.0**60, 123456789.125]
        while len(values) < 5000:
            value = struct.unpack("<d", generator.getrandbits(64).to_bytes(8, "little"))[0]
            if value == value and abs(value) != float("inf"):
                values.append(value)
        values += [generator.uniform(-1e6, 1e6) for _ in range(2000)]
        values += [float(generator.randint(-(2**53), 2**53)) for _ in range(1000)]
        script = "let s='';process.stdin.on('data',d=>s+=d).on('end',()=>process.stdout.write(" \
                 "JSON.stringify(JSON.parse(s).map(v=>JSON.stringify(v)))))"
        texts = json.loads(subprocess.run(["node", "-e", script], input=json.dumps(values).encode(),
                                          capture_output=True, check=True).stdout)
        for value, text in zip(values, texts):
            self.assertEqual(caveat.payload_text(value), text, repr(value))


if __name__ == "__main__":
    unittest.main()
