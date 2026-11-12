---
mode: testing
url: http://localhost:15310/auth
headless: true
max_steps: 40
tags: [reenact, issue-6340]
---

# tel: and sms: URI schemes not supported in links.

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the user is signed in and the main memo page is visible.

## Reproduce the report
Open http://localhost:15310/.
Stay on localhost:15310 for the whole run and do not open any other website.
Do these in order:
1. Open the memo editor to create a new memo
2. Type `[phone me](tel:+440000000000)` into the memo editor
3. Click the Save button to create the memo
4. Locate the saved memo in the memo list and look at the rendered 'phone me' link
The reporter says the correct behavior is that the rendered memo shows a 'phone me' link that is a clickable anchor with href 'tel:+440000000000'. Verify that the rendered memo shows a 'phone me' link that is a clickable anchor with href 'tel:+440000000000'.
