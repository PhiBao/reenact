#!/usr/bin/env bash
# Build the web UI, serve it with fixture cases, and have Kane CLI check the pages
# in a real browser. Prints a short pass/fail digest an agent can act on.
# Used by the Kiro hook in .kiro/hooks and by CI.
set -uo pipefail
cd "$(dirname "$0")/.."

PORT=8092
export REENACT_CASES_DIR="${REENACT_CASES_DIR:-web/fixtures/cases}"
export REENACT_BENCHMARK_FILE="${REENACT_BENCHMARK_FILE:-web/fixtures/benchmark.json}"

(cd web && pnpm build >/dev/null) || { echo "UI BUILD FAILED"; exit 1; }
cargo build -q -p reenact-server || { echo "SERVER BUILD FAILED"; exit 1; }

if curl -fs -o /dev/null "http://localhost:$PORT/healthz"; then echo "PORT $PORT IS BUSY: a stale server would be tested instead"; exit 1; fi
PORT=$PORT ./target/debug/reenact-server >/tmp/reenact-ui-server.log 2>&1 &
SERVER=$!
trap 'kill $SERVER 2>/dev/null' EXIT
for _ in $(seq 1 30); do curl -fs -o /dev/null "http://localhost:$PORT/healthz" && break; sleep 0.5; done

status=0
cd tests/ui
for t in *_test.md; do
  out=$(kane-cli testmd run "$t" --agent --headless --bug-detection stop 2>/dev/null)
  line=$(echo "$out" | grep '"type":"run_end"' | tail -1)
  verdict=$(echo "$line" | python3 -c 'import sys,json
l=sys.stdin.read().strip()
d=json.loads(l) if l else {}
print(d.get("status","no result"),"|",(d.get("reason") or d.get("summary") or "")[:300].replace("\n"," "))')
  echo "KANE $t: $verdict"
  case "$verdict" in passed*) ;; *) status=1 ;; esac
done
exit $status
