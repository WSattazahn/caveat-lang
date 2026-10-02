"""Synthetic controls for the amended post-study runner; no trial agents."""
import ast
import importlib.util
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
COPY = Path(__file__).with_name('run-trials.py')
FROZEN = Path(__file__).with_name('registration') / 'executed-runner.py'
EXPECTED = '38e703c7765bae0c7f98cdac21caf5a5bf5a015e6959d8e4e68adf34f6c509d6'
ast.parse(COPY.read_text(encoding='utf-8'))
spec = importlib.util.spec_from_file_location('runner_capture_copy', COPY)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
assert m.file_record(FROZEN)['sha256'] == EXPECTED
out = ROOT / 'test-results/adoption/runner-capture-checks' / m.now().replace(':', '-')
out.mkdir(parents=True)

class FailingOutput:
    def __init__(self, file, operation):
        self.stream = file.open('xb')
        self.operation = operation
    def write(self, data):
        if self.operation == 'write':
            raise OSError('synthetic output write failure')
        return self.stream.write(data)
    def flush(self):
        if self.operation == 'flush':
            raise OSError('synthetic output flush failure')
        self.stream.flush()
    def close(self):
        self.stream.close()
        if self.operation == 'close':
            raise OSError('synthetic output close failure')

controls = []
for operation in ['write', 'flush', 'close']:
    def factory(file):
        return FailingOutput(file, operation) if file.suffix == '.stdout' else file.open('xb')
    # For write/flush, keep the process alive so failure must trigger tree reaping.
    code = 'import sys,time; print("output",flush=True); time.sleep(10)' if operation != 'close' else 'print("output",flush=True)'
    receipt = m.bounded([sys.executable, '-c', code], ROOT, out, 'fail-' + operation, 5, 65536, output_factory=factory)
    assert receipt['stopReason'] == 'capture-io-error', receipt
    assert receipt['captureComplete'] is False and not m.command_passed(receipt)
    assert any(error['stage'] == 'stdout-file-' + operation for error in receipt['captureErrors']), receipt
    assert receipt['retainedOutputBytes'] == sum(receipt[name]['bytes'] for name in ['stdout', 'stderr'])
    if operation != 'close':
        assert receipt['termination']['pid'] == receipt['pid']
        assert receipt['elapsedSeconds'] < 5
    else:
        assert receipt['exitCode'] == 0, 'Capture failure must fail even a successful child'
    controls.append('capture-' + operation + '-failure')

stdin = b'x' * 16384
receipt = m.bounded([sys.executable, '-c', 'import sys; data=sys.stdin.buffer.read(); print(len(data),flush=True)'], ROOT, out, 'stdin-success', 5, 65536, stdin=stdin)
assert m.command_passed(receipt) and receipt['captureComplete'] is True and receipt['captureErrors'] == []
assert (out / 'stdin-success.stdout').read_bytes().strip() == b'16384'
controls.append('stdin-eof-and-complete-capture')
receipt = m.bounded([sys.executable, '-c', 'import sys; sys.stdout.write("x"*65536); sys.stdout.flush()'], ROOT, out, 'limit', 5, 1024)
assert receipt['stopReason'] == 'output-limit' and receipt['captureComplete'] is False
assert receipt['retainedOutputBytes'] == 1024 and not m.command_passed(receipt)
controls.append('combined-output-limit-still-enforced')
receipt = m.bounded([sys.executable, '-c', 'import time; time.sleep(10)'], ROOT, out, 'timeout', 0.25, 1024)
assert receipt['stopReason'] == 'timeout' and receipt['captureComplete'] is False
assert receipt['termination']['pid'] == receipt['pid'] and not m.command_passed(receipt)
controls.append('owned-timeout-still-enforced')
assert m.file_record(FROZEN)['sha256'] == EXPECTED
result = {'passed': True, 'controls': controls, 'actualAgentsStarted': 0, 'liveStudyFilesChanged': False,
          'frozenRunnerSha256': EXPECTED, 'amendedRunner': str(COPY), 'amendedRunnerSha256': m.file_record(COPY)['sha256']}
m.json_write(out / 'summary.json', result, exclusive=True)
print(json.dumps({'passed': True, 'controls': len(controls), 'receipt': str(out / 'summary.json')}))
