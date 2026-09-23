# Root tool smoke test (excluded from author counts)

This source is authored by the supervisor, not a fresh author. It tests the
frozen WebAssembly runner with elapsed(), explicit invalid null versus an
omitted empty payload, save/restore, source capture and final freezing. It
implements none of the A/B task policies. Validation and replay pass; final
elapsed is 2, sequence is 3, the null payload is rejected, and resume preserves
the snapshot. No private candidate was scored before registration.
