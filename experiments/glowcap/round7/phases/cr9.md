# Phase cr9

Applied after every earlier phase; cumulative.

- **CR9 — watching others eat.** A new event, `witness` (`id`, `kind`): the
  slime sees another creature eat that mushroom and what it did to the
  creature. The mushroom is consumed. The evidence is `witness_<id>`, and it
  always carries the caveat `secondhand`. A witnessed glowcap supports the
  belief and a witnessed duskcap contradicts it, exactly like an absorption
  (it can commit, reopen and recover trust). The slime's own timers do not
  change. Rejected when the id or kind is unknown, the mushroom is consumed,
  or the kind contradicts a previous taste. Being too risky for the slime to
  absorb does not stop another creature eating it.
