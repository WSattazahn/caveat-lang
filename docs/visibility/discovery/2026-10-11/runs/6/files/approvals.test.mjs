import test from "node:test";
import assert from "node:assert/strict";
import { ApprovalTracker } from "./approvals.mjs";

function setup() {
  const t = new ApprovalTracker();
  t.addEvidence("A");
  t.addEvidence("B");
  t.approve("one", ["A"]);
  t.approve("two", ["B"]);
  return t;
}

test("withdrawing evidence A reconsiders only approval one", () => {
  const t = setup();
  assert.deepEqual(t.withdraw("A"), ["one"]);
  assert.equal(t.status("one"), "reconsider");
  assert.equal(t.status("two"), "approved");
});

test("withdrawing evidence B reconsiders only approval two", () => {
  const t = setup();
  assert.deepEqual(t.withdraw("B"), ["two"]);
  assert.equal(t.status("one"), "approved");
});

test("shared evidence affects both approvals", () => {
  const t = new ApprovalTracker();
  t.addEvidence("A");
  t.approve("one", ["A"]);
  t.approve("two", ["A"]);
  assert.deepEqual(t.withdraw("A"), ["one", "two"]);
});

test("unrelated withdrawal changes nothing; cannot approve on withdrawn evidence", () => {
  const t = setup();
  assert.deepEqual(t.withdraw("Z"), []);
  t.withdraw("A");
  assert.throws(() => t.approve("three", ["A"]));
});
