import sys, pathlib
from datetime import datetime, timedelta, timezone
sys.path.insert(0, str(pathlib.Path(__file__).parent.parent / "src"))
from memory_recs import Store


class Clock:
    def __init__(self): self.t = datetime(2026, 1, 1, tzinfo=timezone.utc)
    def __call__(self): return self.t


def dinner(r):
    diet = r.get("diet")
    r.get("city")  # read but irrelevant to text; still recorded
    return f"Try the {'tofu' if diet == 'vegetarian' else 'steak'} place"


def test_correction_flags_and_preserves_history():
    s = Store()
    s.learn("diet", "vegetarian"); s.learn("city", "Oslo"); s.learn("pet", "cat")
    rec = s.recommend(dinner)
    assert {u.key for u in s.explain(rec.id)} == {"diet", "city"}  # pet not used
    assert s.status(rec.id) == "current"

    s.correct("diet", "omnivore", "user said they eat meat")
    assert s.status(rec.id) == "needs_review"
    assert s.problems(rec.id)[0].kind == "corrected"

    new = s.reconsider(rec.id, dinner)
    assert "steak" in new.text and "diet v1 corrected" in new.note
    assert s.status(rec.id) == "superseded" and s.status(new.id) == "current"
    # original untouched: still shows the old value it used
    assert "tofu" in s.get(rec.id).text
    assert next(u for u in s.explain(rec.id) if u.key == "diet").value == "vegetarian"
    assert [r.id for r in s.lineage(new.id)] == [rec.id, new.id]


def test_unrelated_correction_does_not_flag():
    s = Store(); s.learn("diet", "vegetarian"); s.learn("city", "Oslo"); s.learn("pet", "cat")
    rec = s.recommend(dinner)
    s.correct("pet", "dog", "oops")
    assert s.status(rec.id) == "current"


def test_stale_and_retracted():
    c = Clock(); s = Store(c)
    s.learn("city", "Oslo", ttl=timedelta(days=30)); s.learn("diet", "vegetarian")
    rec = s.recommend(dinner)
    c.t += timedelta(days=31)
    assert s.problems(rec.id)[0].kind == "stale"
    s.learn("city", "Bergen")  # refreshed
    s.retract("diet", "no longer true")
    kinds = {p.key: p.kind for p in s.problems(rec.id)}
    assert kinds == {"city": "corrected", "diet": "retracted"}
    new = s.reconsider(rec.id, dinner)   # retracted fact reads as missing
    assert [u.key for u in new.used] == ["city"]
    assert s.needs_review() == []
