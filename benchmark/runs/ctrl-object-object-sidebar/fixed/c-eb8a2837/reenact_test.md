---
mode: testing
url: http://localhost:15329/auth
headless: true
max_steps: 40
tags: [reenact, issue-5368]
---

# Memos Detail Sidebar renders values as [object Object]

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15329/.
Stay on localhost:15329 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Type a short memo text such as "test memo" into the memo editor on the home page
2. Click the Save button to post the memo
3. Click the new memo's timestamp (or open the memo) to open the Memo detail page
4. Look at the detail sidebar widget showing date/time information
The reporter says the correct behavior is that the Memo detail sidebar shows readable date/time values (for example a formatted created and updated date) and the text "[object Object]" does not appear anywhere in the sidebar. Verify that the Memo detail sidebar shows readable date/time values (for example a formatted created and updated date) and the text "[object Object]" does not appear anywhere in the sidebar.
