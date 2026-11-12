---
mode: testing
url: http://localhost:15312/auth
headless: true
max_steps: 40
tags: [reenact, issue-6325]
---

# Using dollar sign causes improper formatting

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15312/.
Stay on localhost:15312 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor input on the home page
2. Type "with the local markets... $22 for the smallest bit of rice and fried chicken."
3. Click the Save button to create the memo
4. Look at the rendered memo in the memo list
The reporter says the correct behavior is that the saved memo shows the text exactly as typed: "with the local markets... $22 for the smallest bit of rice and fried chicken.", with the "$22" and the ellipsis visible and no broken or math-style formatting. Verify that the saved memo shows the text exactly as typed: "with the local markets... $22 for the smallest bit of rice and fried chicken.", with the "$22" and the ellipsis visible and no broken or math-style formatting.
