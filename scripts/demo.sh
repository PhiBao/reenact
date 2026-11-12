#!/usr/bin/env bash
# One command: start the Corner Shop demo app, have Reenact reproduce a bug report
# against it with Kane in a real browser, and print the verdict.
#
#   ./scripts/demo.sh [coupon|negative-quantity|remove-wrong-item]
#
# Needs: kane-cli (logged in), Node 18+, Chrome, and REENACT_LLM_API_KEY
# (any OpenAI-compatible endpoint; see .env.example).
set -euo pipefail
cd "$(dirname "$0")/.."
ISSUE="${1:-coupon}"
[ -f .env ] && set -a && . ./.env && set +a
: "${REENACT_LLM_API_KEY:?set REENACT_LLM_API_KEY (see .env.example)}"

cargo build -q --release -p reenact-cli
(cd demo/shop && PORT=4100 node server.js >/tmp/reenact-demo-shop.log 2>&1 &) 
trap 'fuser -k 4100/tcp >/dev/null 2>&1 || true' EXIT
for _ in $(seq 1 20); do curl -fs -o /dev/null http://localhost:4100/healthz && break; sleep 0.5; done

TITLE=$(head -c 0 /dev/null; case "$ISSUE" in
  coupon) echo "Coupon makes my total go UP";;
  negative-quantity) echo "Quantity can go negative";;
  remove-wrong-item) echo "Remove button deletes the wrong item";;
  *) echo "$ISSUE";; esac)
./target/release/reenact repro --config demo/shop/.reenact.yml \
  --title "$TITLE" --body-file "demo/issues/$ISSUE.md" --out .reenact/cases
echo
echo "Open the case folder above, or run the web UI: REENACT_CASES_DIR=.reenact/cases PORT=8092 ./target/release/reenact-server"
