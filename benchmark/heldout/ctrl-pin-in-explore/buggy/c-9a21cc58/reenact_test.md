---
mode: testing
url: http://localhost:16216/auth
headless: true
max_steps: 40
tags: [reenact, issue-5223]
---

# Pinning memos in the "Explore section" not working

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16216/.
Stay on localhost:16216 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Type "older memo" into the memo editor on the home page
2. Click the Save button to create the first memo
3. Type "newer memo" into the memo editor
4. Click the Save button to create the second memo
5. Open the Explore section from the navigation
6. Open the menu on the "older memo" entry in the Explore list
7. Click Pin in the menu
The reporter says the correct behavior is that After pinning, the "older memo" entry appears at the top of the Explore list above "newer memo" and is marked as pinned. Verify that After pinning, the "older memo" entry appears at the top of the Explore list above "newer memo" and is marked as pinned.
