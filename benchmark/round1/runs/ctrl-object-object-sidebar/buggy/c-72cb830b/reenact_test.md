---
mode: testing
url: http://localhost:15328/auth
headless: true
max_steps: 40
tags: [reenact, issue-5368]
---

# Memos Detail Sidebar renders values as [object Object]

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15328/.
Stay on localhost:15328 for the whole run and do not open any other website.
Do these in order:
1. Type "do tags nest?" into the memo editor on the home page
2. Click the Save button to post the memo
3. Click the newly posted memo's timestamp (or open the memo) to open its detail page
4. Look at the detail sidebar on the memo detail page
The reporter says the correct behavior is that the sidebar on the memo detail page shows the memo's dates as readable date/time text, and the text "[object Object]" does not appear anywhere in the sidebar. Verify that the sidebar on the memo detail page shows the memo's dates as readable date/time text, and the text "[object Object]" does not appear anywhere in the sidebar.
