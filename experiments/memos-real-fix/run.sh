#!/usr/bin/env bash
# Reenact fixes a REAL bug in a REAL open-source app, and checks itself against the maintainers' fix.
#
#   ./run.sh <case>            case: search-filter | memo-detail-stale
#
# What it does
#   1. Checks out Memos at the release the bug was filed on (v0.26.2) and serves its web UI from source
#      (vite dev server) in front of the matching official backend image.
#   2. Reproduces the upstream issue as written (`reenact repro`).
#   3. `reenact fix` with Kiro CLI as the fixing agent: confirm the bug twice, let the agent edit
#      web/src only, re-run the SAME Kane test, keep the change only if it passes.
#   4. Reference check: apply the maintainers' actual fix commit to a clean checkout and run the
#      same Kane test against it. A good test must accept the real fix too.
#
# Needs: Docker, Node 22 + pnpm, Chrome, kane-cli (logged in), kiro-cli (logged in), gh, an LLM key
# for the compiler (REENACT_LLM_*), and the release binary built with `cargo build --release`.
set -euo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../.." && pwd)
CASE=${1:-search-filter}
WORK=${WORK:-/tmp/memos-real-fix}
TAG=v0.26.2          # git tag; the docker tag has no leading v
BACKEND_PORT=18081
WEB_PORT=13001
REENACT=$ROOT/target/debug/reenact
[ -x "$REENACT" ] || REENACT=$ROOT/target/release/reenact

case "$CASE" in
  search-filter)
    ISSUE=5462; ID=search-filter-persists-shortcut
    TITLE="Search filter persists after clicking Shortcuts even when search box is empty"
    ;;
  memo-detail-stale)
    ISSUE=5659; ID=memo-detail-stale-after-edit
    TITLE="Memo detail page not refresh after editing and saving the content"
    ;;
  *) echo "unknown case $CASE"; exit 2 ;;
esac

FIX_URL=$(python3 -c "import json;print([c for c in json.load(open('$ROOT/benchmark/cases.json')) if c['id']=='$ID'][0]['fix_pr_or_commit'])")
ISSUE_URL="https://github.com/usememos/memos/issues/$ISSUE"
[ -f "$ROOT/.env" ] && set -a && . "$ROOT/.env" && set +a
export REENACT_USERNAME=admin REENACT_PASSWORD='Passw0rd!'
mkdir -p "$WORK"

# ---- source checkout (once) --------------------------------------------------------------------
if [ ! -d "$WORK/memos/.git" ]; then
  git clone -q --depth 1 --branch "$TAG" https://github.com/usememos/memos.git "$WORK/memos"
  echo "web/pnpm-workspace.yaml" >> "$WORK/memos/.git/info/exclude"
  (cd "$WORK/memos/web" && pnpm install --frozen-lockfile --ignore-scripts >/dev/null)
fi
cd "$WORK/memos" && git checkout -q -- . && git clean -fdq && git checkout -q "$TAG" 2>/dev/null || true

cat > "$WORK/reenact.yml" <<EOF
app_url: http://localhost:$WEB_PORT
description: Memos, a self-hosted note-taking app
max_steps: 40
timeout_s: 420
login:
  path: /auth
  instruction: Sign in with username {{username}} and password {{password}}.
  signed_in_check: the sign-in form is gone and the memo app's main page is shown
EOF

python3 -c "import json;print([c for c in json.load(open('$ROOT/benchmark/cases.json')) if c['id']=='$ID'][0]['body'])" > "$WORK/$ID.md"

START="cd web && pnpm exec vite --port \$PORT --host 127.0.0.1 --strictPort"
RESET="bash $ROOT/scripts/memos-backend.sh ${TAG#v} $BACKEND_PORT"
fuser -k $WEB_PORT/tcp >/dev/null 2>&1 || true

# serve the web UI of the CURRENT checkout (scrubbed env, like the fix loop does)
serve() { env -i PATH="$PATH" HOME="$(mktemp -d)" PORT=$WEB_PORT DEV_PROXY_SERVER=http://localhost:$BACKEND_PORT \
            sh -c "$START" >/dev/null 2>&1 & echo $! > "$WORK/web.pid"
          for _ in $(seq 1 60); do curl -fs -o /dev/null http://localhost:$WEB_PORT/ && break; sleep 1; done; }
unserve() { fuser -k $WEB_PORT/tcp >/dev/null 2>&1 || true; sleep 1; }

if [ -z "${SKIP_REPRO:-}" ]; then
  echo "== 1. reproduce issue #$ISSUE as written"
  rm -rf "$WORK/cases/issue-$ISSUE"
  eval "$RESET"; serve
  "$REENACT" repro --config "$WORK/reenact.yml" --issue "$ISSUE_URL" \
    --out "$WORK/cases" --case-id "issue-$ISSUE"
  unserve
fi

echo "== 2. fix with Kiro CLI, verified by the same Kane test"
"$REENACT" fix --config "$WORK/reenact.yml" --case "$WORK/cases/issue-$ISSUE" --repo "$WORK/memos" \
  --start-cmd "$START" --app-env DEV_PROXY_SERVER=http://localhost:$BACKEND_PORT \
  --reset-cmd "$RESET" --scope web/src \
  --agent command --agent-cmd "kiro-cli chat --no-interactive --trust-tools=fs_read,fs_write" \
  | tee "$WORK/fix-$ISSUE.json"
unserve

echo "== 3. our change vs the maintainers' change"
cd "$WORK/memos"
git diff "$TAG" HEAD -- web/src > "$WORK/ours-$ISSUE.diff" || true
case "$FIX_URL" in
  */pull/*) gh api "repos/usememos/memos/pulls/${FIX_URL##*/}" -H 'Accept: application/vnd.github.v3.diff' > "$WORK/upstream-$ISSUE.diff" ;;
  *)        gh api "repos/usememos/memos/commits/${FIX_URL##*/}" -H 'Accept: application/vnd.github.v3.diff' > "$WORK/upstream-$ISSUE.diff" ;;
esac
wc -l "$WORK/ours-$ISSUE.diff" "$WORK/upstream-$ISSUE.diff"
"$REENACT" attach --case "$WORK/cases/issue-$ISSUE" --fix-diff "$WORK/ours-$ISSUE.diff" --reference-diff "$WORK/upstream-$ISSUE.diff" --agent "Kiro CLI"

echo "== 4. reference check: the maintainers' fix against the same Kane test"
git checkout -q -B reference-$ISSUE "$TAG"
git apply --include='web/src/*' "$WORK/upstream-$ISSUE.diff"
eval "$RESET"; serve
"$REENACT" verify --kind reference --config "$WORK/reenact.yml" --case "$WORK/cases/issue-$ISSUE" | tee "$WORK/reference-$ISSUE.json"
unserve
git checkout -q -- . && git checkout -q "reenact/fix-$ISSUE" 2>/dev/null || true
