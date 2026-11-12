---
mode: testing
url: http://localhost:15314/auth
headless: true
max_steps: 40
tags: [reenact, issue-5620]
---

# Tags with & in them are now broken (won't display correctly) and can no longher be created

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15314/.
Stay on localhost:15314 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor input on the home page
2. Type "#science&tech" into the editor
3. Click the Save button to create the memo
4. Look at the newly created memo in the memo list
The reporter says the correct behavior is that the saved memo shows a single tag "#science&tech" with the whole string, including "&tech", rendered as part of the tag. Verify that the saved memo shows a single tag "#science&tech" with the whole string, including "&tech", rendered as part of the tag.
