"""Facts, recommendations, and the provenance linking them.

Everything is append-only. A correction adds a new fact version and marks the old one
invalidated; it never edits or deletes. A recommendation records the exact fact
versions it used, so a correction can be traced to the recommendations it affects.
The record can be exported as W3C PROV (via the `prov` package).
"""
from __future__ import annotations

import itertools
from dataclasses import dataclass, field
from datetime import datetime, timezone

from prov.model import ProvDocument

NS = "https://example.org/ledger/"


def _now() -> datetime:
    return datetime.now(timezone.utc)


@dataclass(frozen=True)
class FactVersion:
    id: str            # e.g. "diet/2"
    key: str           # e.g. "diet"
    value: object
    source: str        # where it came from: "user said", "inferred", ...
    at: datetime


@dataclass
class Invalidation:
    fact_id: str
    reason: str        # "stale" | "corrected" | free text
    at: datetime
    replaced_by: str | None = None


@dataclass(frozen=True)
class Recommendation:
    id: str            # e.g. "rec-restaurant/2"
    topic: str
    text: str
    used: tuple[str, ...]          # fact version ids
    at: datetime
    revises: str | None = None     # earlier recommendation version
    revision_note: str | None = None


class Ledger:
    def __init__(self) -> None:
        self.facts: dict[str, FactVersion] = {}
        self.invalid: dict[str, Invalidation] = {}
        self.recs: dict[str, Recommendation] = {}
        self._n = itertools.count(1)

    # ---- facts -------------------------------------------------------
    def current_fact(self, key: str) -> FactVersion | None:
        live = [f for f in self.facts.values() if f.key == key and f.id not in self.invalid]
        return max(live, key=lambda f: f.at) if live else None

    def _versions(self, key: str) -> int:
        return sum(f.key == key for f in self.facts.values())

    def assert_fact(self, key: str, value: object, source: str) -> FactVersion:
        """Record a fact. If one is already live for `key`, this is a correction."""
        prev = self.current_fact(key)
        fv = FactVersion(f"{key}/{self._versions(key) + 1}", key, value, source, _now())
        self.facts[fv.id] = fv
        if prev:
            self.invalid[prev.id] = Invalidation(prev.id, "corrected", fv.at, fv.id)
        return fv

    def mark_stale(self, fact_id: str, reason: str = "stale") -> None:
        """Retire a fact with no replacement (e.g. it has aged out)."""
        self.invalid.setdefault(fact_id, Invalidation(fact_id, reason, _now()))

    # ---- recommendations --------------------------------------------
    def recommend(self, topic: str, text: str, used_keys: list[str]) -> Recommendation:
        """Make a recommendation from the *current* versions of `used_keys`."""
        used = []
        for k in used_keys:
            f = self.current_fact(k)
            if f is None:
                raise LookupError(f"no live fact for {k!r}")
            used.append(f.id)
        return self._add_rec(topic, text, tuple(used), None, None)

    def _add_rec(self, topic, text, used, revises, note) -> Recommendation:
        n = sum(r.topic == topic for r in self.recs.values()) + 1
        r = Recommendation(f"rec-{topic}/{n}", topic, text, used, _now(), revises, note)
        self.recs[r.id] = r
        return r

    def why(self, rec_id: str) -> list[tuple[FactVersion, Invalidation | None]]:
        """The facts a recommendation used, and whether each has since been invalidated."""
        return [(self.facts[i], self.invalid.get(i)) for i in self.recs[rec_id].used]

    def head(self, topic: str) -> Recommendation | None:
        """Latest recommendation version for a topic."""
        rs = [r for r in self.recs.values() if r.topic == topic]
        return max(rs, key=lambda r: r.at) if rs else None

    def needs_review(self) -> list[Recommendation]:
        """Head recommendations that rest on at least one invalidated fact."""
        heads = {r.topic: self.head(r.topic) for r in self.recs.values()}
        return [r for r in heads.values() if any(i in self.invalid for i in r.used)]

    def reconsider(self, rec_id: str, new_text: str | None = None) -> Recommendation:
        """Re-issue a recommendation against current facts. The old version stays.

        Pass new_text if the answer changes; if omitted the text is carried over
        (the facts that changed turned out not to matter) but the new version is
        still recorded, so the review is visible.
        """
        old = self.recs[rec_id]
        used = []
        for fid in old.used:
            inv = self.invalid.get(fid)
            if inv is None:
                used.append(fid)
            elif inv.replaced_by:
                used.append(inv.replaced_by)
            else:
                raise LookupError(f"{fid} retired with no replacement; supply a new fact")
        changed = [f"{o}->{n}" for o, n in zip(old.used, used) if o != n]
        note = "facts changed: " + ", ".join(changed)
        return self._add_rec(old.topic, new_text or old.text, tuple(used), old.id, note)

    # ---- export ------------------------------------------------------
    def to_prov(self) -> ProvDocument:
        d = ProvDocument()
        d.add_namespace("l", NS)
        q = lambda s: f"l:{s}"
        for f in self.facts.values():
            d.entity(q(f.id), {"l:key": f.key, "l:value": str(f.value), "l:source": f.source})
        for inv in self.invalid.values():
            act = d.activity(q(f"invalidate:{inv.fact_id}"), inv.at, inv.at,
                             {"l:reason": inv.reason})
            d.wasInvalidatedBy(q(inv.fact_id), act, inv.at)
            if inv.replaced_by:
                d.wasRevisionOf(q(inv.replaced_by), q(inv.fact_id))
        for r in self.recs.values():
            d.entity(q(r.id), {"l:topic": r.topic, "l:text": r.text})
            act = d.activity(q(f"make:{r.id}"), r.at, r.at)
            d.wasGeneratedBy(q(r.id), act)
            for fid in r.used:
                d.used(act, q(fid))
                d.wasDerivedFrom(q(r.id), q(fid))
            if r.revises:
                d.wasRevisionOf(q(r.id), q(r.revises))
        return d
