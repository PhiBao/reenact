---
mode: testing
url: http://localhost:15317/auth
headless: true
max_steps: 40
tags: [reenact, issue-5622]
---

# Visibility selection in the submission form becomes "Private" after the post

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15317/.
Stay on localhost:15317 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Open the Settings page
2. In the preferences settings, set 'Default memo visibility' to 'Public'
3. Open the top page (/)
4. Observe that the Visibility selector in the memo editor shows 'Public'
5. Click the memo editor text area
6. Type 'test memo'
7. Click the 'Save' button
8. Observe the Visibility selector in the memo editor after the memo is posted
The reporter says the correct behavior is that After the memo is saved, the Visibility selector in the memo editor still shows 'Public' (the chosen default) and not 'Private'. Verify that After the memo is saved, the Visibility selector in the memo editor still shows 'Public' (the chosen default) and not 'Private'.
