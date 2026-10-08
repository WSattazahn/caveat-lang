"""Read-only integrity and bounded arithmetic check of retained observations."""
from pathlib import Path
from fractions import Fraction
import hashlib
import json
import statistics
import zipfile

root = Path(__file__).resolve().parent
manifest = json.loads((root / 'manifest.json').read_text(encoding='utf-8'))
def verify(data, row):
    assert len(data) == row['bytes'], row['path']
    assert hashlib.sha256(data).hexdigest() == row['sha256'], row['path']
for row in manifest['files']:
    verify((root / row['path']).read_bytes(), row)
count = 0
for row in manifest['archives']:
    file = root / row['path']
    verify(file.read_bytes(), row)
    with zipfile.ZipFile(file) as z:
        assert z.testzip() is None
        assert len(z.namelist()) == len(set(z.namelist())) == len(row['entries'])
        assert set(z.namelist()) == {item['path'] for item in row['entries']}
        for item in row['entries']:
            verify(z.read(item['path']), item)
            count += 1
with zipfile.ZipFile(root / 'high-degree-confirmation/raw-observations.zip') as z:
    phases = {}
    for phase in ['growth', 'release', 'failed_release']:
        ratios, differences = [], []
        for trial in range(1, 9):
            records = []
            for side in ['before', 'after']:
                prefix = f'run-001/capture/high-degree-1000-t{trial}-{side}'
                receipt = json.loads(z.read(prefix + '.json'))
                assert receipt['status'] == 0 and receipt['error'] is None
                raw = z.read(prefix + '.stdout')
                assert hashlib.sha256(raw).hexdigest() == receipt['stdout']['sha256']
                records.append(json.loads(raw))
            before, after = [record[phase]['median_ns'] for record in records]
            ratios.append(Fraction(after, before) - 1)
            differences.append(after - before)
            assert records[0]['archive']['ndjson_sha256'] == records[1]['archive']['ndjson_sha256']
            for point in ['before_release', 'after_release']:
                assert records[0][point] == records[1][point]
        median = statistics.median(ratios)
        phases[phase] = {'median_paired_percent': float(median * 100), 'median_paired_delta_ns': statistics.median(differences), 'trigger': median > Fraction(5, 100)}
    assert [phases[p]['trigger'] for p in phases] == [False, True, False]
print(json.dumps({'accepted': True, 'copied_reports': len(manifest['files']), 'raw_entries': count, 'high_degree': phases}, indent=2))
