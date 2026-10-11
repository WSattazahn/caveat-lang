# Post-hoc sensitivity check, not registered: rewrites every Caveat M5 mutant
# to cite the bare evidence name, as round 7's mutator did, instead of
# observed(...), and runs `caveat check` on it. Output: results/m5-bare-name.txt
M=/home/claude/glowcap-r8/mutator/mutants
for d in $M/C*/M5-*; do prog=$(basename $(dirname $d)); n=$(basename $d)
  w=/home/claude/glowcap-r8/authors/$prog/m5bare; rm -rf $w; mkdir -p $w; cp $d/glowcap.cav $w/
  sed -i -E 's/observed\((absorb_cave|a_cave)\)/\1/' $w/glowcap.cav
  if diff -q $d/glowcap.cav $w/glowcap.cav >/dev/null; then echo "$prog $n nochange"; continue; fi
  out=$(cd /home/claude/glowcap-r8/authors/$prog && node node_modules/caveat-lang/bin/caveat.mjs check m5bare/glowcap.cav 2>&1); st=$?
  echo "$prog $n status=$st $(echo "$out" | grep -o 'ungrounded[a-z_]*\|does not load[^:]*' | head -1)"
done
