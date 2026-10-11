# Post-hoc, not registered: re-fuzzes every mutation survivor against its
# original as mutate.mjs does (100 sequences, seed 11), but with
# all-divergences.mjs --release-between under node --expose-gc, so a program
# whose sessions are never freed cannot trap both sides into the same error.
# Output lines: PROGRAM OPERATOR SITE DIVERGED.
cd "$(dirname "$0")/.."
ROOT=/home/claude/glowcap-r8
node -e 'for (const x of require("./results/mutation.json")) if (!x.caught) console.log(x.program, x.operator, x.site)' |
while read p o s; do
  orig=$ROOT/mut-run/$p/original/impl; mut=$ROOT/mut-run/$p/$o-$s/impl
  f=adapter.mjs; case $p in T*) f=glowcap.ts;; esac
  echo "$p $o $s"
done | xargs -P 4 -L 1 sh -c '
  ROOT=/home/claude/glowcap-r8; p=$0; o=$1; s=$2; f=adapter.mjs; case $p in T*) f=glowcap.ts;; esac
  out=$(cd $ROOT/mut-run/$p/$o-$s && node --expose-gc '"$PWD"'/fuzz-checks/all-divergences.mjs --impl=$ROOT/mut-run/$p/original/impl/$f --impl=$ROOT/mut-run/$p/$o-$s/impl/$f --sequences=100 --seed=11 --release-between --dump=/tmp/claude-0/surv-$p-$o-$s.json | grep -o ": [0-9]* diverged" | grep -o "[0-9]*")
  echo "$p $o $s $out"'
