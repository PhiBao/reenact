---
mode: testing
url: http://localhost:16206/auth
headless: true
max_steps: 40
tags: [reenact, issue-5194]
---

# Selecting a layout doesn't update the selection or layout until the page is refreshed

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:16206/.
Stay on localhost:16206 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Open the layout selector
2. Select the Masonry layout
The reporter says the correct behavior is that After selecting Masonry, the layout selector shows Masonry as the selected layout and the memo list is displayed in the masonry layout without refreshing the page. Verify that After selecting Masonry, the layout selector shows Masonry as the selected layout and the memo list is displayed in the masonry layout without refreshing the page.
