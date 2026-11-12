---
mode: testing
url: http://localhost:15321/auth
headless: true
max_steps: 40
tags: [reenact, issue-5659]
---

# Memo detail page not refresh after editing and saving the content

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15321/.
Stay on localhost:15321 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Create a new memo with the text 'original content' and save it
2. Open the menu of the new memo and click 'Copy Link'
3. Open the memo's detail page (the page for that single memo, as the copied link would open)
4. Click the edit option for the memo in its menu
5. Change the memo content to 'edited content'
6. Click the Save button
The reporter says the correct behavior is that After saving, the memo detail page shows the new content 'edited content' and no longer shows 'original content', without a browser refresh. Verify that After saving, the memo detail page shows the new content 'edited content' and no longer shows 'original content', without a browser refresh.
