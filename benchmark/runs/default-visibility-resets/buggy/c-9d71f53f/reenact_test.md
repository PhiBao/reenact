---
mode: testing
url: http://localhost:15316/auth
headless: true
max_steps: 40
tags: [reenact, issue-5622]
---

# Visibility selection in the submission form becomes "Private" after the post

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15316/.
Stay on localhost:15316 for the whole run and do not open any other website.
Do these in order:
1. Open the Settings page
2. In the preferences section, set 'Default memo visibility' to 'Public'
3. Open the top page (/)
4. Observe that the visibility selector in the memo editor shows 'Public'
5. Click into the memo editor and type 'test memo'
6. Click the 'Save' button
7. Observe the visibility selector in the memo editor after the memo is posted
The reporter says the correct behavior is that after the memo is saved, the visibility selector in the memo editor still shows 'Public' (the selected default), not 'Private'. Verify that after the memo is saved, the visibility selector in the memo editor still shows 'Public' (the selected default), not 'Private'.
