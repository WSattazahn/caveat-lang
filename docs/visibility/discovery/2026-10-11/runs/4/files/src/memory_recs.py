"""Facts about a user, and recommendations that record which facts they used.

Design:
- Facts are append-only versions. A correction adds a version; the old one stays.
- A recommendation is immutable. It snapshots every (fact, version, value) it read.
- Whether a recommendation needs review is derived from the facts, not stored,
  so no correction can be missed.
- Reconsidering writes a new recommendation that points at the old one.
  The old one keeps its original inputs and reasoning.
"""
from __future__ import annotations

import itertools
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from typing import Any, Callable, Optional


def _now() -> datetime:
    return datetime.now(timezone.utc)


@dataclass(frozen=True)
class FactVersion:
    key: str
    version: int
    value: Any                      # None means retracted
    recorded_at: datetime
    expires_at: Optional[datetime] = None
    reason: str = "initial"         # why this version was written


@dataclass(frozen=True)
class FactUse:
    """Snapshot of a fact as it was when a recommendation read it."""
    key: str
    version: int
    value: Any


@dataclass(frozen=True)
class Recommendation:
    id: int
    text: str
    created_at: datetime
    used: tuple[FactUse, ...]
    supersedes: Optional[int] = None   # earlier recommendation this one revises
    note: str = ""                     # e.g. why it was reconsidered


@dataclass(frozen=True)
class Problem:
    """One reason a recommendation's input is no longer trustworthy."""
    key: str
    used_version: int
    kind: str                       # "corrected" | "retracted" | "stale"
    current_version: Optional[int]
    detail: str


class FactReader:
    """Handed to a recommender; records every fact it reads."""

    def __init__(self, store: "Store", at: datetime):
        self._store, self._at, self.used = store, at, {}

    def get(self, key: str, default: Any = None) -> Any:
        fv = self._store.current(key, at=self._at)
        if fv is None:
            return default
        self.used[key] = FactUse(fv.key, fv.version, fv.value)
        return fv.value


Recommender = Callable[[FactReader], str]


class Store:
    def __init__(self, clock: Callable[[], datetime] = _now):
        self._clock = clock
        self._facts: dict[str, list[FactVersion]] = {}
        self._recs: dict[int, Recommendation] = {}
        self._ids = itertools.count(1)

    # ---- facts ----
    def _append(self, key, value, reason, ttl: Optional[timedelta]) -> FactVersion:
        history = self._facts.setdefault(key, [])
        now = self._clock()
        fv = FactVersion(key, len(history) + 1, value, now,
                         now + ttl if ttl else None, reason)
        history.append(fv)
        return fv

    def learn(self, key: str, value: Any, ttl: Optional[timedelta] = None) -> FactVersion:
        return self._append(key, value, "initial" if key not in self._facts else "updated", ttl)

    def correct(self, key: str, value: Any, reason: str,
                ttl: Optional[timedelta] = None) -> FactVersion:
        if key not in self._facts:
            raise KeyError(key)
        return self._append(key, value, f"corrected: {reason}", ttl)

    def retract(self, key: str, reason: str) -> FactVersion:
        if key not in self._facts:
            raise KeyError(key)
        return self._append(key, None, f"retracted: {reason}", None)

    def history(self, key: str) -> list[FactVersion]:
        return list(self._facts.get(key, []))

    def current(self, key: str, at: Optional[datetime] = None) -> Optional[FactVersion]:
        """Latest version that is neither retracted nor expired."""
        h = self._facts.get(key)
        if not h:
            return None
        fv = h[-1]
        if fv.value is None or self._expired(fv, at):
            return None
        return fv

    def _expired(self, fv: FactVersion, at: Optional[datetime]) -> bool:
        return fv.expires_at is not None and (at or self._clock()) >= fv.expires_at

    # ---- recommendations ----
    def recommend(self, recommender: Recommender, *, supersedes: Optional[int] = None,
                  note: str = "") -> Recommendation:
        now = self._clock()
        reader = FactReader(self, now)
        text = recommender(reader)
        rec = Recommendation(next(self._ids), text, now,
                             tuple(reader.used.values()), supersedes, note)
        self._recs[rec.id] = rec
        return rec

    def get(self, rec_id: int) -> Recommendation:
        return self._recs[rec_id]

    def explain(self, rec_id: int) -> list[FactUse]:
        """The facts (as they were then) behind a recommendation."""
        return list(self._recs[rec_id].used)

    def problems(self, rec_id: int) -> list[Problem]:
        out = []
        for u in self._recs[rec_id].used:
            latest = self._facts[u.key][-1]
            if latest.version != u.version:
                kind = "retracted" if latest.value is None else "corrected"
                out.append(Problem(u.key, u.version, kind, latest.version, latest.reason))
            elif self._expired(latest, None):
                out.append(Problem(u.key, u.version, "stale", latest.version,
                                   f"expired {latest.expires_at.isoformat()}"))
        return out

    def superseded_by(self, rec_id: int) -> Optional[Recommendation]:
        later = [r for r in self._recs.values() if r.supersedes == rec_id]
        return later[-1] if later else None

    def status(self, rec_id: int) -> str:
        if self.superseded_by(rec_id):
            return "superseded"
        return "needs_review" if self.problems(rec_id) else "current"

    def needs_review(self) -> list[Recommendation]:
        return [r for r in self._recs.values() if self.status(r.id) == "needs_review"]

    def reconsider(self, rec_id: int, recommender: Recommender) -> Recommendation:
        """Re-run against current facts. The old recommendation is kept as-is."""
        old = self._recs[rec_id]
        if self.superseded_by(rec_id):
            raise ValueError(f"recommendation {rec_id} is already superseded")
        why = "; ".join(f"{p.key} v{p.used_version} {p.kind} ({p.detail})"
                        for p in self.problems(rec_id)) or "manual reconsideration"
        new = self.recommend(recommender, supersedes=old.id, note=why)
        return new

    def lineage(self, rec_id: int) -> list[Recommendation]:
        """Oldest-first chain of revisions ending at rec_id."""
        chain = [self._recs[rec_id]]
        while chain[0].supersedes is not None:
            chain.insert(0, self._recs[chain[0].supersedes])
        return chain
