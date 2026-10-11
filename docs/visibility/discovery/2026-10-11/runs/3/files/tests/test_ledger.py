import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent.parent / "src"))
import pytest
from factledger import Ledger


def make():
    L = Ledger()
    L.assert_fact("diet", "vegetarian", "user said")
    L.assert_fact("city", "Lisbon", "user said")
    r = L.recommend("dinner", "Try the veggie place", ["diet", "city"])
    return L, r


def test_recommendation_shows_facts_used():
    L, r = make()
    assert [f.id for f, _ in L.why(r.id)] == ["diet/1", "city/1"]
    assert L.needs_review() == []


def test_correction_flags_only_affected_recs_and_keeps_history():
    L, r = make()
    other = L.recommend("walk", "Park route", ["city"])
    L.assert_fact("diet", "vegan", "user corrected")
    assert [x.id for x in L.needs_review()] == [r.id]
    assert "diet/1" in L.facts and L.facts["diet/1"].value == "vegetarian"
    assert L.invalid["diet/1"].replaced_by == "diet/2"
    assert other not in L.needs_review()


def test_reconsider_keeps_original_and_links_it():
    L, r = make()
    L.assert_fact("diet", "vegan", "user corrected")
    r2 = L.reconsider(r.id, "Try the vegan place")
    assert r.id in L.recs and L.recs[r.id].text == "Try the veggie place"
    assert r2.revises == r.id and r2.used == ("diet/2", "city/1")
    assert L.needs_review() == []
    # the old rec still explains itself, including that its fact is now invalid
    assert L.why(r.id)[0][1].reason == "corrected"


def test_stale_without_replacement_cannot_silently_reconsider():
    L, r = make()
    L.mark_stale("city/1")
    assert L.needs_review()
    with pytest.raises(LookupError):
        L.reconsider(r.id)


def test_prov_export():
    L, r = make()
    L.assert_fact("diet", "vegan", "user corrected")
    L.reconsider(r.id, "vegan place")
    s = L.to_prov().get_provn()
    assert "wasInvalidatedBy" in s and "prov:Revision" in s and "used(" in s
