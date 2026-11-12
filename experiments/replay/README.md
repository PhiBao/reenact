# Experiment: does Kane's cached replay notice a regression?

**Short answer, with kane-cli 0.10.3: no, not for a visual check.** Reenact therefore re-runs saved tests live (`--author`).

`./run.sh` (needs Kane CLI logged in, Node, Chrome; costs about 60 credits) does this:

1. Author `coupon_test.md` against a **fixed** Corner Shop. A live agent run, it passes, and Kane saves a recording.
2. Replay the saved test (a plain `testmd run`, no `--author`) against the **buggy** Corner Shop, twice.
3. Run the same test live (`--author`) against the buggy shop.

The buggy shop shows **$30.00** after the coupon on a two-item cart. The test says the total must be **$10.00**.

Result from the last run, saved in [`result.txt`](result.txt):

| Step | Status | Total the run saw |
|---|---|---|
| Author on fixed app | passed | `$10.00` |
| Replay on buggy app (1) | **passed**, `replay completed` | `$10.00` (the recorded value) |
| Replay on buggy app (2) | **passed**, `replay completed` | `$10.00` (the recorded value) |
| Live run on buggy app | **failed**, "the Total shows $10.00" | `$30.00` |

The replayed run reports the value captured when the test was authored, not the page in front of it.

## Caveats

- One app, one test, one Kane version. It is a reproducible experiment (`run.sh`), not a survey.
- The assertion is a visual one ("the Total shows $10.00"). We did not test DOM-level or network assertions, which may be re-evaluated on replay.
- Replay is a feature meant for speed and cost; this is about what it can be trusted to prove, not a defect report.

## What to tell TestMu

> Replayed `testmd run` steps that contain visual extractions/assertions return the recorded value (`context.variables` shows `$10.00`) even when the page shows `$30.00`, and the run is reported `passed / replay completed`. Is the check expected to be re-evaluated on replay? Repro: https://github.com/PhiBao/reenact/tree/main/experiments/replay
