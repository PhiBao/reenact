import sys, json

total = status = reason = None
decisions = ""
for line in sys.stdin:
    try:
        d = json.loads(line)
    except ValueError:
        continue
    if d.get("type") == "run_end":
        status = d.get("status")
        reason = (d.get("reason") or "")[:90]
        for k, x in ((d.get("context") or {}).get("variables") or {}).items():
            if "total" in k.lower():
                total = x.get("value")
    if d.get("type") == "test_md_summary":
        s = d.get("steps", {})
        decisions = "replay_decisions=%s author_decisions=%s" % (s.get("replay_decisions"), s.get("author_decisions"))
print("status=%s | reason=%s | total variable seen by the run=%s | %s" % (status, reason, total, decisions))
