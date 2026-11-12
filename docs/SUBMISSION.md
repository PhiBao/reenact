# Submission kit

Everything needed for the form at https://www.surveymonkey.com/r/kane-cli-hackathon-submission. Deadline: **19 November, 11:59 PM IST** (10:29 AM PT). Submissions lock at the deadline.

## Fields

- **Repo (public):** https://github.com/PhiBao/reenact (initialised 12 November 2026)
- **Live URL:** https://reenact-kane.fly.dev
- **Runnable command:** `./scripts/demo.sh` (needs Kane CLI logged in, Chrome, Node, an OpenAI-compatible key)
- **Demo video (YouTube, unlisted is fine, keep the scoring material inside the first 3:00):** _paste link_
- **Launch posts (at least two, different public posts):** _paste links_

## One paragraph

Reenact turns a GitHub bug report into a browser reproduction with proof. A Rust engine compiles the issue into a fixed plan, renders a plain-English Kane test, and runs it with Kane CLI in a real Chrome while a comment on the issue fills in live; the verdict (Reproduced, Not reproduced, Needs info, Inconclusive) comes from Kane's structured events and failure triage, not the exit code. If the report is vague the reporter's answer re-runs it. On `/reenact fix` the bug must reproduce a second time, then a coding agent (Kiro CLI on real Memos bugs) edits the code, the same Kane test must pass with the app running in a scrubbed environment, and every earlier saved test must still hold, before a pull request opens. Kane fixed-and-verified two real filed Memos bugs whose maintainers' own fixes pass the same tests, and a held-out benchmark of unseen reports (about half reproduced, one false alarm on a fixed release) is published with its misses. It is for maintainers of web apps on GitHub. Built with Kiro; Kane CLI is the runtime that reproduces, the judge that verifies fixes, and the checker for the product's own UI.

## Video storyboard (3:00 hard stop; the good part first)

| Time | Show | Say |
|---|---|---|
| 0:00 | Issue #1 on the demo repo, as written ("Coupon makes my total go UP"). Cut to the Reenact comment with the strip. | "A vague bug report. Two minutes later, a real browser has already done what the reporter did." |
| 0:15 | The comment: expected behavior, steps Kane took, ringed element, Kane's read on the cause. | "Kane reproduced it and its own investigation says it is the app, not the test." |
| 0:35 | Comment `/reenact fix` on issue #3 (quantity goes negative). Show the Actions log: attempt, Kane run, result. | "Now the agent has to earn it. The same Kane test is the only judge." |
| 1:05 | The pull request and the before/after slider on https://reenact-kane.fly.dev. | "The test that failed now passes, and it is committed in the repo." |
| 1:20 | Case file for Memos #5462: Kiro's diff next to the maintainers' diff, before/after slider. | "A real bug in a real app. Kiro's fix matches the maintainers', and their fix passes our test too." |
| 1:40 | Benchmark page, held-out tab. Point at a miss. | "Ten reports we never tuned on. About half reproduced. We publish the misses." |
| 2:00 | Terminal: `reenact suite` result on branch `wip/coupon-refactor`: one test REGRESSED with Kane's reason, two hold. | "Months later someone brings the bug back. The suite catches it." |
| 2:25 | Finding: cached replay passes on the buggy app, so the suite runs live. Show `kane-cli` replay output. | "We found Kane's cached replay cannot catch a regression, so we do not use it for that." |
| 2:45 | Front page. | "Every bug report, reenacted." |

Record at 1080p, terminal font 18+, browser zoom 110%. Set the video to Unlisted, test the link in an incognito window.

## Launch posts

**Post 1 (X / LinkedIn)**

> Maintainers spend the first hour of every bug report trying to reproduce it.
>
> Reenact does it for you: it reads the issue, makes a real browser (Kane CLI) do what the reporter did, and comments back with screenshots and a verdict. Comment /reenact fix and an agent has to pass the same Kane test before a PR opens.
>
> Live: https://reenact-kane.fly.dev
> Code: https://github.com/PhiBao/reenact
> #KaneCLI #TestMuAI

**Post 2 (X thread or dev.to) — the finding**

> We built on Kane CLI for the #KaneCLI hackathon and found something worth knowing: cached replay cannot catch a regression.
>
> We ran a saved test against the buggy app. It passed. The run context still held the $10.00 recorded when the test was authored while the page said $30.00.
>
> So Reenact's regression suite runs live every time. Details and the other four lessons: https://github.com/PhiBao/reenact#what-building-on-kane-taught-us

**Show HN draft**

> Title: Show HN: Reenact – reproduce GitHub bug reports in a real browser, then prove the fix
>
> I wanted the first hour of bug triage to be automatic. Reenact compiles an issue into a fixed JSON plan, renders a plain-English Kane CLI test in Rust, runs it in Chrome and judges the result from Kane's structured events and failure triage. Comment /reenact fix and an agent edits the code; the same test must pass before a PR opens. I benchmarked it on 16 closed Memos issues against the buggy and the fixed release and I list the misses in the README. Honest limits: visual/rendering bugs are hard, and it should only be pointed at a disposable test instance. https://github.com/PhiBao/reenact

## Pre-submit checklist

- [ ] `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `pnpm build` green
- [ ] `./scripts/verify-ui.sh` passes
- [ ] https://reenact-kane.fly.dev loads, case pages open, Benchmark page populated
- [ ] README numbers match `benchmark/runs/RESULTS.md`
- [ ] Demo repo issues #1 #3 #4 #5 show comments; PRs #2 #6 #7 merged
- [ ] Video unlisted link works in incognito, first 3:00 contains the Kane loop
- [ ] Two public posts live; links pasted above
- [ ] Rotate keys that were shared during the build (SandBase, TestMu access key, Fly upload token)
