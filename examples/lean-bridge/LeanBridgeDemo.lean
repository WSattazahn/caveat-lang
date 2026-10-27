/-
Theorems Caveat claims can rest on. The Caveat program in this folder names
them in lean-grounds.json; scripts/check-lean-grounds.mjs checks that each
exists, that its proof is complete, that it uses only the standard axioms and
the results the manifest lists as assumed for now, and that its statement is
the one the claim was made about.
-/
namespace LeanBridgeDemo

/-- Every natural number is smaller than two to its power. Fully proven. -/
theorem lt_two_pow (n : Nat) : n < 2 ^ n := by
  induction n with
  | zero => decide
  | succ k ih =>
    rw [Nat.pow_succ]
    have : 1 ≤ 2 ^ k := Nat.one_le_two_pow
    omega

namespace Assumed

/-- Assumed for now, the way a large formalization assumes an established
result it has not yet proven. Replacing this axiom with a theorem of the same
statement clears the assumption. -/
axiom two_pow_add (a b : Nat) : 2 ^ (a + b) = 2 ^ a * 2 ^ b

end Assumed

/-- A bound that rests on `lt_two_pow` and on the assumed result. -/
theorem add_lt_two_pow_mul (a b : Nat) : a + b < 2 ^ a * 2 ^ b := by
  rw [← Assumed.two_pow_add]
  exact lt_two_pow (a + b)

end LeanBridgeDemo
