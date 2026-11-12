# Reenact

**Bug report in. Browser reproduction and proof out.**

Reenact reads a GitHub issue, makes a real browser do what the reporter did using [Kane CLI](https://www.testmuai.com/kane-cli/), and tells the maintainer one of four things: *Reproduced*, *Not reproduced*, *Needs info* or *Inconclusive*. The verdict comes with screenshots, Kane's own triage and the plain-English test it ran. If a maintainer comments `/reenact fix`, a coding agent edits the code and **the same test has to pass** before a pull request opens.

Built for the [Kane CLI Online Hackathon](https://luma.com/kanecli-online-hackathon) (12 to 19 November 2026), lane 2 ("workflows with a spine"), with Kane also used as the product's runtime (lane 3) and as the build loop for its own UI.

| | |
|---|---|
| Live case files | **https://reenact-kane.fly.dev** |
| **Fixes of real bugs in a real app** | [Memos #5462](https://reenact-kane.fly.dev/c/issue-5462) and [Memos #5659](https://reenact-kane.fly.dev/c/issue-5659): Kiro CLI fixed both, Kane verified, and the maintainers' own fixes pass the same tests ([how](#a-fix-on-a-real-app)) |
| Demo repo (real GitHub Actions runs) | [PhiBao/reenact-demo-shop](https://github.com/PhiBao/reenact-demo-shop): issues [#1](https://github.com/PhiBao/reenact-demo-shop/issues/1) [#3](https://github.com/PhiBao/reenact-demo-shop/issues/3) [#4](https://github.com/PhiBao/reenact-demo-shop/issues/4) [#5](https://github.com/PhiBao/reenact-demo-shop/issues/5), PRs [#2](https://github.com/PhiBao/reenact-demo-shop/pull/2) [#6](https://github.com/PhiBao/reenact-demo-shop/pull/6) [#7](https://github.com/PhiBao/reenact-demo-shop/pull/7) [#8](https://github.com/PhiBao/reenact-demo-shop/pull/8) |
| Benchmarks, misses included | [Benchmark page](https://reenact-kane.fly.dev/benchmark) and [`benchmark/`](benchmark/) |
| Demo video | _linked in the submission form_ |

![The Reenact loop](docs/diagrams/loop.svg)

## The problem

Every maintainer knows the first hour of a bug report: read a vague issue, set up the app, click around, and either find the bug or write "can't reproduce, which version?". Teams that automated this (Astro with Cloudflare, Metabase's Repro-Bot) each built a one-off tool tied to their codebase, with hand-written recipes for their app. AI has made filing issues nearly free and left reproducing them expensive.

Reenact's bet: a bug report and a Kane test are both plain English, so reproduction does not need per-app code. A report becomes a test of the *expected* behavior. If the test fails in a real browser and Kane's investigation says the app is at fault, the bug is real, and the same test becomes the acceptance test for the fix.

## Who it is for

Maintainers of web apps on GitHub, open source first, then small product teams who triage user reports. The first moment of value is the first issue comment that shows *your* bug on *your* app with the failing frame circled.

## What happens

1. **Intake.** The issue (and, on a re-run, the reporter's answers in the thread) is compiled by an LLM into a fixed JSON plan: start path, steps, what *should* happen, what the reporter says happens, and what is missing. Reports that are not a browser bug, or have no steps, stop here as *Needs info* and cost no browser run. A guessed start page that returns 404 is replaced by the home page, so a missing route is never reported as the bug.
2. **Render.** Rust (not the model) writes a Kane `_test.md` from that plan, inside a fixed scaffold. Issue text never becomes a URL, a flag or a variable name.
3. **Reenact.** `kane-cli testmd run --agent --bug-detection stop` drives Chrome while a comment on the issue fills in step by step. Reenact reads the NDJSON events and the sealed evidence pack, not the exit code. If the first run does not reproduce the bug it runs once more, because a false "Not reproduced" is the common error of a browser agent.
4. **Judge.** *Reproduced* needs the check at the end of the reproduction step to fail, Kane's investigation to class it as an application defect, **and** at least one real user action to have happened first. A failed check Kane blames on the test, a run that got stuck, or a failure before any step was performed is *Needs info*, never *Reproduced*.
5. **Comment and label.** A strip of Kane's screenshots, cropped to the part of the page that matters and ringed where the user acted; the steps taken; Kane's read on the cause; browser console and network signals; the test itself; and one `reenact:*` label on the issue.
6. **Needs info loop.** When the verdict is *Needs info*, the reporter's reply re-runs Reenact with the answer added.
7. **Fix (on `/reenact fix`).** The bug must reproduce a **second** time before anyone edits code. A fixer then changes the code: the built-in LLM editor, or any coding agent run as a command (we use **Kiro CLI**). The app restarts with a scrubbed environment and **the same Kane test runs live**. A change is accepted only if it passes **and** every earlier saved test still holds. Failures are fed back, up to three attempts. On success: a branch, a pull request that is never merged automatically, a before/after comparison and the test committed under `reenact/tests/`.
8. **Keep.** `reenact suite` re-runs every saved reproduction live. If a fixed bug returns, it exits non-zero with the evidence.

## What is new

- **The test comes from the reporter's words, not from the code or the coding agent.** Tests written by the same agent that wrote the code share its blind spots. Here the expectation is lifted from the issue before anyone looks at the diff.
- **No per-app recipes.** The tools that exist for this need code written for one codebase. Reenact takes a URL, an optional sign-in and a sentence about the app.
- **Reproduction is the acceptance test.** One artifact serves as bug proof, fix gate and regression test.
- **The fixer cannot mark its own homework.** It cannot edit tests or CI, it runs with no secrets in reach of the app it changed, a second independent run must see the bug first, and an earlier fix it breaks is rejected.
- **Honest verdicts.** Four outcomes, and *Not reproduced* explicitly does not claim the bug is absent.

## A fix on a real app

Toy apps prove little, so the strongest evidence is two fixes of **real, filed bugs in [Memos](https://github.com/usememos/memos)** (a popular open-source notes app), done end to end by `experiments/memos-real-fix/run.sh`:

- Memos is checked out at the release the bug was filed on (v0.26.2) and its web UI is served from source in front of the matching official backend image.
- Reenact reproduces the upstream issue **as written**, with the retry policy above.
- `reenact fix` confirms the bug a second time, then **Kiro CLI** edits the code (restricted to `web/src`). Kane re-runs the same test.
- Finally the **maintainers' own fix** is applied to a clean checkout and run through the same test, to check the test accepts a correct fix.

| Upstream issue | Result | Kiro's change vs the maintainers' |
|---|---|---|
| [#5462](https://github.com/usememos/memos/issues/5462) search filter persists after clicking a shortcut | Reproduced twice, **fixed on attempt 1**, maintainers' fix also passes | Same file, same condition, same filter. Only the statement order differs. [Case file](https://reenact-kane.fly.dev/c/issue-5462) |
| [#5659](https://github.com/usememos/memos/issues/5659) memo detail page stale after editing | Reproduced twice, **fixed on attempt 1**, maintainers' fix also passes | One line: invalidate all cached memo details. The maintainers invalidate the edited memo's detail only. Broader, same effect. [Case file](https://reenact-kane.fly.dev/c/issue-5659) |

Both case files show the change, the maintainers' change side by side, and the before/after of the same page. What this does and does not show: two small frontend bugs, chosen because their upstream fixes are small and in `web/`; not a claim that Reenact fixes arbitrary bugs. The agent saw the same checkout and the issue text, as a maintainer's agent would.

## Hackathon relevance

| Judging criterion | Where it shows |
|---|---|
| **Ships** | Deployed case-file site; real GitHub Actions workflow on the demo repo; `scripts/demo.sh` runs the whole thing locally with one command. |
| **Verified** | Kane reproduced real bugs from real issues of an open-source app, benchmarked on buggy and fixed releases with the misses published, and it verified two real fixes whose upstream fixes pass the same tests. Kane also failed this project's own UI checks once during the build (see "Kane in the build loop"). |
| **Closed loop** | Kane's failure evidence is the fix agent's input; the agent's output triggers a fresh Kane run of the same test; repeat until pass or give up. In CI this is `issue → comment → /reenact fix → PR`, and a *Needs info* verdict loops back to the reporter. |
| **Craft** | Installs by adding one workflow file and `.reenact.yml`. Every comment is a link to a case file. The UI is an "evidence room", not a dashboard. |

Kane is used three ways: **runtime** (it reproduces the bug), **gate** (it is the only judge of whether a fix worked) and **build loop** (a Kiro hook runs Kane against this repo's UI on save, `scripts/verify-ui.sh`).

## Try it

### 30 seconds, no setup

Open **https://reenact-kane.fly.dev**. Drag the before/after slider on the front page (a real Memos fix), open the case file, and compare Kiro's diff with the maintainers'. Then read an [issue thread](https://github.com/PhiBao/reenact-demo-shop/issues/5) where a vague report got a *Needs info*, the reporter answered, and the bug was reproduced, fixed and [turned into a pull request](https://github.com/PhiBao/reenact-demo-shop/pull/8) that re-ran three earlier tests.

> The demo repo's Action only auto-runs for the repo owner and collaborators, and anyone else's issue needs a maintainer to add the `reenact` label. This is deliberate: issue text is untrusted and every run costs credits.

### Run the whole thing locally

Prerequisites: Rust (stable), Node 22, Google Chrome, [Kane CLI](https://www.testmuai.com/kane-cli/) (`npm install -g @testmuai/kane-cli`, then `kane-cli login`) and an OpenAI-compatible key.

```bash
cp .env.example .env     # set REENACT_LLM_API_KEY (and base URL / model if not OpenAI)
./scripts/demo.sh        # starts the Corner Shop app, reproduces "Coupon makes my total go UP"
./scripts/demo.sh remove-wrong-item
```

It prints the verdict and writes a case folder under `.reenact/cases/`. To browse it: `cd web && pnpm install && pnpm build`, then `REENACT_CASES_DIR=.reenact/cases cargo run -p reenact-server`.

To repeat a real-app fix (Docker, Node 22 with pnpm, Chrome, `kane-cli` and `kiro-cli` logged in): `./experiments/memos-real-fix/run.sh search-filter` (about 20 minutes, 300 to 500 Kane credits).

### On your own repo

`.reenact.yml` (see [`demo/shop/.reenact.yml`](demo/shop/.reenact.yml)):

```yaml
app_url: http://localhost:4100
description: One sentence about your app
# login:                       # only if the app needs a signed-in user
#   path: /auth
#   instruction: Sign in with username {{username}} and password {{password}}.
#   signed_in_check: the sign-in form is gone and the main page is shown
```

Then copy [`.github/workflows/reenact.yml`](https://github.com/PhiBao/reenact-demo-shop/blob/main/.github/workflows/reenact.yml) and set the secrets `TESTMU_USERNAME`, `TESTMU_ACCESS_KEY`, `REENACT_LLM_API_KEY`. Use a throwaway test account and a disposable instance of your app, never production. Leave `start-command` empty to test an app that is already running at `app_url` (a preview deployment); `fix` mode needs it, plus `fix-agent` / `fix-scope` if you want a coding agent instead of the built-in editor.

## CLI

```text
reenact repro  --issue <url> [--with-thread] [--post]    reproduce a report (live comment, labels)
reenact fix    --case <dir> --repo <dir> --start-cmd ... confirm, fix, verify, gate on earlier tests (optionally --pr)
reenact suite  [--dir reenact/tests]                     re-run saved reproductions live
reenact sweep  --repo owner/name                         triage a project's open issues, read-only
reenact bench  [--cases file]                            historical benchmark on Memos
reenact verify --case <dir> [--kind reference]           re-run a case's test now
reenact ship / restrip / attach                          publish, recompose images, attach diffs
```

## Architecture

```text
GitHub issue ─▶ reenact (Rust CLI, runs in the repo's Action)
                 ├─ compile  LLM ─▶ Plan (fixed JSON schema) ─▶ sanitize ─▶ start path check
                 ├─ render   Plan ─▶ Kane _test.md (deterministic)
                 ├─ run      kane-cli testmd run --agent  (own process group, 0600 creds file)
                 ├─ read     NDJSON events + sealed .evidence zip (frames, triage, console, HAR)
                 ├─ judge    four verdicts
                 ├─ fix      confirm ─▶ fixer (LLM editor | coding agent command) ─▶ scrubbed app ─▶ same test, live ─▶ earlier tests
                 └─ ship     images ─▶ evidence branch · bundle ─▶ reenact-server · comment + label ─▶ issue
reenact-server (Axum) ─ serves case folders + web UI (Vite/React) with per-case social cards
```

| Crate / dir | Role |
|---|---|
| `crates/reenact-core` | Compiler, sanitizer, renderer, Kane runner, evidence reader, verdict, image composer, GitHub comment. Unit-tested, including parsers run on real recorded Kane output and a real evidence pack. |
| `crates/reenact-cli` | The `reenact` binary. |
| `crates/reenact-server` | Case-file API and static UI host. A case is a folder (`case.json`, images). Uploads are bearer-token, zip-slip-safe and allow-listed. |
| `web/` | Case-file UI. Hand-written CSS, Instrument Serif + Geist, no component kit. |
| `action.yml` | Composite GitHub Action that installs Kane and Reenact and runs `repro`, `fix` or `suite`. |
| `benchmark/` | Memos benchmarks (main set, held-out set), sweep results, evidence. |
| `experiments/` | Reproducible experiments: real-app fix, cached-replay check. |
| `demo/shop` | Corner Shop, the demo target with planted bugs. |

Why Rust for the engine: it spawns and supervises a browser agent, parses untrusted text and evidence archives, and ships as one binary that downloads in a second in CI. The server is Axum with the filesystem as storage on purpose: a case is a folder you can `ls`.

![Trust boundaries](docs/diagrams/trust.svg)

## Verification: what is real

| Claim | Status |
|---|---|
| Reproduction of real issues in a real browser | **Real.** Kane CLI 0.10.3 driving Chrome. |
| Fixes of real bugs | **Real, two cases**, by Kiro CLI on Memos, checked against the maintainers' fixes. Run locally, not in CI (Kiro needs a logged-in session). |
| Fix loop in CI | **Real** on the Corner Shop demo, with the built-in editor. **The demo's bugs are planted by us.** |
| Historical benchmarks | **Real** Memos releases from Docker, issues written by real users. Reenact's plan is compiled once and run on both releases. |
| GitHub Action, live comment, labels, PRs, reporter-reply loop | **Real.** See the demo repo runs. |
| Case-file site | **Real**, deployed on Fly.io. |
| Cached replay as a regression check | **Not used, and why:** see "What building on Kane taught us". |
| Mocked / simulated | Nothing in the product path. `web/fixtures/` holds a copy of one real case for the UI tests. |

## Benchmarks

Every row is a closed bug report from [Memos](https://github.com/usememos/memos), tied to the release that had the problem and the release that fixed it, run from pinned Docker images. Reenact compiles each report **once**, then runs the same plan against both releases. A correct result is *Reproduced* on the buggy release and *Not reproduced* on the fixed one. Controls are reports that are not bugs. Full tables, every verdict and Kane's reason: the [Benchmark page](https://reenact-kane.fly.dev/benchmark), `benchmark/*/results.json`, and one folder of evidence strips per run.

### Held-out set: reports it had never seen

Ten new reports (8 bugs, 2 controls) were collected **after the prompts and verdict logic were frozen** (commit `80b9d59`) and run once. Nothing was tuned on them. This is the number to believe.

| Held-out, 7 scored bugs | |
|---|---|
| Reproduced on the buggy release | **4 of 7** (3 of 7 by the first run alone; the automatic retry found one more) |
| Wrongly *Reproduced* on the fixed release | 1 of 7 |
| Not reproduced on the fixed release | 5 of 7 |
| Non-bug controls called *Reproduced* | 3 of 4 runs (see below) |
| Judged exactly right on both releases | 2 of 10 |
| Not scored | 1 bug: our test harness could not create the first user on release 0.24.4 before Reenact ran |

Where it failed: three reports were *Not reproduced* on the buggy release (an editor cursor position after Enter, a tag-hierarchy filter, a calendar label that should switch language), and one *Reproduced* on both releases because typing the report's text into the editor changed it (typing `---` triggers editor features) so the saved memo differed on both. A sample of seven is small; read it as "roughly half".

### Main set, tuned: the 16 reports we developed against

| | Round 1 | Round 2 |
|---|---|---|
| Bugs reproduced on the buggy release (of 12) | 5 | **8** |
| Bugs cleared on the fixed release (of 12) | 11 | 9 |
| Bug reports wrongly *Reproduced* on the fixed release | 1 | **0** |
| *Reproduced* verdicts on the 4 controls (8 runs) | 1 | 4 |
| Both releases judged correctly (of 16) | 7 | **8** |

Round 2 ran after two general changes found in round 1 (preconditions are no longer rendered into the test; the scaffold tells Kane not to undo or work around unexpected behavior). They were made after seeing round-1 results, so round 2 is tuned and the held-out set is the honest comparison: 8 of 12 tuned against 4 of 7 held out.

### How to read the controls

Controls are reports where the maintainers say the behavior is expected, or where the described behavior is identical on both releases. Reenact called most of them *Reproduced* because the page does behave as the report describes. That is not a false claim about the browser; it is the limit of the approach, which sees behavior and not intent. The comment always states the expected behavior it tested, so a maintainer can overrule it in one line.

Other notes: results vary run to run (`tag-with-ampersand` was reproduced in round 1 and not in round 2 with the same report); several reports are deliberately noisy (a typo in the steps, a wrong version, a screenshot-only report); about 35 Kane credits and 2 to 3 minutes per run. Reproduce a run with `reenact bench --cases benchmark/heldout-cases.json --out benchmark/heldout` (needs Docker, Kane CLI and an LLM key).


### Backlog sweep: the open issues of a real project, read-only

`reenact sweep` triages a project's open issues against a running instance and posts nothing to the project. We pointed it at the 12 newest open issues of Memos on its latest release (26.11.0), with no labels to go by: [`benchmark/sweep/SWEEP.md`](benchmark/sweep/SWEEP.md).

| Result | Issues |
|---|---|
| Reproduced | 1: [#6530](https://github.com/usememos/memos/issues/6530) custom URI links such as `obsidian://...` are shown as plain text ([case file](https://reenact-kane.fly.dev/c/sweep-6530)) |
| Not reproduced | 1: [#6550](https://github.com/usememos/memos/issues/6550) numbered lists over 10 items overflow ([case file](https://reenact-kane.fly.dev/c/sweep-6550)) |
| Not a browser-visible bug (feature request, docs, Safari-only input, WIP notes) | 10, sorted out in seconds without a browser run |

The whole sweep used about 91 Kane credits. We have not checked #6530 with the Memos maintainers and make no claim about their roadmap; the table says what Kane saw on that release, and the maintainers decide.


## What building on Kane taught us

These findings changed the design. They are useful to anyone wiring Kane into an agent.

1. **Cached replays cannot catch regressions.** After Kane authors a passing test, `testmd run` replays the recording without the model. We ran the saved test against the *buggy* app and it still **passed**, twice, while the page showed `$30.00` and the test required `$10.00`: the replay reported the value recorded when the test was authored. Scripted and repeatable: [`experiments/replay`](experiments/replay). So `reenact suite`, every verification and our UI checks run with `--author`, a live agent run each time. It costs credits (about 20 to 30 per test) and is the only way to get a real answer.
2. **Trust the events, not the exit code.** We take the verdict from `run_end`, `reason_code` and `test_md_bug_verdict`. `--bug-detection stop` is what produces a *confirmed product bug* classification; without it a failed assertion reads as a test problem.
3. **Preconditions become checkpoints.** Writing "make sure a memo exists" into a test makes Kane check it *before* the steps that create it, and fail. Reenact never renders preconditions.
4. **Kane repairs what it was asked to observe.** On the "Remove deletes the wrong item" report, Kane noticed the wrong item vanish, restored it, removed the right one and passed. The scaffold now says: do each step once, do not undo or work around anything unexpected, check right after the last step.
5. **A missing page looks like a confirmed bug.** On a reporter's follow-up, the compiler guessed `/cart`, Kane got a 404, and its triage called that a confirmed product bug. Reenact now checks guessed start pages and refuses *Reproduced* when no user action happened.
6. **Process groups.** Killing the `kane-cli` child leaves Chrome running. Each run leads its own group and is killed as a group, as GuardianKane's author also found.

## Kane in the build loop

`scripts/verify-ui.sh` builds the web UI, serves fixture cases, and has Kane check two pages in plain English (`tests/ui/*_test.md`), live with `--author`. A Kiro hook (`.kiro/hooks/kane-verify-ui.json`) runs it when a file under `web/src/` is saved. Kane failed this suite once during the build: a stale server from an earlier run was answering on the test port with the wrong data, so the Benchmark page showed its error state and Kane reported the table missing. The script now refuses to run if the port is busy. The first version of the script also ran cached replays and reported `replay completed`; that is how we noticed finding 1.

## Security and reliability

- **Issue text is hostile input.** It is clipped, stripped of control and bidi characters, URLs and Kane's template syntax (`{{ }}`, `@import`), and the model may only return a fixed JSON shape. The start URL is `app_url + a checked path`; the model never supplies one. See `crates/reenact-core/src/sanitize.rs` (unit-tested against injected payloads).
- **Prompt injection is bounded, not solved.** A hostile report could still try to steer the browser agent or the fixing agent. The blast radius is a throwaway test account on a disposable instance, a "stay on this host" instruction, a fresh browser profile, and a pull request that a human reviews and that is never merged automatically.
- **The app under test never sees CI secrets.** After a model edits code, that code runs. `reenact fix` starts it with `env_clear`: only `PATH`, `PORT`, a temporary `HOME`, `CI` and variables the maintainer listed. Unit-tested. On a developer machine it can still read files in your home directory, so run `fix` in CI or a container.
- **Who can spend money or change code.** Auto-run is limited to repo owners, members and collaborators; anyone else's issue needs a maintainer's `reenact` label. `/reenact fix` is honored only from people with write access, and never on pull requests.
- **The fixer cannot grade itself.** Tests, CI, `reenact/`, lock files and config are off limits (checked after every attempt, with a change-size cap and an optional `--scope`); the test is re-rendered from the case file each attempt; the bug must reproduce twice first; only a live Kane run counts; an earlier saved test that breaks rejects the attempt.
- **Secrets.** Test credentials reach Kane through a `0600` variables file as secret variables, never argv. GitHub tokens reach `git` through environment config, not the command line. Kane evidence packs are sealed unredacted, so treat them as sensitive. They are not published; only screenshots, the case JSON and Kane's share link are.
- **Server.** Uploads need a bearer token (constant-time compare), reject paths outside the case folder, accept only `case.json` and images, and cap sizes. The container runs as root because Fly volumes mount root-owned; it holds only case folders.
- **Failure states.** Kane error, timeout and credit exhaustion produce *Inconclusive* and say nothing about the bug. A fix that fails three times is reported with what was tried.

## Limitations

- **Recall is limited.** On reports it had never seen, Reenact reproduced about half of real bugs (see Benchmarks). Visual or subtle rendering bugs and anything that depends on exactly how text is typed into an editor are the main misses.
- **It sees behavior, not intent.** A control that merely describes behavior the maintainers consider correct is *Reproduced*, because the page does behave that way. The comment states the expected behavior it tested; a human decides.
- Plans are compiled by an LLM, so two runs of the same issue can differ. The benchmarks compile once per case and run both releases with the same plan; results vary run to run and the tables say so.
- A report that depends on data, settings or versions the fresh test instance does not have will not reproduce. *Not reproduced* means "not on this version with this data".
- Kane runs cost credits (about 20 to 60 per run) and take 1 to 4 minutes.
- Fixing real apps needs a way to serve the app from source and a coding agent; two small frontend bugs are shown, not a general claim.
- Web desktop only. Kane supports mobile; Reenact does not use it yet. Single-tenant case-file server; case ids are `issue-<n>`, so two repos on one server would collide.

## Roadmap

Version bisect across releases ("reproduces in 0.24, fixed in 0.26"), mobile targets through Kane, more fixer integrations (Codex, Claude Code) in CI, support tickets from Intercom and Zendesk as report sources, and a hosted GitHub App.

## Develop

```bash
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
(cd web && pnpm install && pnpm build)
./scripts/verify-ui.sh        # Kane checks the UI in a real browser
```

Environment: `REENACT_LLM_API_KEY`, `REENACT_LLM_BASE_URL`, `REENACT_LLM_MODEL`, `REENACT_FIX_MODEL` (optional), `REENACT_USERNAME` / `REENACT_PASSWORD` (test account), `KANE_CLI_BIN`. Server: `PORT`, `REENACT_CASES_DIR`, `REENACT_WEB_DIR`, `REENACT_BENCHMARK_FILE`, `REENACT_UPLOAD_TOKEN`, `REENACT_PUBLIC_URL`.

## Credits

Built with Kiro and Kane CLI by TestMu AI. Licensed under Apache-2.0. Memos is a project of the Memos authors and is used here only as a test subject; nothing was posted to its repository.
