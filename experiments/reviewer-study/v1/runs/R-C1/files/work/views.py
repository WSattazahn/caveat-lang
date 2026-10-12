import json,subprocess,sys
for h in ["H1","H2","H3"]:
    out=subprocess.run(["npx","--no-install","caveat-lang","replay","gate.cav",f"histories/{h}.jsonl"],capture_output=True,text=True).stdout
    print("==",h)
    for line in out.splitlines():
        r=json.loads(line); s=r.get("snapshot") or {}
        keys=[k for k in s if 'display' in k or 'bind' in k]
        disp=s.get("displayed") or s.get("display") or {}
        rest={k:v for k,v in r.items() if k!="snapshot"}
        cg=s.get("commitment_grounds")
        print(rest.get("record"),rest.get("line",rest.get("index")),json.dumps({k:v for k,v in rest.items() if k not in("record",)})[:200], keys)
        print("   values:",s.get("values"))
        print("   commitment_grounds:",cg, " taken_back:",s.get("taken_back"))
