#!/usr/bin/env bash
# Does Kane's cached replay notice when the app regresses?
#   1. author a test against a FIXED Corner Shop (live agent run; passes; recording is saved)
#   2. replay the saved test (no --author) against the BUGGY Corner Shop, twice
#   3. run the same test live (--author) against the BUGGY shop
# If step 2 passes, replay did not re-check the page.
set -uo pipefail
cd "$(dirname "$0")"
HERE=$PWD
ROOT=$(cd ../.. && pwd)
PORT=4101
W=$(mktemp -d)
cp -r "$ROOT/demo/shop" "$W/buggy"; cp -r "$ROOT/demo/shop" "$W/fixed"
# the fix: always take $10 off
python3 - "$W/fixed/public/app.js" <<'PY'
import sys,re
p=sys.argv[1]; s=open(p).read()
s=s.replace("state.discount = items > 1 ? -10 : 10;","state.discount = 10;")
open(p,'w').write(s)
PY
cp coupon_test.md "$W/coupon_test.md"
start(){ (cd "$W/$1" && PORT=$PORT nohup node server.js >/dev/null 2>&1 & echo $! > "$W/pid"); sleep 1; }
stop(){ fuser -k $PORT/tcp >/dev/null 2>&1; sleep 1; }
run(){ ( cd "$W" && kane-cli testmd run coupon_test.md --agent --headless --bug-detection stop "$@" 2>/dev/null ) | python3 "$HERE/parse.py"; }

echo "== 1. author on FIXED app";   start fixed; run --author; stop
echo "== 2a. replay on BUGGY app";  start buggy; run; 
echo "== 2b. replay on BUGGY app (again)"; run
echo "== 3. live agent on BUGGY app (--author)"; run --author; stop
echo "page total on the buggy app after a coupon with two items: \$30.00 (see demo/shop/public/app.js)"
