---
mode: testing
url: http://localhost:15323/auth
headless: true
max_steps: 40
tags: [reenact, issue-5462]
---

# Search filter persists after clicking Shortcuts even when search box is empty

## Sign in
Sign in with username {{username}} and password {{password}}.
Verify the sign-in form is gone and the memo app's main page is shown.

## Reproduce the report
Open http://localhost:15323/.
Stay on localhost:15323 for the whole run and do not open any other website.
Do each step once, in order. If the app behaves unexpectedly, do not undo, retry, repair or work around it: the behavior itself is what is being checked. Check the result right after the last step.
Steps:
1. Click the memo editor on the home page and type 'apple pie'
2. Click the Save button to save the memo
3. Click the memo editor and type 'banana bread'
4. Click the Save button to save the memo
5. In the sidebar, click the '+' button next to Shortcuts
6. Type 'Banana' in the shortcut title field and type 'content.contains("banana")' in the filter field
7. Click Save in the shortcut dialog
8. Click the search box and type 'apple'
9. Press Enter to run the search
10. Click the 'Banana' item under Shortcuts
The reporter says the correct behavior is that After clicking the 'Banana' shortcut, the search box is empty and the memo 'banana bread' is visible in the list, because only the shortcut filter applies. Verify that After clicking the 'Banana' shortcut, the search box is empty and the memo 'banana bread' is visible in the list, because only the shortcut filter applies.
