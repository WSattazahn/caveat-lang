/-
A theorem a Caveat claim can rest on. The Caveat program in this folder
names it in lean-grounds.json; scripts/check-lean-grounds.mjs checks that it
exists, that its proof is complete and uses only the standard axioms, and that
its statement is the one the claim was made about.
-/
namespace LeanBridgeDemo

/-- Every natural number is smaller than two to its power. -/
theorem lt_two_pow (n : Nat) : n < 2 ^ n := by
  induction n with
  | zero => decide
  | succ k ih =>
    rw [Nat.pow_succ]
    have : 1 ≤ 2 ^ k := Nat.one_le_two_pow
    omega

end LeanBridgeDemo
