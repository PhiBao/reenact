# Benchmark target: Memos (usememos/memos)

## Why Memos
- Single container, SQLite, no external accounts; image per release on `ghcr.io/usememos/memos` (all tags used here exist, checked via the registry tag list; every `docker run` below was actually started).
- Large supply of closed UI bugs, many tied to a specific release. Memos ships frequent small releases, so buggy_tag and fixed_tag are adjacent (0.25.3 → 26.11.0 releases).
- Real, messy human reports (templates filled in badly, wrong versions, screenshots only, other languages).
- Other candidates were not evaluated in depth: Memos met every criterion on first pass.

## Run a given tag (fresh volume each time)
```sh
TAG=0.26.1; PORT=15230
docker rm -f memos-$TAG 2>/dev/null
docker run -d --name memos-$TAG -p $PORT:5230 ghcr.io/usememos/memos:$TAG
# wait for readiness
until curl -fs -o /dev/null http://localhost:$PORT/; do sleep 1; done
# stop + discard
docker rm -f memos-$TAG
```
No `-v` means an anonymous volume, so every container starts empty. Port inside the container: 5230. Tags have no `v` prefix (`0.26.1`, not `v0.26.1`).

## Initial user (no login wall once a user exists)
First user created becomes ADMIN. Credentials used in verification: `admin` / `Passw0rd!`.
```sh
# 0.26.x and later
curl -s -X POST http://localhost:$PORT/api/v1/users -H 'content-type: application/json' \
  -d '{"username":"admin","password":"Passw0rd!"}'
# 0.25.x (the flat body is rejected, nested body works)
curl -s -X POST http://localhost:$PORT/api/v1/users -H 'content-type: application/json' \
  -d '{"user":{"username":"admin","password":"Passw0rd!","role":"HOST"}}'
```
Alternatively use the sign-up page in the browser (first visit shows sign-up when no user exists).
Browser sign-in: open `/auth`, fill username and password, press Sign in.
API token (0.26+ only; 0.25.x uses the session cookie): `POST /api/v1/auth/signin` with `{"passwordCredentials":{"username":"admin","password":"Passw0rd!"}}` returns `accessToken`, send as `Authorization: Bearer ...`.

Seed a memo without the UI: `POST /api/v1/memos` with `{"content":"...","visibility":"PRIVATE"}` (Bearer on 0.26+, session cookie on 0.25.x).
Comments: `POST /api/v1/memos/{id}/comments` with `{"content":"...","visibility":"PRIVATE"}`.

## Reproduction conditions that matter
- Case notes in `cases.json` say what setup each case needs. Most cases only need a memo with specific content.
- Viewport: 1300–1700px wide desktop was used. The sidebar layout differs slightly across 0.25–0.31.
- Releases 0.31+ and 26.x have a new layout (Views, Spaces, rich editor); no 26.x case is included because the UI changed too much to verify cheaply (a memo seeded through the API did not show on the 26.10.0 home screen in my attempt).

## Method and honesty notes
- Each of the 12 bug cases was **executed** on buggy_tag and fixed_tag with Playwright (Chromium) driving the real UI, plus the fix commit/PR/release notes read. Observed behaviour is in each case's `notes`.
- Candidates dropped after testing (did not reproduce as written or did not differ between tags): #5581 (error toast on both tags), #5264, #5189, #5495, #4989, #5011, #5865, and #6500 (could not drive the 26.10 UI). #5368 and #5635 became controls.
- Four controls (`kind: "control"`): #4926, #5382 (closed as working-as-designed), #5368 and #5635 (steps as written work on both tags). In #5635 the real bug is different from what the reporter described, so a correct run on the report text gives "Not reproduced".
- Several bug issues have wrong or imprecise details: #5319 (typo `-[ ]`), #6325 (needs two dollar amounts), #5462 (filed on 0.25.3 but verified on 0.26.2), #6340 (filed on rc.1). They are kept because the benchmark is meant to include this noise, and are flagged in `notes`.
- Fix-to-release mapping came from `git tag --contains` on the fix commit for v0.x tags, and from the GitHub release notes for 0.30/0.31. The tested tags are adjacent releases and the behaviour change was observed directly.
- Reaction part of #5640 was not exercised; only comment deletion was (the issue lists both).
