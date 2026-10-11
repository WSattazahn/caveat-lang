import test from "node:test";
import assert from "node:assert/strict";
import { ApprovalLedger } from "./approvals.js";

function setup() {
  const l = new ApprovalLedger();
  ["A", "B", "shared"].forEach((e) => l.addEvidence(e));
  l.approve("one", ["A", "shared"]);
  l.approve("two", ["B", "shared"]);
  return l;
}

test("withdrawing evidence only affects the approval that used it", () => {
  const l = setup();
  assert.deepEqual(l.withdraw("A"), ["one"]);
  assert.equal(l.isValid("one"), false);
  assert.equal(l.isValid("two"), true);
});

test("withdrawing shared evidence affects both", () => {
  const l = setup();
  assert.deepEqual(l.withdraw("shared").sort(), ["one", "two"]);
});

test("withdrawing unrelated evidence changes nothing", () => {
  const l = setup();
  l.addEvidence("other");
  assert.deepEqual(l.withdraw("other"), []);
  assert.deepEqual(l.needsReconsideration(), []);
});

test("an alternative justification keeps an approval valid", () => {
  const l = setup();
  l.approve("one", ["A"], ["B"]);
  assert.deepEqual(l.withdraw("A"), []);
  assert.deepEqual(l.withdraw("B"), ["one", "two"]);
});

test("restoring evidence revives the approval; withdraw reports only new losses", () => {
  const l = setup();
  l.withdraw("A");
  assert.deepEqual(l.withdraw("B"), ["two"]);
  l.restore("A");
  assert.equal(l.isValid("one"), true);
  assert.equal(l.isValid("two"), false);
});

test("approving on withdrawn evidence or without justification throws", () => {
  const l = setup();
  l.withdraw("A");
  assert.throws(() => l.approve("x", ["A"]));
  assert.throws(() => l.approve("y"));
});
