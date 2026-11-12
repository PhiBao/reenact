---
mode: testing
url: http://localhost:16204/auth
headless: true
max_steps: 40
tags: [reenact, issue-5110]
---

# Filtering with tags no longer shows sub-tagged Memos

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16204/.
Stay on localhost:16204 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor on the home page
2. Type "#Tag/Subtag" into the memo editor
3. Click the Save button to create the memo
4. In the tag list in the sidebar, click the "Tag" tag to filter memos by it
The reporter says the correct behavior is that After filtering by the tag "Tag", the memo containing "#Tag/Subtag" is visible in the memo list. Verify that After filtering by the tag "Tag", the memo containing "#Tag/Subtag" is visible in the memo list.
