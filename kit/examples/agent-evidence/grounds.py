"""Application-specific permit check for grounded_assessment.cav.

One session represents one task. The caller supplies exact intended reading IDs.
Clarity and tools must be from this attempt. Memory may carry over only when
its exact ID is explicitly allowed; this is example policy, not a runtime rule.
"""
from caller import assessment_permits

STREAMS = ("clarity", "freshness", "tools")


def grounded_permits(snapshot, attempt, expected, *, carry_memory=None):
    if set(expected) != set(STREAMS) or not assessment_permits(snapshot, attempt):
        return False
    series = snapshot["decision_series"]["assessment"]
    current = series["current"]
    grounds = snapshot["commitment_grounds"][current]
    if set(grounds["evidence"]) != set(expected.values()):
        return False
    withdrawn = {entry["evidence"] for entry in snapshot.get("withdrawals", [])}
    for stream, identity in expected.items():
        history = snapshot["reading_streams"][stream]
        reading = next((item for item in history["occurrences"] if item["id"] == identity), None)
        if reading is None or history["current"] != identity or identity in withdrawn:
            return False
        if reading["sequence"] <= attempt.start_sequence:
            if stream != "freshness" or identity != carry_memory:
                return False
    return True
