---
mode: testing
url: http://localhost:15320/auth
headless: true
max_steps: 40
tags: [reenact, issue-5659]
---

# Memo detail page not refresh after editing and saving the content

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15320/.
Stay on localhost:15320 for the whole run and do not open any other website.
Do these in order:
1. Create a new memo with some text content (for example 'original content') and save it
2. Open the menu of the newly created memo
3. Click the 'Copy Link' menu item
4. Click the memo's timestamp or title link to open the memo detail page
5. Open the memo's menu on the detail page and click 'Edit'
6. Change the memo content to 'edited content'
7. Click the 'Save' button
The reporter says the correct behavior is that After saving, the memo detail page shows the new text 'edited content' and no longer shows 'original content', without a browser refresh. Verify that After saving, the memo detail page shows the new text 'edited content' and no longer shows 'original content', without a browser refresh.
