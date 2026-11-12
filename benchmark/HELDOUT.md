# Held-out cases (Memos)

File: `heldout-cases.json` (same schema as `cases.json`, plus `frontend_only`). 8 bugs + 2 controls. None overlap with `cases.json` or its dropped list.

| Issue | Kind | Tags (buggy → fixed) | Area | frontend_only |
|---|---|---|---|---|
| #5314 setext heading from `---` | bug | 0.25.3 → 0.26.0 | markdown render | yes |
| #5236 list Enter puts marker in wrong place | bug | 0.25.2 → 0.25.3 | editor | yes |
| #5110 parent tag filter hides sub-tag memos | bug | 0.25.2 → 0.25.3 | sidebar tags/filter | no (Go filter) |
| #5194 layout select not updating | bug | 0.25.2 → 0.25.3 | display menu | yes |
| #5578 calendar title not translated on language switch | bug | 0.26.1 → 0.26.2 | sidebar calendar / user menu | yes |
| #5564 Edit member dialog fails | bug | 0.26.0 → 0.26.1 | admin settings | yes |
| #4767 link search drops spaces | bug | 0.24.4 → 0.25.0 | editor / memo relation | yes |
| #5324 `/todo` + Enter does nothing | bug | 0.25.3 → 0.26.0 | editor slash command | yes |
| #5223 pin in Explore | control | 0.25.2 / 0.25.3 | explore | n/a |
| #5594 Chinese username rejected | control | 0.26.0 / 0.26.2 | admin settings | n/a |

## Method
- Candidates came from closed `bug` issues (GitHub API), then the fix was located by `Fixes #N` / `#N` in `git log` of a local clone, the issue timeline, or the PR. Release membership came from `git tag --contains <commit>` restricted to non-rc `v0.*` tags.
- Each case was driven in headless Chromium (Playwright, venv `/tmp/pv`, scripts in `/tmp/ho/`) on both Docker tags, one container at a time on host port 16001, fresh volume, `admin` / `Passw0rd!`, removed with `docker rm -f` afterwards. All tags were pulled and started, so they exist in the registry.
- A case is kept only if the observable behaviour differed between buggy_tag and fixed_tag (bugs) or was identical (controls).
- Seeding used the API where the bug is in rendering; the actual trigger (typing, clicking menus, dialogs) was done through the UI.

## Caveats
- #5110: the issue was closed (2025-10-10) without a linked commit. The fix commit is inferred from the observed 0.25.2 → 0.25.3 change and the commit "add hierarchical tag filtering support". Not frontend-only.
- #5236: observed symptom differs slightly from the reporter's text (marker is inserted after the first typed character rather than appended to the old line); same root cause area as the fix PR.
- #5194: only the dropdown state was asserted, not the masonry visuals.
- #5324: only keyboard Enter triggers it; mouse click on the suggestion works on both tags.
- #4767: 0.24.4 → 0.25.0 is a large visual redesign, so screenshots differ a lot.
- #5594 control: the "by design" judgement comes from the server's username regex, not from a maintainer comment. Verdict wording depends on how the scorer reads "Reproduced".
- Versions in some reports differ from the tags used (#5578 reported on 0.26.0, tested on 0.26.1; #4767 reported on 0.24.3, tested on 0.24.4).

## Dropped after testing (or reading)
- #5206 (archived memos missing): on a fresh DB the archived memo shows on both 0.25.2 and 0.25.3; the reporter's case depended on an upgraded database.
- #5566 (tag lowercased): the tag stays `SomethingSomething` in the UI and API on both 0.26.0 and 0.26.1; no discriminating behaviour found.
- #5471 (guests cannot see public comments): reported on canary only; works on 0.25.3 and 0.26.0.
- #5300 (Japanese text breaks memos): posting the string works on 0.25.3 and 0.26.0.
- #5588 (datetime picker gone): no UI difference found in the editor between 0.26.0 and 0.26.2; closer to a feature change.
- #4756 / #4768 (extra blank line in lists): not reproduced with Playwright typing on 0.24.4 (cursor misplacement showed up on both 0.24.4 and 0.25.0).
- #5626 (signup page freezes): `startViewTransition` was called 226 times in 4 s on 0.26.1 vs 1 on 0.26.2, but the page was still fillable and submittable in headless Chromium, so no visible symptom for an agent.
- #5117 (reaction popover width): popover differs (178px / 2 rows vs 235px / 1 row with default reactions) but the reported overflow with many reactions was not tested; judged too ambiguous.
- #5567 (NSFW blur): closed as intentional, but `#nsfw` blur works on 0.26.1 and not 0.26.0, so it is not a clean control.
- #5260 (Leaflet crash), #5282 (tag styles), #5617/#5589 (token expiry), #5568 (clipboard paste): no cheap deterministic UI repro.
- #5459 and #5609: duplicates of #5324 and #5314.
- Releases 0.31+/26.x not used.
