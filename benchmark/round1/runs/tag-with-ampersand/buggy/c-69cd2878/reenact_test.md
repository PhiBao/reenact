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
Do these in order:
1. Click on the memo editor input at the top of the home page
2. Type "#science&tech" into the memo editor
3. Click the Save button to create the memo
4. Look at the newly created memo in the memo list
The reporter says the correct behavior is that the saved memo shows the entire string "#science&tech" as a single tag, rather than only "#science" as a tag followed by plain text "&tech". Verify that the saved memo shows the entire string "#science&tech" as a single tag, rather than only "#science" as a tag followed by plain text "&tech".
